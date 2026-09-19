@echo off
setlocal EnableExtensions

rem Build CSV Compare as a Windows release executable and installer.
cd /d "%~dp0"

echo ========================================
echo CSV Compare - Release Build
echo ========================================
echo.

where npm >nul 2>&1
if errorlevel 1 (
    echo ERROR: npm was not found. Install Node.js LTS and try again.
    exit /b 1
)

where cargo >nul 2>&1
if errorlevel 1 (
    echo ERROR: cargo was not found. Install Rust and try again.
    exit /b 1
)

if not exist "node_modules" (
    echo Installing JavaScript dependencies...
    call npm ci
    if errorlevel 1 (
        echo ERROR: npm dependency installation failed.
        exit /b 1
    )
)

echo Building the frontend and Windows application...
call npm run tauri -- build
if errorlevel 1 (
    echo ERROR: Tauri release build failed.
    exit /b 1
)

set "RELEASE_EXE=src-tauri\target\release\csv-compare.exe"
set "PORTABLE_EXE=portable\CSVCompare.exe"

if not exist "%RELEASE_EXE%" (
    echo ERROR: The release executable was not generated:
    echo        %RELEASE_EXE%
    exit /b 1
)

if not exist "portable" mkdir "portable"
copy /y "%RELEASE_EXE%" "%PORTABLE_EXE%" >nul
if errorlevel 1 (
    echo WARNING: Could not update %PORTABLE_EXE% because the file is in use.
    echo          Close the running application and run this script again.
)

echo.
echo BUILD SUCCEEDED
echo Executable: %RELEASE_EXE%
if exist "%PORTABLE_EXE%" (
    echo Portable copy: %PORTABLE_EXE%
)
echo Installers: src-tauri\target\release\bundle
exit /b 0
