#!/usr/bin/env pwsh

if ($IsWindows) {
} elseif ($IsMacOS) {
    $env:PATH += [IO.Path]::PathSeparator + "$env:HOME/.cargo/bin"
} else {
}

cargo fmt --all -- --check
cargo clippy
# Enable full callstacks
$env:RUST_BACKTRACE ="full"
# Enable debug logging
$env:RUST_LOG="runng=debug,test_main=debug"
cargo test
