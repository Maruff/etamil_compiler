# eTamil Installation Guide

Complete guide to installing and setting up eTamil compiler.

## Quick Install (Recommended)

```bash
./install.sh
```

That's it! The `etamil` command is now available.

## What Gets Installed

- **Binary**: `~/.local/bin/etamil` (or `/usr/local/bin/etamil` with sudo)
- **Size**: 2.1 MB
- **Dependencies**: None (standalone binary)

## Verification

```bash
# Check installation
which etamil

# Test execution
./test_installation.sh
```

## Manual Installation

If you prefer manual installation:

```bash
# Build the compiler (requires Rust - one time only)
cd etamil_compiler
cargo build --release

# Copy binary
mkdir -p ~/.local/bin
cp target/release/etamil_compiler ~/.local/bin/etamil
chmod +x ~/.local/bin/etamil

# Add to PATH
echo 'export PATH="$PATH:$HOME/.local/bin"' >> ~/.bashrc
source ~/.bashrc
```

## System-Wide Installation

```bash
# Run as root
sudo ./install.sh

# Binary will be at /usr/local/bin/etamil
```

## Uninstallation

```bash
# User installation
rm ~/.local/bin/etamil

# System installation
sudo rm /usr/local/bin/etamil
```

## Troubleshooting

### "etamil: command not found"

Add installation directory to PATH:
```bash
export PATH="$PATH:$HOME/.local/bin"
echo 'export PATH="$PATH:$HOME/.local/bin"' >> ~/.bashrc
```

### "Permission denied"

Make binary executable:
```bash
chmod +x ~/.local/bin/etamil
```

### Build errors

Ensure Rust is up to date:
```bash
rustup update
cargo clean
cargo build --release
```

## Next Steps

- [Quick Start Guide](QUICKSTART.md)
- [Command Reference](../reference/COMMANDS.md)
- [Examples](../reference/EXAMPLES.md)

---

**No Rust Required for End Users!** ✅
