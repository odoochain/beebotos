#!/bin/bash
set -e

echo "Installing WASM tools for BeeBotOS..."

# Detect OS
OS=$(uname -s)

echo "Detected: $OS"

# Install wasmtime
if ! command -v wasmtime &> /dev/null; then
    echo "Installing wasmtime..."
    curl https://wasmtime.dev/install.sh -sSf | bash
fi

# Install wasmer
if ! command -v wasmer &> /dev/null; then
    echo "Installing wasmer..."
    curl https://get.wasmer.io -sSfL | sh
fi

# Install wasm-opt from binaryen
if ! command -v wasm-opt &> /dev/null; then
    echo "Installing binaryen (wasm-opt)..."
    if [ "$OS" == "Darwin" ]; then
        brew install binaryen
    else
        # Linux
        BINARYEN_VERSION=version_116
        wget https://github.com/WebAssembly/binaryen/releases/download/$BINARYEN_VERSION/binaryen-$BINARYEN_VERSION-x86_64-linux.tar.gz
        tar -xzf binaryen-$BINARYEN_VERSION-x86_64-linux.tar.gz
        sudo mv binaryen-$BINARYEN_VERSION/bin/* /usr/local/bin/
        rm -rf binaryen-$BINARYEN_VERSION*
    fi
fi

# Install wabt tools
if ! command -v wasm2wat &> /dev/null; then
    echo "Installing wabt..."
    if [ "$OS" == "Darwin" ]; then
        brew install wabt
    else
        # Linux
        WABT_VERSION=1.0.34
        wget https://github.com/WebAssembly/wabt/releases/download/$WABT_VERSION/wabt-$WABT_VERSION-ubuntu.tar.gz
        tar -xzf wabt-$WABT_VERSION-ubuntu.tar.gz
        sudo mv wabt-$WABT_VERSION/bin/* /usr/local/bin/
        rm -rf wabt-$WABT_VERSION*
    fi
fi

# Verify installation
echo "Verifying installation..."
wasmtime --version
wasmer --version
wasm-opt --version
wasm2wat --version

echo "WASM tools installation complete!"
