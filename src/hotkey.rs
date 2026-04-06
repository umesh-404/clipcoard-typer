use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, VIRTUAL_KEY};

use crate::ensure;
use crate::error::{WinError, WinResult};

pub struct GlobalHotkey(i32);

impl GlobalHotkey {
    pub fn register(id: i32, modifiers: HOT_KEY_MODIFIERS, key: VIRTUAL_KEY) -> WinResult<Self> {
        unsafe {
            ensure!(RegisterHotKey(0, id, modifiers, key as _) != FALSE, WinError::last_error());
        }
        Ok(Self(id))
    }
}

impl Drop for GlobalHotkey {
    fn drop(&mut self) {
        unsafe {
            UnregisterHotKey(0, self.0);
        }
    }
}
