# Installation

There is no prebuilt binary yet, so installing means building from source. It takes one command once the prerequisites are in place.

## Prerequisites

| | Requirement | Why |
|---|---|---|
| Rust | **1.85 or newer** | the crate uses edition 2024 |
| C toolchain | platform linker | Rust needs a system linker; some dependencies build C |
| LLVM 18 | *optional* | only for the `--llvm` backend, Linux/macOS only |

Install Rust from [rustup.rs](https://rustup.rs).

### Linux

```bash
sudo apt install build-essential      # Debian/Ubuntu
sudo dnf groupinstall "Development Tools"   # Fedora
```

### macOS

```bash
xcode-select --install
```

### Windows

Rust's default toolchain is `x86_64-pc-windows-msvc`, which needs the Microsoft linker. Installing Rust alone is **not** enough — without `link.exe` even `cargo check` fails, because procedural-macro crates are compiled and linked as DLLs during the build.

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools -e --override "--quiet --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

Open a new terminal afterwards so `PATH` picks up both tools.

## Build

```bash
git clone https://github.com/Maruff/etamil_compiler.git
cd etamil_compiler/etamil_compiler
cargo build --release
```

The binary lands at `target/release/etamil` (`etamil.exe` on Windows).

## Install the binary

### Linux / macOS

```bash
cd ..
./install.sh
```

`install.sh` copies the binary to `~/.local/bin/etamil`, or to `/usr/local/bin/etamil` when run with `sudo`. It builds first if the binary is missing.

### Windows

```powershell
New-Item -ItemType Directory -Force "$env:USERPROFILE\bin"
Copy-Item "target\release\etamil.exe" "$env:USERPROFILE\bin\etamil.exe"
```

Add `%USERPROFILE%\bin` to your user `PATH` through **Settings → System → About → Advanced system settings → Environment Variables**, then open a new terminal.

## Verify

```bash
etamil --version
etamil --help
```

Then run a real program:

```bash
echo "950000" | etamil --vm examples/basic_samples/example.qmz
```

Expected output ends with:

```
High Tax Bracket
30000

✓ Execution completed successfully
```

## Optional: the LLVM backend

Off by default. It requires **LLVM 18 specifically** — `llvm-sys` is pinned to `"180"`, so a newer LLVM will not link — and it is not available on Windows.

```bash
cargo build --release --features llvm
```

The LLVM backend currently covers a smaller subset than the VM. Verify the
build and IR path with a minimal arithmetic program:

```bash
printf 'எண் x = 2 + 3;\nஅச்சு x;\n' >/tmp/etamil_llvm_smoke.qmz
target/release/etamil --llvm /tmp/etamil_llvm_smoke.qmz
```

This writes `output.ll`. Programs using unsupported features are refused with
an explicit diagnostic; use `--vm` for the full language.

## Troubleshooting

**`etamil: command not found`** — the install directory is not on `PATH`.

```bash
export PATH="$PATH:$HOME/.local/bin"
echo 'export PATH="$PATH:$HOME/.local/bin"' >> ~/.bashrc
```

**`linker 'link.exe' not found` (Windows)** — Visual Studio Build Tools with the C++ workload is missing. See the Windows prerequisites above.

**`edition2024 is required`** — Rust is older than 1.85. Run `rustup update`.

**Build fails after changing dependencies**

```bash
cargo clean && cargo build --release
```

## Uninstall

```bash
rm ~/.local/bin/etamil          # user install
sudo rm /usr/local/bin/etamil   # system install
```

---

Next: [Quick Start](QUICKSTART.md) · [Command Reference](../reference/COMMANDS.md)
