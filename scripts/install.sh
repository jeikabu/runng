#!/usr/bin/env bash

if [[ "$TRAVIS_OS_NAME" == "osx" ]]; then
    # `brew install ninja` requires `brew update` which takes ages....
    wget https://github.com/ninja-build/ninja/releases/download/v1.8.2/ninja-mac.zip
    unzip ninja-mac.zip
    
    export PATH=`pwd`:$PATH
fi

if [[ "$TRAVIS_OS_NAME" == "linux" ]]; then
    wget https://cmake.org/files/v3.11/cmake-3.11.4-Linux-x86_64.tar.gz
    tar xzf cmake-3.11.4-Linux-x86_64.tar.gz
    export CMAKE_ROOT=`pwd`cmake-3.11.4-Linux-x86_64/share/cmake-3.11/
    export PATH=`pwd`cmake-3.11.4-Linux-x86_64:$PATH
fi
