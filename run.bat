@echo off
REM ----------------------------------------------------------------------
REM  CMOS -- one-click build & launch
REM
REM  Checks and installs all dependencies:
REM    0. Visual Studio Build Tools (MSVC linker + Windows SDK)
REM    1. Rust (via rustup)
REM    2. Node.js (via winget)
REM    3. pnpm (via npm)
REM    4. Tauri CLI (via cargo)
REM    5. Frontend deps (pnpm install)
REM  Then builds and launches the app.
REM
REM  Usage:
REM    run.bat          -- interactive mode selection
REM    run.bat debug    -- debug build
REM    run.bat release  -- release build
REM    run.bat dev      -- dev mode (hot reload)
REM ----------------------------------------------------------------------

setlocal enabledelayedexpansion
cd /d "%~dp0"

echo.
echo ============================================================
echo   CMOS - Cognitive Memory Operating System
echo ============================================================
echo.

REM === Ensure cargo is in PATH (common issue on fresh shells) ===
if exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)

REM === STEP 0: Check Visual Studio Build Tools (MSVC linker) ===
set "VCVARS="
call :find_vcvars
if not defined VCVARS (
    echo [setup] Visual Studio Build Tools not found.
    echo [setup] Installing via winget - this may take 5-10 minutes...
    echo.

    winget install Microsoft.VisualStudio.2022.BuildTools --accept-package-agreements --accept-source-agreements --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

    call :find_vcvars
    if not defined VCVARS (
        echo [setup] ERROR: VS Build Tools installation failed.
        echo [setup] Please install manually: https://visualstudio.microsoft.com/visual-cpp-build-tools/
        echo [setup] Select "Desktop development with C++" workload.
        goto :fail
    )

    echo [setup] VS Build Tools installed successfully.
    echo.
) else (
    echo [check] VS Build Tools found
)

REM Load MSVC environment
echo [setup] Loading MSVC environment...
call "%VCVARS%" x64 >nul 2>&1

REM === STEP 0.5: Check WebView2 Runtime ===
set "WV2_FOUND=0"
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BEE-13A6279FE7FF}" /v pv >nul 2>&1
if not errorlevel 1 set "WV2_FOUND=1"
if "%WV2_FOUND%"=="0" (
    reg query "HKLM\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BEE-13A6279FE7FF}" /v pv >nul 2>&1
    if not errorlevel 1 set "WV2_FOUND=1"
)
if "%WV2_FOUND%"=="0" (
    echo [setup] WebView2 Runtime not found. Installing...
    echo.

    powershell -Command "Invoke-WebRequest -Uri 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile '%TEMP%\MicrosoftEdgeWebview2Setup.exe'"
    if errorlevel 1 (
        echo [setup] ERROR: Failed to download WebView2 bootstrapper.
        echo [setup] Please install manually: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
        goto :fail
    )

    "%TEMP%\MicrosoftEdgeWebview2Setup.exe" /silent /install
    if errorlevel 1 (
        echo [setup] ERROR: WebView2 installation failed.
        echo [setup] Please install manually: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
        goto :fail
    )

    del /q "%TEMP%\MicrosoftEdgeWebview2Setup.exe" >nul 2>&1
    echo [setup] WebView2 Runtime installed successfully.
    echo.
) else (
    echo [check] WebView2 Runtime found
)

REM === STEP 1: Check Rust ===
where rustc >nul 2>&1
if errorlevel 1 (
    echo [setup] Rust not found. Installing via rustup...
    echo.

    powershell -Command "Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile '%TEMP%\rustup-init.exe'"
    if errorlevel 1 (
        echo [setup] ERROR: Failed to download rustup-init.exe
        echo [setup] Please install Rust manually: https://rustup.rs
        goto :fail
    )

    "%TEMP%\rustup-init.exe" -y --default-toolchain stable
    if errorlevel 1 (
        echo [setup] ERROR: Rust installation failed.
        goto :fail
    )

    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
    echo [setup] Rust installed successfully.
    echo.
) else (
    for /f "tokens=*" %%v in ('rustc --version') do echo [check] %%v
)

REM === STEP 2: Check Node.js ===
where node >nul 2>&1
if errorlevel 1 (
    echo [setup] Node.js not found. Installing via winget...
    echo.

    winget install OpenJS.NodeJS.LTS --accept-package-agreements --accept-source-agreements
    if errorlevel 1 (
        echo [setup] ERROR: Node.js installation failed.
        echo [setup] Please install Node.js manually: https://nodejs.org
        goto :fail
    )

    for /f "tokens=*" %%p in ('where node 2^>nul') do set "NODEPATH=%%~dpp"
    if defined NODEPATH set "PATH=!NODEPATH!;%PATH%"

    echo [setup] Node.js installed successfully.
    echo.
) else (
    for /f "tokens=*" %%v in ('node --version') do echo [check] Node.js %%v
)

