use std::char::REPLACEMENT_CHARACTER;
use std::fmt::{Display, Formatter, Write};
use std::iter::Copied;
use std::slice;
use std::slice::Iter;

use windows_sys::core::PCWSTR;
use windows_sys::Win32::Foundation::{FALSE, HANDLE};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{
    GlobalAlloc, GlobalFree, GlobalLock, GlobalUnlock, GMEM_MOVEABLE,
};
use windows_sys::Win32::System::Ole::CF_UNICODETEXT;

use crate::ensure;
use crate::error::{WinError, WinResult};

#[derive(Debug)]
pub struct Clipboard;

impl Clipboard {
    pub fn new() -> WinResult<Self> {
        unsafe {
            ensure!(OpenClipboard(0) != FALSE, WinError::last_error());
            Ok(Self)
        }
    }

    pub fn get_text(&self) -> WinResult<U16String> {
        unsafe {
            let sys_str = SystemString::new(GetClipboardData(CF_UNICODETEXT as _))?;
            let usr_str = U16String::from(&sys_str);
            Ok(usr_str)
        }
    }

    /// Write a UTF-16 slice to the system clipboard as CF_UNICODETEXT.
    /// Opens and closes the clipboard within this call.
    pub fn set_text(text: &[u16]) -> WinResult<()> {
        unsafe {
            // Allocate global memory: text + null terminator, in bytes
            let size = (text.len() + 1) * std::mem::size_of::<u16>();
            let h_glob = GlobalAlloc(GMEM_MOVEABLE, size);
            ensure!(h_glob != 0, WinError::last_error());

            let dst = GlobalLock(h_glob);
            if dst.is_null() {
                GlobalFree(h_glob);
                return Err(WinError::last_error());
            }
            std::ptr::copy_nonoverlapping(text.as_ptr(), dst as *mut u16, text.len());
            // Null-terminate
            *(dst as *mut u16).add(text.len()) = 0;
            GlobalUnlock(h_glob);

            ensure!(OpenClipboard(0) != FALSE, {
                GlobalFree(h_glob);
                WinError::last_error()
            });
            EmptyClipboard();
            let result = SetClipboardData(CF_UNICODETEXT as _, h_glob as _);
            CloseClipboard();

            if result == 0 {
                GlobalFree(h_glob);
                return Err(WinError::last_error());
            }
            // System now owns the memory — do NOT GlobalFree.
            Ok(())
        }
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        unsafe {
            CloseClipboard();
        }
    }
}

struct SystemString {
    handle: HANDLE,
    ptr: PCWSTR
}

impl SystemString {
    unsafe fn new(handle: HANDLE) -> WinResult<Self> {
        ensure!(handle != 0, WinError::last_error());
        let ptr = GlobalLock(handle) as PCWSTR;
        ensure!(!ptr.is_null(), WinError::last_error());
        Ok(Self { handle, ptr })
    }
    unsafe fn len(&self) -> usize {
        let mut ptr = self.ptr;
        let mut length = 0;
        while *ptr != 0 {
            length += 1;
            ptr = ptr.offset(1);
        }
        length
    }
    fn as_slice(&self) -> &[u16] {
        unsafe { slice::from_raw_parts(self.ptr, self.len()) }
    }
}

impl Drop for SystemString {
    fn drop(&mut self) {
        unsafe {
            GlobalUnlock(self.handle);
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct U16String(Vec<u16>);

impl U16String {
    pub fn as_slice(&self) -> &[u16] {
        &self.0
    }
}

impl From<&SystemString> for U16String {
    fn from(value: &SystemString) -> Self {
        Self(Vec::from(value.as_slice()))
    }
}

impl Display for U16String {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for c in char::decode_utf16(self.0.iter().copied()).map(|r| r.unwrap_or(REPLACEMENT_CHARACTER)) {
            f.write_char(c)?;
        }
        Ok(())
    }
}

impl<'a> IntoIterator for &'a U16String {
    type Item = u16;
    type IntoIter = Copied<Iter<'a, Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}
