# MemoChan

A simple, lightweight notepad application built with the Iced GUI framework.

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## About

MemoChan is a minimal text editor designed for everyday note-taking and text editing tasks. It provides a clean, distraction-free writing experience with essential features you would expect from a notepad application.

**This application was implemented by the GLM5 AI model.**

## Features

- **File Operations**: Create new files, open existing text files, and save your work
- **Edit Functions**: Cut, copy, paste, delete, and select all text
- **Format Options**: Toggle word wrap for better readability
- **View Options**: Zoom in/out, reset zoom, and toggle status bar visibility
- **Status Bar**: Shows current line and column position, zoom level, and encoding
- **Keyboard Shortcuts**: Quick access to common functions
- **Mouse Wheel Zoom**: Hold Ctrl and scroll to zoom in/out

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| New File | Ctrl+N |
| Open File | Ctrl+O |
| Save File | Ctrl+S |
| Cut | Ctrl+X |
| Copy | Ctrl+C |
| Paste | Ctrl+V |
| Select All | Ctrl+A |
| Zoom In | Ctrl++ |
| Zoom Out | Ctrl+- |
| Reset Zoom | Ctrl+0 |
| Time/Date | F5 |
| Delete | Del |

## Known Issues

1. **Undo Menu Not Working**: The undo function in the Edit menu is currently disabled and does not perform any action.

2. **Keyboard Shortcuts with Korean Input**: When the system is in Korean input mode (IME), keyboard shortcuts may not work properly. Please switch to English input mode to use keyboard shortcuts.

## Requirements

- Rust 1.70 or later
- macOS, Windows, or Linux

## Installation

### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/ppugend/memochan.git
   cd memochan
   ```

2. Build and run:
   ```bash
   cargo run --release
   ```

### Development Build

```bash
cargo run
```

## Technology Stack

- **GUI Framework**: [Iced](https://iced.rs/) 0.14
- **Language**: Rust
- **Font**: Pretendard Variable

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Ppugend**

---

*Made with ❤️ and AI assistance*