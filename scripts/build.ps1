#!/usr/bin/env pwsh

if ($IsWindows) {
    cargo test
} elseif ($IsMacOS) {
    $env:PATH += [IO.Path]::PathSeparator + "$env:HOME/.cargo/bin"
    cargo test
} else {
    cargo test
}

