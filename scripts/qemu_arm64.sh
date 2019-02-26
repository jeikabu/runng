#!/usr/bin/env bash

if [[ "$TRAVIS_OS_NAME" == "linux" ]] || [[ "$OSTYPE" == "linux-gnu" ]]; then
    docker run --rm --privileged multiarch/qemu-user-static:register --reset
    docker run -t -v $(pwd):/usr/src jeikabu/debian-rust:arm64v8-stretch-1.32.0 /bin/bash -c "cd /usr/src; cargo test"
fi