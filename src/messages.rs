use std::sync::atomic::{AtomicU32, Ordering};

use windows_sys::Win32::Foundation::{BOOL, FALSE, TRUE};
use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, PostThreadMessageW, TranslateMessage, MSG, WM_HOTKEY, WM_QUIT};

use crate::ensure;
use crate::error::{WinError, WinResult};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Event {
    Hotkey(i32)
}

impl TryFrom<MSG> for Event {
    type Error = ();

    fn try_from(value: MSG) -> Result<Self, Self::Error> {
        if value.message == WM_HOTKEY {
            Ok(Event::Hotkey(value.wParam as _))
        } else {
            Err(())
        }
    }
}

pub fn run_event_loop<F>(mut handler: F) -> WinResult<()>
where
    F: FnMut(Event) -> WinResult<()>
{
    let _console_hook = ConsoleQuitHook::current_thread()?;
    while let Some(msg) = next_msg()? {
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        if let Ok(event) = Event::try_from(msg) {
            handler(event)?;
        }
    }
    Ok(())
}

fn next_msg() -> WinResult<Option<MSG>> {
    unsafe {
        let mut msg = std::mem::zeroed();
        match GetMessageW(&mut msg, 0, 0, 0) {
            -1 => Err(WinError::last_error()),
            0 => Ok(None),
            _ => Ok(Some(msg))
        }
    }
}

static MAIN_THREAD: AtomicU32 = AtomicU32::new(0);
unsafe extern "system" fn ctrl_handler(_: u32) -> BOOL {
    PostThreadMessageW(MAIN_THREAD.load(Ordering::Acquire), WM_QUIT, 0, 0);
    TRUE
}

struct ConsoleQuitHook;

impl ConsoleQuitHook {
    fn current_thread() -> WinResult<Self> {
        ensure!(MAIN_THREAD.load(Ordering::Acquire) == 0, WinError::AlreadyRegistered);
        unsafe {
            ensure!(SetConsoleCtrlHandler(Some(ctrl_handler), TRUE) != FALSE, WinError::last_error());
            MAIN_THREAD.store(GetCurrentThreadId(), Ordering::Release);
        }
        Ok(Self)
    }
}

impl Drop for ConsoleQuitHook {
    fn drop(&mut self) {
        MAIN_THREAD.store(0, Ordering::Release);
    }
}
