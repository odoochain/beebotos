@echo off
call "d:\Program_Files\Microsoft Visual Studio\2022\Professional\Common7\Tools\VsDevCmd.bat" -arch=x64
cargo clean
cargo build
