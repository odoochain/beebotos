@echo off
REM Build environment setup for BeeBotOS on Windows

set WindowsSdkDir=D:\Windows Kits\10
set WindowsSDKVersion=10.0.22621.0

set INCLUDE=D:\Windows Kits\10\Include\10.0.22621.0\ucrt;D:\Windows Kits\10\Include\10.0.22621.0\um;D:\Windows Kits\10\Include\10.0.22621.0\shared;D:\program_files\Microsoft Visual Studio\2022\Professional\VC\Tools\MSVC\14.39.33519\include

set LIB=D:\Windows Kits\10\Lib\10.0.22621.0\ucrt\x64;D:\Windows Kits\10\Lib\10.0.22621.0\um\x64;D:\program_files\Microsoft Visual Studio\2022\Professional\VC\Tools\MSVC\14.39.33519\lib\x64

echo Environment variables set for building BeeBotOS
echo.
echo You can now run: cargo check -p beebotos-agents --lib
