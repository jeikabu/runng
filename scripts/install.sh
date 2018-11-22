#!/usr/bin/env bash

if [[ "$TRAVIS_OS_NAME" == "osx" ]]; then
    # `brew install ninja` requires `brew update` which takes ages....
    wget https://github.com/ninja-build/ninja/releases/download/v1.8.2/ninja-mac.zip
    unzip ninja-mac.zip
    
    export PATH=`pwd`:$PATH
fi
