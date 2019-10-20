#!/usr/bin/env pwsh

if ($IsWindows) {
} elseif ($IsMacOS) {
    $env:PATH += [IO.Path]::PathSeparator + "$env:HOME/.cargo/bin"
} else {
}

cargo fmt --all -- --check
cargo clippy
$env:RUST_BACKTRACE = 1
cargo test
