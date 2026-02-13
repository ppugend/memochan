# MemoChan

A simple, lightweight notepad application built with the egui GUI framework.

![Version](https://img.shields.io/badge/version-0.2.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## About

MemoChan is a minimal text editor designed for everyday note-taking and text editing tasks. It provides a clean, distraction-free writing experience with essential features you would expect from a notepad application.

**This application was implemented by the GLM5 AI model.**

## Features

- **File Operations**: New, Open, Save, Save As
- **Undo/Redo**: Full undo/redo support with history stack
- **Word Wrap**: Toggle line wrapping for better readability
- **Zoom**: Zoom in/out with font size adjustment
- **Status Bar**: Shows current line, column, zoom level, and encoding (UTF-8)
- **Tab Support**: Tab key inserts tab character in editor
- **Time/Date Insertion**: Insert current date and time

## Menu Structure

### File
| Item | Shortcut |
|------|----------|
| New | `Ctrl+N` / `Cmd+N` |
| Open... | `Ctrl+O` / `Cmd+O` |
| Save | `Ctrl+S` / `Cmd+S` |
| Save As... | `Shift+Ctrl+S` / `Shift+Cmd+S` |
| Exit | - |

### Format
| Item | Description |
|------|-------------|
| Word Wrap | Toggle line wrapping |

### View
| Item | Shortcut |
|------|----------|
| Zoom In | `Ctrl++` |
| Zoom Out | `Ctrl+-` |
| Reset Zoom | `Ctrl+0` |
| Status Bar | Toggle visibility |

### Help
| Item | Description |
|------|-------------|
| About MemoChan | Show about dialog |

## Keyboard Shortcuts

| Action | Windows/Linux | macOS |
|--------|---------------|-------|
| New File | `Ctrl+N` | `Cmd+N` |
| Open File | `Ctrl+O` | `Cmd+O` |
| Save File | `Ctrl+S` | `Cmd+S` |
| Save As | `Shift+Ctrl+S` | `Shift+Cmd+S` |
| Undo | `Ctrl+Z` | `Cmd+Z` |
| Redo | `Ctrl+Y` or `Ctrl+Shift+Z` | `Cmd+Y` or `Cmd+Shift+Z` |
| Zoom In | `Ctrl++` | `Ctrl++` |
| Zoom Out | `Ctrl+-` | `Ctrl+-` |
| Reset Zoom | `Ctrl+0` | `Ctrl+0` |
| Time/Date | `F5` | `F5` |
| Close Dialog | `Esc` or `Enter` | `Esc` or `Enter` |

## Requirements

- Rust 1.93 or later
- macOS, Windows, or Linux

## Supported Platforms

| OS | Architecture | Status |
|----|--------------|--------|
| macOS | amd64 (Intel) | ✅ Tested |
| macOS | arm64 (Apple Silicon) | ✅ Supported |
| Linux | amd64 | ✅ Supported |
| Linux | arm64 | ✅ Tested |
| Windows | amd64 | ✅ Supported |
| Windows | arm64 | ✅ Supported |

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

- **GUI Framework**: [egui](https://www.egui.rs/) 0.30 / eframe
- **Language**: Rust
- **Font**: Pretendard Variable

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Ppugend**

---

*Made with AI assistance*