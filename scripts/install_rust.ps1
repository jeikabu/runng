#!/usr/bin/env pwsh

if ($IsWindows) {
    Invoke-WebRequest https://win.rustup.rs/ -OutFile rustup-init.exe
    ./rustup-init.exe -yv --default-toolchain stable --default-host x86_64-pc-windows-msvc
} elseif ($IsMacOS) {
    Invoke-WebRequest https://sh.rustup.rs -OutFile rustup-init.sh
    bash rustup-init.sh -y -v --default-toolchain stable
} else {

}