#!/usr/bin/env bash

TARBALL="v36.tar.gz"
KCOV_DIR="kcov-36"

if [[ "$TRAVIS_OS_NAME" == "linux" ]] || [[ "$OSTYPE" == "linux-gnu" ]]; then
    # https://github.com/codecov/example-rust
    wget https://github.com/SimonKagstrom/kcov/archive/$TARBALL
    tar xzf $TARBALL
    cd $KCOV_DIR
    mkdir build
    cd build
    cmake ..
    make
    make install DESTDIR=../../kcov-build
    cd ../../
    # rm -rf $KCOV_DIR
    for file in target/debug/*-*[^\.d]; do 
        mkdir -p "target/cov/$(basename $file)"
        # Arguments at the end are what would be passed to `cargo test`
        ./kcov-build/usr/local/bin/kcov --exclude-pattern=/.cargo,/usr/lib --verify "target/cov/$(basename $file)" "$file" -- "tests::"
    done
    # Upload reports in current directory
    # https://github.com/SimonKagstrom/kcov/blob/master/doc/codecov.md
    bash <(curl -s https://codecov.io/bash)
fi