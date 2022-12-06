#!/usr/bin/env bash

set -e

# get current version from Cargo.toml
get_version() {
  cat Cargo.toml | grep '^version =' | sed -E 's/.*"([^"]+)".*/\1/'
}

# download the silicon.nvim (of the specified version) from Releases
download() {
  echo "Downloading silicon.nvim library: " $1
  if [ "$(uname)" == "Darwin" ]; then
    arch_name="$(uname -m)"
    curl -fsSL https://github.com/krivahtoo/silicon.nvim/releases/download/$1/silicon-mac-${arch_name}.tar.gz | tar -xz
  elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
    curl -fsSL https://github.com/krivahtoo/silicon.nvim/releases/download/$1/silicon-linux.tar.gz | tar -xz
  elif [ "$(expr substr $(uname -s) 1 10)" == "MINGW64_NT" ]; then
    curl -fsSL https://github.com/krivahtoo/silicon.nvim/releases/download/$1/silicon-win.zip --output silicon-win.zip
    unzip silicon-win.zip
  fi
}

case "$1" in
  build)
    echo "Building silicon.nvim from source..."

    cargo build --release --target-dir ./target

    # Place the compiled library where Neovim can find it.
    mkdir -p lua

    if [ "$(uname)" == "Darwin" ]; then
        mv target/release/libsilicon.dylib lua/silicon.so
    elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
        mv target/release/libsilicon.so lua/silicon.so
    elif [ "$(expr substr $(uname -s) 1 10)" == "MINGW64_NT" ]; then
        mv target/release/silicon.dll lua/silicon.dll
    fi
    ;;
  *)
    version="v$(get_version)"
    download $version

    ;;
esac
