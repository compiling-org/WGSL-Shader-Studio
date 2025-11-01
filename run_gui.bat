@echo off
echo Starting WGSL Shader Studio GUI...
cd /d "%~dp0"
cargo run --bin isf-shaders --no-default-features --features gui
pause