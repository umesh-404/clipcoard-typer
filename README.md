# clipboard-typer

A Windows program designed to flawlessly inject clipboard text into "smart" browser-based online IDEs (like CodeChef, LeetCode, HackerRank) that use editors such as Ace, Monaco, or CodeMirror. 

Traditional copy-pasting or character-by-character key simulations often fail in these environments because the editors' auto-indent, auto-bracketing, and syntax helpers interfere, resulting in garbled or double-indented code.

This tool bypasses those issues entirely by leveraging Chrome DevTools and direct JavaScript API injection.

## How It Works

Instead of simulating raw key presses character-by-character, `clipboard-typer` perfectly preserves your format and indentation by writing directly into the editor's internal document model:

1. You copy your source code to the clipboard.
2. When you hit the `Insert` hotkey, the program:
   - Swaps your clipboard with a short JavaScript snippet specifically designed for the target editor instance (Ace, Monaco, or CodeMirror).
   - Opens the Chrome DevTools Console (`Ctrl` + `Shift` + `J`).
   - Pastes the JavaScript command.
   - Restores your original codebase back to the clipboard.
   - Executes the script (`Enter`), which safely reads the clipboard and updates the editor natively.
   - Closes the DevTools Console (`F12`).

Because the code is inserted atomically via the editor's native API (`setValue()`), formatting remains 100% accurate.

## Prerequisites

- **OS:** Windows
- **Browser:** Tested with Google Chrome (relies on standard Chrome DevTools shortcuts).
- **Environment:** Install Rust and Cargo to build from source.

## Building and Running

```bash
git clone https://github.com/sidit77/clipboard-typer.git
cd clipboard-typer
cargo build --release
```

After building, start the binary. You can run it via the terminal:
```powershell
.\target\release\clipboard-typer.exe
```

## Usage

1. Start `clipboard-typer.exe` so it is running in the background.
2. Navigate to your coding problem in Chrome (e.g., CodeChef).
3. Copy your target C++ (or other language) code to your clipboard.
4. Click anywhere inside the Chrome browser window to ensure it has focus.
5. Press the **`Insert`** key.
6. **Wait (~6 seconds):** You will see the DevTools console pop up briefly, a snippet pasted, your code appear in the editor perfectly formatted, and then the console close. *Do not press any keys or click while the injection is happening.*

## Editor Support

The injected script automatically detects the underlying editor framework and uses the correct API:
- **Ace Editor** (CodeChef, HackerRank)
- **Monaco Editor** (LeetCode, VS Code web environments)
- **CodeMirror** (Many generic online IDEs)

## License
MIT License
