#!/usr/bin/env bash

set -e

cargo build --release 

# Place the compiled library where Neovim can find it.
mkdir -p lua

if [ "$(uname)" == "Darwin" ]; then
    mv target/release/libsilicon_nvim.dylib lua/silicon.so
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
    mv target/release/libsilicon_nvim.so lua/silicon.so
elif [ "$(expr substr $(uname -s) 1 10)" == "MINGW64_NT" ]; then
    mv target/release/silicon_nvim.dll lua/silicon.dll
fi
