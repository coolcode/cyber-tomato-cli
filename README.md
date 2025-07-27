# CYBER TOMATO

A minimalist terminal-based pomodoro application built with Rust, featuring ASCII art timer display, Mario-style animations, and immersive audio feedback.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Terminal](https://img.shields.io/badge/Terminal-UI-green?style=for-the-badge)
![Audio](https://img.shields.io/badge/Audio-Enabled-blue?style=for-the-badge)


[![asciicast](https://asciinema.org/a/730138.svg)](https://asciinema.org/a/730138)

## Features

### Core Timer Functions
- **Work Sessions**: Default 25-minute focused work periods
- **Break Sessions**: 5-minute rest periods with relaxing completion music
- **Custom Timers**: Flexible timing with format like "30,10" (30min work + 10min break) or "20" (20min work + 5min default break)
- **Auto/Manual Modes**: Auto mode cycles between work and break sessions automatically

### Visual Experience
- **Large ASCII Art Timer**: Eye-catching countdown display with custom digit patterns
- **Cyberpunk Green Theme**: Consistent neon green aesthetic throughout the interface
- **Real-time Progress Bar**: Visual progress tracking with percentage display
- **Clean TUI Layout**: 4-panel interface optimized for terminal use

### Audio & Animation
- **Mario Animation**: Delightful Super Mario-style brick-breaking animation for work completion
- **Synchronized Music**: Mario Bros theme music with sound effects during animations
- **Break Completion Music**: 6-second melodic sequence to signal end of break time
- **Work Completion Sounds**: Quick notification tones for work session completion

### Keyboard-Driven Interface
- **Lightning-fast Controls**: All functions accessible via single keypresses
- **Interactive Help**: Press **X** for comprehensive controls popup
- **Custom Timer Input**: Intuitive dialog with format validation and examples

## Quick Start

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Audio System** - Ensure your system has working audio output

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

| Key | Action | Description |
|-----|--------|-------------|
| `W` | Start Work Session | Begin 25-minute work period |
| `B` | Start Break Session | Begin 5-minute break period |
| `C` | Custom Timer | Open custom timer input dialog |
| `Space`/`Enter` | Pause/Resume | Toggle timer pause state |
| `T` | Toggle Mode | Switch between Manual/Auto modes |
| `M` | Mario Animation | Trigger Mario animation (for testing) |
| `X` | Help | Show/hide controls popup |
| `Esc` | Exit | Quit application |

### Custom Timer Format

- **"30,10"** → 30 minutes work + 10 minutes break
- **"20"** → 20 minutes work + 5 minutes default break
- **Numbers only** → Work time with 5-minute default break

## Interface Layout

CYBER TOMATO features a clean, bordered interface:

```
┌─────────────────────────────────┐
│         CYBER TOMATO            │  ← Title Bar
├─────────────────────────────────┤
│                                 │
│     ██████   ██████   ██████    │  ← ASCII Art Timer
│     ██  ██   ██  ██   ██  ██    │    (Large Digital Display)
│     ██████   ██████   ██████    │
│                                 │
├─────────────────────────────────┤
│ ███████████████████████░░░ 85%  │  ← Progress Bar
├─────────────────────────────────┤
│ Mode: Auto | Status: Working    │  ← Status & Help
│ Done: 5 | X: Help               │
└─────────────────────────────────┘
```

## Special Features

### Mario Animation
- Triggered automatically when work sessions complete
- Features Mario jumping, hitting bricks, and collecting mushrooms
- Synchronized with classic Mario Bros theme music
- Interactive brick-breaking physics simulation

### Audio System
- **Work Completion**: Quick notification beeps
- **Break Completion**: Musical melody (notification tones + 6-second relaxing tune)
- **Mario Animation**: Full theme song with jump, brick-break, and power-up sound effects
- **High-Quality Audio**: Square wave synthesis with decay envelopes

### Auto Mode
- Automatically cycles between work and break sessions
- Perfect for uninterrupted pomodoro technique practice
- Visual and audio feedback for session transitions

## Technical Details

### Core Dependencies
- **`rodio 0.20`** - High-quality audio playback and synthesis
- **`ratatui 0.29`** - Modern terminal user interface framework
- **`crossterm 0.29`** - Cross-platform terminal control

### Project Structure

```
cyber-tomato-cli/
├── src/
│   ├── main.rs              # Core application logic
│   ├── audio.rs             # Audio management and synthesis
│   ├── mario_animation.rs   # Mario animation system
│   └── ascii_digits.rs      # ASCII art digit rendering
├── Cargo.toml              # Dependencies and metadata
├── rustfmt.toml            # Code formatting rules
└── README.md               # This documentation
```

### Audio Implementation
- **Fresh Stream Architecture**: Creates new audio streams for each playback
- **Custom Synthesis**: Square wave generation with exponential decay
- **Synchronized Playback**: Coordinated music and sound effects
- **Cross-Platform**: Works on Linux, macOS, and Windows

## Development

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

### Audio Testing
The application gracefully handles systems without audio:
- Displays warning messages for audio initialization failures
- Continues normal timer operation without sound
- All visual features remain fully functional

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Submit a pull request

## Acknowledgments

- **ASCII Art Numbers** from [yuanqing](https://gist.github.com/yuanqing/ffa2244bd134f911d365#file-gistfile1-txt)
- **Rodio Team** for excellent Rust audio library
- **Ratatui Team** for powerful TUI framework
- **Rust Community** for amazing ecosystem and tools
- **Super Mario Bros** for inspiration on the animation system

---

**Built with Claude Code**

*Perfect your productivity with the timeless pomodoro technique, enhanced by retro gaming charm and modern Rust performance.*