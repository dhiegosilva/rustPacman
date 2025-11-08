# Rust Pacman Game

A Pacman-like game built with Rust and SDL2.

## Prerequisites

- Rust (installed via rustup)
- MSYS2 with MinGW-w64 toolchain
- SDL2 development libraries (installed via MSYS2)

## Building

### Option 1: Use the build script (Recommended)

```powershell
.\build.ps1 build
.\build.ps1 run
.\build.ps1 run --release
```

### Option 2: Manual setup

Add MinGW to your PATH for the current session:

```powershell
$env:PATH = "C:\msys64\mingw64\bin;$env:PATH"
cargo build
cargo run
```

### Option 3: Permanent PATH setup

Add `C:\msys64\mingw64\bin` to your system PATH environment variable permanently.

## Running

```powershell
cargo run
```

Or run the executable directly:
```powershell
.\target\debug\paclike_2600_rs.exe
```

For optimized release build:
```powershell
cargo build --release
.\target\release\paclike_2600_rs.exe
```

## Controls

- Arrow keys: Move Pacman
- ESC: Quit game

