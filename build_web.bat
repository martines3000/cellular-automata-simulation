@echo off

SET script_path=%~dp0
cd %script_path%

for %%I in (.) do SET FOLDER_NAME=%%~nxI

@REM assume crate name is the same as the folder name
SET CRATE_NAME=%FOLDER_NAME%

@REM for those who name crates with-kebab-case
SET CRATE_NAME_SNAKE_CASE=%FOLDER_NAME:-=_%

@REM This is required to enable the web_sys clipboard API which egui_web uses
@REM https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Clipboard.html
@REM https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html
SET RUSTFLAGS=--cfg=web_sys_unstable_apis

@REM Clear output from old stuff:
DEL /F docs\%CRATE_NAME_SNAKE_CASE%_bg.wasm

echo Building rust...
SET BUILD=release
cargo build -p %CRATE_NAME% --release --lib --target wasm32-unknown-unknown

@REM Get the output directory (in the workspace it is in another location)
FOR /F %%i IN ('cargo metadata --format-version=1 ^| jq --raw-output .target_directory') DO SET TARGET=%%i

echo Generating JS bindings for wasm...
SET TARGET_NAME=%CRATE_NAME_SNAKE_CASE%.wasm
wasm-bindgen "%TARGET%\wasm32-unknown-unknown\%BUILD%\%TARGET_NAME%" --out-dir "docs" --no-modules --no-typescript

echo Optimizing wasm...
@REM to get wasm-opt:  apt/brew/dnf install binaryen
@REM add -g to get debug symbols :
wasm-opt "docs\%CRATE_NAME%_bg.wasm" -O2 --fast-math -o "docs\%CRATE_NAME%_bg.wasm"

echo Finished: docs/%CRATE_NAME_SNAKE_CASE%.wasm"

GOTO end_program

:print_help
echo build_web.sh [--fast] [--open]
echo   --fast: skip optimization step
echo   --open: open the result in a browser
GOTO end_program

:end_program
