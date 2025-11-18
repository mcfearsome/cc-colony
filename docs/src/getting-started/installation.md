# Installation

This guide will help you install Colony on your system.

## Prerequisites

### Required
- **Rust** (1.70+): Install from [rustup.rs](https://rustup.rs)
- **tmux** (2.0+): Terminal multiplexer for agent isolation
- **Git**: For state management

### Optional
- **Claude Code**: The AI coding assistant (agents will use this)
- **MCP Servers**: For extended functionality

## Installing from Source

Currently, Colony must be built from source:

```bash
# Clone the repository
git clone https://github.com/yourusername/cc-colony.git
cd cc-colony

# Build the project
cargo build --release

# Install to ~/.cargo/bin
cargo install --path .
```

The `colony` binary will be available in your PATH if `~/.cargo/bin` is included.

## Installing tmux

### macOS
```bash
brew install tmux
```

### Ubuntu/Debian
```bash
sudo apt-get install tmux
```

### Fedora/RHEL
```bash
sudo dnf install tmux
```

### Arch Linux
```bash
sudo pacman -S tmux
```

## Verifying Installation

Check that everything is installed correctly:

```bash
# Check Colony version
colony --version

# Check tmux version
tmux -V

# Check Git version
git --version

# Check Rust version
rustc --version
```

## Configuration

After installation, initialize your first colony:

```bash
# Navigate to your project directory
cd /path/to/your/project

# Initialize colony configuration
colony init
```

This creates a `colony.yml` configuration file in your project directory.

## Updating Colony

To update Colony to the latest version:

```bash
cd cc-colony
git pull
cargo install --path .
```

## Troubleshooting

### tmux not found
If `colony start` fails with "tmux not found":
1. Install tmux using your package manager
2. Verify tmux is in your PATH: `which tmux`

### Permission denied
If you get permission errors:
```bash
# Ensure the binary is executable
chmod +x ~/.cargo/bin/colony
```

### Build failures
If cargo build fails:
1. Update Rust: `rustup update`
2. Clean build artifacts: `cargo clean`
3. Try building again: `cargo build --release`

## Next Steps

- [Quick Start](./quick-start.md) - Create your first colony
- [Configuration](./configuration.md) - Configure your colony
