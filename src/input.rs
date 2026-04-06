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

    // Put the JS command on clipboard
    Clipboard::set_text(&js_utf16)?;
    thread::sleep(Duration::from_millis(100));

    // Open Chrome DevTools Console
    send_input(&ctrl_shift_vk(VK_J))?;
    thread::sleep(Duration::from_millis(3000));

    // Paste the JS command
    send_input(&ctrl_vk(VK_V))?;
    thread::sleep(Duration::from_millis(1000));

    // Execute
    send_input(&vk_press(VK_RETURN))?;
    thread::sleep(Duration::from_millis(1500));

    // Close DevTools
    send_input(&vk_press(VK_F12))?;
    thread::sleep(Duration::from_millis(500));

    // Restore original clipboard
    Clipboard::set_text(string.as_slice())?;

    Ok(())
}

/// Build a universal JS command that tries Ace → Monaco → CodeMirror.
/// Uses a template literal (backtick string) to preserve multi-line formatting.
fn build_js_command(text: &str) -> String {
    // Escape for JS template literal
    let escaped = text
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");

    // Universal editor detection: Ace, Monaco, CodeMirror
    // Ace: document.querySelector('.ace_editor').env.editor.setValue(text, -1)
    // Monaco: monaco.editor.getModels()[0].setValue(text)
    // CodeMirror: document.querySelector('.CodeMirror').CodeMirror.setValue(text)
    format!(
        concat!(
            "(function(){{",
            "var t=`{0}`;",
            "var a=document.querySelector('.ace_editor');",
            "if(a&&a.env&&a.env.editor){{a.env.editor.setValue(t,-1);return}}",
            "if(typeof monaco!=='undefined'){{monaco.editor.getModels()[0].setValue(t);return}}",
            "var c=document.querySelector('.CodeMirror');",
            "if(c&&c.CodeMirror){{c.CodeMirror.setValue(t);return}}",
            "console.error('No editor found')",
            "}})()"
        ),
        escaped
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

fn vk_press(vk: VIRTUAL_KEY) -> [INPUT; 2] {
    [make_vk_input(vk, true), make_vk_input(vk, false)]
}

fn ctrl_vk(vk: VIRTUAL_KEY) -> [INPUT; 4] {
    [
        make_vk_input(VK_CONTROL, true),
        make_vk_input(vk, true),
        make_vk_input(vk, false),
        make_vk_input(VK_CONTROL, false),
    ]
}

fn ctrl_shift_vk(vk: VIRTUAL_KEY) -> [INPUT; 6] {
    [
        make_vk_input(VK_CONTROL, true),
        make_vk_input(VK_SHIFT, true),
        make_vk_input(vk, true),
        make_vk_input(vk, false),
        make_vk_input(VK_SHIFT, false),
        make_vk_input(VK_CONTROL, false),
    ]
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