REM === STEP 3: Check pnpm ===
where pnpm >nul 2>&1
if errorlevel 1 (
    echo [setup] pnpm not found. Installing via npm...
    echo.

    call npm install -g pnpm
    if errorlevel 1 (
        echo [setup] ERROR: pnpm installation failed.
        goto :fail
    )

    echo [setup] pnpm installed successfully.
    echo.
) else (
    for /f "tokens=*" %%v in ('pnpm --version') do echo [check] pnpm %%v
)

REM === STEP 4: Check Tauri CLI ===
where cargo-tauri >nul 2>&1
if errorlevel 1 (
    echo [setup] Tauri CLI not found. Installing via cargo...
    echo.

    cargo install tauri-cli --version "^2"
    if errorlevel 1 (
        echo [setup] ERROR: Tauri CLI installation failed.
        goto :fail
    )

    echo [setup] Tauri CLI installed successfully.
    echo.
) else (
    echo [check] Tauri CLI installed
)

REM === STEP 5: Install frontend dependencies ===
echo [setup] Syncing frontend dependencies...
pushd apps\desktop
call pnpm install --frozen-lockfile >nul 2>&1
if errorlevel 1 (
    echo [setup] Lockfile out of date, running full install...
    call pnpm install
    if errorlevel 1 (
        echo [setup] ERROR: Frontend dependency installation failed.
        popd
        goto :fail
    )
)
popd
echo [check] Frontend dependencies synced

echo.
echo ============================================================
echo   All dependencies OK. Ready to build.
echo ============================================================
echo.

REM === Portable data: store all CMOS data inside the project folder ===
set "CMOS_DATA_DIR=%~dp0data"
if not exist "%CMOS_DATA_DIR%" mkdir "%CMOS_DATA_DIR%"

REM === BUILD MODE SELECTION ===
set "MODE=%~1"
if /I "%MODE%"=="release" goto :mode_release
if /I "%MODE%"=="debug"   goto :mode_debug
if /I "%MODE%"=="dev"     goto :mode_dev

:ask
echo Select mode:
echo     [1] Dev     (hot reload, fastest iteration)
echo     [2] Debug   (full build, debug symbols)
echo     [3] Release (optimized build)
set "CHOICE="
set /p "CHOICE=Enter 1, 2, or 3 (default 1): "
if not defined CHOICE set "CHOICE=1"
if "%CHOICE%"=="1" goto :mode_dev
if "%CHOICE%"=="2" goto :mode_debug
if "%CHOICE%"=="3" goto :mode_release
echo Invalid choice - try again.
goto :ask

:mode_dev
echo [build] Starting in dev mode (hot reload)...
echo.
pushd apps\desktop
cargo tauri dev
popd
goto :done

:mode_debug
echo [build] Building debug...
echo.
pushd apps\desktop
cargo tauri build --debug
popd
if errorlevel 1 (
    echo.
    echo [build] BUILD FAILED.
    goto :fail
)
echo.
echo [build] Debug build complete.
set "EXE=%~dp0target\debug\cmos-desktop.exe"
if exist "%EXE%" (
    echo [run] Launching %EXE%
    start "" "%EXE%"
)
goto :done

:mode_release
echo [build] Building release (this may take a while)...
echo.
pushd apps\desktop
cargo tauri build
popd
if errorlevel 1 (
    echo.
    echo [build] BUILD FAILED.
    goto :fail
)
echo.
echo [build] Release build complete.
set "EXE=%~dp0target\release\cmos-desktop.exe"
if exist "%EXE%" (
    echo [run] Launching %EXE%
    start "" "%EXE%"
) else (
    echo [build] Installer available in target\release\bundle\
)
goto :done

:done
echo.
echo [done] Finished.
goto :end

:fail
echo.
echo [error] Something went wrong. See messages above.
goto :end

:end
endlocal
pause
exit /b

REM === SUBROUTINE: Find vcvarsall.bat ===
:find_vcvars
set "VCVARS="
REM Check BuildTools 2022
if exist "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat" (
    set "VCVARS=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat"
    goto :eof
)
REM Check Community 2022
if exist "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat" (
    set "VCVARS=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat"
    goto :eof
)
REM Check Professional 2022
if exist "C:\Program Files (x86)\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvarsall.bat" (
    set "VCVARS=C:\Program Files (x86)\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvarsall.bat"
    goto :eof
)
REM Check Enterprise 2022
if exist "C:\Program Files (x86)\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvarsall.bat" (
    set "VCVARS=C:\Program Files (x86)\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvarsall.bat"
    goto :eof
)
REM Try vswhere as last resort
set "VSWHERE=C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe"
if exist "%VSWHERE%" (
    for /f "tokens=*" %%p in ('"%VSWHERE%" -latest -products * -property installationPath 2^>nul') do (
        if exist "%%p\VC\Auxiliary\Build\vcvarsall.bat" (
            set "VCVARS=%%p\VC\Auxiliary\Build\vcvarsall.bat"
        )
    )
)
goto :eof
