# Setup script for Rust Pacman game
# This adds MinGW to PATH for the current session

$env:PATH = "C:\msys64\mingw64\bin;$env:PATH"

Write-Host "MinGW added to PATH. You can now run:" -ForegroundColor Green
Write-Host "  cargo build    - to compile" -ForegroundColor Yellow
Write-Host "  cargo run      - to compile and run" -ForegroundColor Yellow
Write-Host "  cargo run --release  - for optimized build" -ForegroundColor Yellow

# Run cargo with the provided arguments
if ($args.Count -gt 0) {
    cargo $args
}

