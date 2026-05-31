use std::mem::size_of;
use std::thread;
use std::time::Duration;

use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

use crate::clipboard::{Clipboard, U16String};
use crate::ensure;
use crate::error::{WinError, WinResult};

/// Inject clipboard code into the focused browser's code editor.
///
/// Flow:
///  1. Build a universal JS command that detects Ace/Monaco/CodeMirror
///  2. Put the JS command on clipboard
///  3. Open Chrome DevTools Console (Ctrl+Shift+J)
///  4. Ctrl+V to paste the JS command
///  5. Enter to execute
///  6. F12 to close DevTools
///  7. Restore original clipboard
pub fn type_string(string: &U16String) -> WinResult<()> {
    let text: String = string.to_string();
    let js = build_js_command(&text);
    let js_utf16: Vec<u16> = js.encode_utf16().collect();

    // Give the user a moment to release the hotkey (e.g. Insert) so it doesn't interfere
    thread::sleep(Duration::from_millis(300));

    // Put the JS command on clipboard
    Clipboard::set_text(&js_utf16)?;
    thread::sleep(Duration::from_millis(100));

    // Open Chrome DevTools Console
    press_ctrl_shift(VK_J)?;
    thread::sleep(Duration::from_millis(4000));

    // 1. Type "allow pasting" to unlock (in case it is restricted)
    type_unicode("allow pasting")?;
    press_key(VK_RETURN)?;
    thread::sleep(Duration::from_millis(500));

    // 2. Paste the actual JS command (now guaranteed to be allowed)
    press_ctrl(VK_V)?;
    thread::sleep(Duration::from_millis(1000));

    // Execute
    press_key(VK_RETURN)?;
    thread::sleep(Duration::from_millis(1500));

    // Close DevTools
    press_key(VK_F12)?;
    thread::sleep(Duration::from_millis(500));

    // Restore original clipboard
    Clipboard::set_text(string.as_slice())?;

    Ok(())
}

/// Build a universal JS command that tries Ace → Monaco → CodeMirror.
/// It inserts at the current cursor position instead of replacing the entire file.
/// We use URL encoding to ensure the payload is a single line, minimizing damage if pasted normally.
fn build_js_command(text: &str) -> String {
    let encoded = urlencoding::encode(text);
    
    // Minified JS to:
    // 1. Decode the text
    // 2. Try Ace Editor (insert at cursor)
    // 3. Try Monaco Editor (executeEdits at selection)
    // 4. Try CodeMirror (replaceSelection)
    format!(
        "(function(){{var t=decodeURIComponent(\"{0}\");var a=document.querySelector('.ace_editor');if(a&&a.env&&a.env.editor){{a.env.editor.session.insert(a.env.editor.getCursorPosition(),t);a.env.editor.focus();return}}if(typeof monaco!=='undefined'){{var m=monaco.editor.getEditors()[0];if(m){{m.executeEdits('paste',[{{range:m.getSelection(),text:t,forceMoveMarkers:true}}]);m.focus();return}}var mod=monaco.editor.getModels()[0];if(mod){{mod.setValue(t);return}}}}var c=document.querySelector('.CodeMirror');if(c&&c.CodeMirror){{c.CodeMirror.replaceSelection(t);c.CodeMirror.focus();return}}console.error('No editor found')}})()",
        encoded
    )
}

fn send_input(inputs: &[INPUT]) -> WinResult<()> {
    if inputs.is_empty() {
        return Ok(());
    }
    let sent = unsafe {
        SendInput(inputs.len() as _, inputs.as_ptr(), size_of::<INPUT>() as _) as usize
    };
    ensure!(sent == inputs.len(), WinError::last_error());
    Ok(())
}

fn press_key(vk: VIRTUAL_KEY) -> WinResult<()> {
    send_input(&[make_vk_input(vk, true)])?;
    thread::sleep(Duration::from_millis(20));
    send_input(&[make_vk_input(vk, false)])?;
    Ok(())
}

fn press_ctrl(vk: VIRTUAL_KEY) -> WinResult<()> {
    send_input(&[make_vk_input(VK_CONTROL, true)])?;
    thread::sleep(Duration::from_millis(20));
    send_input(&[make_vk_input(vk, true)])?;
    thread::sleep(Duration::from_millis(20));
    send_input(&[make_vk_input(vk, false)])?;
    thread::sleep(Duration::from_millis(20));
    send_input(&[make_vk_input(VK_CONTROL, false)])?;
    Ok(())
}

fn press_ctrl_shift(vk: VIRTUAL_KEY) -> WinResult<()> {
    send_input(&[make_vk_input(VK_CONTROL, true), make_vk_input(VK_SHIFT, true)])?;
    thread::sleep(Duration::from_millis(20));
    send_input(&[make_vk_input(vk, true)])?;
    thread::sleep(Duration::from_millis(20));
    send_input(&[make_vk_input(vk, false)])?;
    thread::sleep(Duration::from_millis(20));
    send_input(&[make_vk_input(VK_SHIFT, false), make_vk_input(VK_CONTROL, false)])?;
    Ok(())
}

const fn make_vk_input(vk: VIRTUAL_KEY, pressed: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: match pressed {
                    true => 0,
                    false => KEYEVENTF_KEYUP,
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn type_unicode(text: &str) -> WinResult<()> {
    for c in text.encode_utf16() {
        send_input(&[make_unicode_input(c, true)])?;
        thread::sleep(Duration::from_millis(20));
        send_input(&[make_unicode_input(c, false)])?;
        thread::sleep(Duration::from_millis(20));
    }
    Ok(())
}

const fn make_unicode_input(c: u16, pressed: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: c,
                dwFlags: match pressed {
                    true => KEYEVENTF_UNICODE,
                    false => KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
