# CYBER TOMATO

A minimalist terminal-based pomodoro application built with Rust.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Terminal](https://img.shields.io/badge/Terminal-UI-green?style=for-the-badge)


## Features

- **Beautiful TUI**: Clean terminal interface with cyberpunk green theme
- **Visual Progress**: Real-time progress bar with time display
- **Smart Controls**: Intuitive keyboard controls with popup help (Press **X**)
- **Keyboard-Driven**: Lightning-fast keyboard-only interface

## Quick Start

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)

### Installation

```bash
# Clone the repository
git clone git@github.com:coolcode/cyber-tomato-cli.git
cd cyber-tomato-cli

# Build and run
cargo run

# Or build optimized release version
cargo build --release
./target/release/cyber-tomato
```


## Controls

> **Tip**: Press **X** anytime to view the interactive controls popup!

### Essential Keys

| Key | Action |
|-----|--------|
| `W` | Start 25 mins Work |
| `B` | 5 mins Break |
| `C` | Custom mins, popup an input box for mins, eg. "30,10"=30mins work+10mins break; "20"=20mins work, then 5mins break as default value. |
| `Space/↵` | Pause Resume timer |
| `T` | Toggle Manual/Auto mode, auto means loop work+break for 20 times |
| `X` | Help |
| `Esc` | Exit application |

## Interface

CYBER TOMATO features a clean, 4-panel interface that maximizes space for your music:

```
┌─────────────────────────────────┐
│         CYBER TOMATO            │  ← Title Bar
├─────────────────────────────────┤
│ → 25 mins Work                  │  ← Menu List
│   5 mins Break                  │    
│   Custom                        │
│                                 │
├─────────────────────────────────┤
│ ████████████████░░░░ 02:30/04:15│  ← Progress Bar
├─────────────────────────────────┤
│ Mode: ___ | Done: 5 | X: Help   │  ← Status & Help
└─────────────────────────────────┘
```
## Technical Details


### Core Dependencies
- **`rodio`** - Play a sound effect once work/break completed 
- **`ratatui`** - Modern terminal user interface framework
- **`crossterm`** - Cross-platform terminal control

## Development

### Project Structure

```
cyber-tomato/
├── src/
│   └── main.rs          # Complete application 
├── .github/workflows/   # CI/CD automation
├── Cargo.toml          # Dependencies and metadata
├── rustfmt.toml        # Code formatting rules
└── README.md           # Documentation
```

### Building & Testing

```bash
# Development build
cargo build

# Optimized release build
cargo build --release

# Run all tests
cargo test

# Code quality checks
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Acknowledgments

- **ASCII-ART Numbers** from [yuanqing](https://gist.github.com/yuanqing/ffa2244bd134f911d365#file-gistfile1-txt)
- **Rodio** team for excellent Rust audio library
- **Ratatui** team for powerful TUI framework
- **Rust** community for amazing ecosystem

---

**Built with Claud Code**
