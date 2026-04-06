mod clipboard;
mod error;
mod hotkey;
mod input;
mod messages;

use windows_sys::Win32::UI::Input::KeyboardAndMouse::{MOD_NOREPEAT, VK_INSERT};

use crate::clipboard::Clipboard;
use crate::error::WinResult;
use crate::hotkey::GlobalHotkey;
use crate::input::type_string;
use crate::messages::{run_event_loop, Event};

const POWER_PASTE: i32 = 0x0001;

fn main() -> WinResult<()> {
    let _hotkey = GlobalHotkey::register(POWER_PASTE, MOD_NOREPEAT, VK_INSERT)?;
    run_event_loop(|event| {
        if let Event::Hotkey(POWER_PASTE) = event {
            let text = Clipboard::new()?.get_text()?;
            type_string(&text)?;
        }
        Ok(())
    })?;
    Ok(())
}
