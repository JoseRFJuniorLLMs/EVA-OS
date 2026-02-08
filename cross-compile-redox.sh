#!/bin/bash
# Cross-compile EVA-OS NPU driver for Redox OS
# Run this inside WSL (Ubuntu)

set -e

echo "=== EVA-OS Cross-Compilation for Redox OS ==="
echo ""

# Step 1: Install Rust if not present
if ! command -v rustup &> /dev/null; then
    echo "[1/5] Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "[1/5] Rust already installed"
    source "$HOME/.cargo/env"
fi

# Step 2: Install build dependencies
echo "[2/5] Installing build dependencies..."
sudo apt-get update -qq
sudo apt-get install -y -qq build-essential pkg-config gcc-multilib

# Step 3: Add Redox target
echo "[3/5] Adding x86_64-unknown-redox target..."
rustup target add x86_64-unknown-redox 2>/dev/null || true

# Step 4: Install Redox cross-compiler toolchain
echo "[4/5] Setting up Redox cross-compiler..."
if ! command -v x86_64-unknown-redox-gcc &> /dev/null; then
    # Install the Redox toolchain
    # Option A: Use relibc-install (official method)
    if [ ! -d "$HOME/.redox" ]; then
        mkdir -p "$HOME/.redox"
    fi

    # Download pre-built Redox toolchain
    REDOX_TOOLCHAIN="$HOME/.redox/toolchain"
    if [ ! -d "$REDOX_TOOLCHAIN" ]; then
        echo "Downloading Redox toolchain..."
        git clone --depth 1 https://gitlab.redox-os.org/redox-os/redox.git /tmp/redox-build
        cd /tmp/redox-build
        # Use the bootstrap script to get the toolchain
        ./bootstrap.sh -d
        make prefix
        cd -
    fi

    export PATH="$REDOX_TOOLCHAIN/bin:$PATH"
fi

# Step 5: Configure Cargo for Redox target
echo "[5/5] Configuring Cargo..."
DRIVER_DIR="/mnt/d/DEV/EVA-OS/drive"
mkdir -p "$DRIVER_DIR/.cargo"

cat > "$DRIVER_DIR/.cargo/config.toml" << 'CARGO_CONFIG'
[target.x86_64-unknown-redox]
linker = "x86_64-unknown-redox-gcc"
ar = "x86_64-unknown-redox-ar"
CARGO_CONFIG

# Build the driver
echo ""
echo "=== Building intel-npu driver for Redox ==="
cd "$DRIVER_DIR"
cargo build --target x86_64-unknown-redox --release 2>&1

echo ""
echo "=== Building C API library for Redox ==="
cd "/mnt/d/DEV/EVA-OS/driver-c-api"
mkdir -p .cargo
cp "$DRIVER_DIR/.cargo/config.toml" .cargo/config.toml
cargo build --target x86_64-unknown-redox --release --lib 2>&1

echo ""
echo "=== BUILD COMPLETE ==="
echo "Driver binary: $DRIVER_DIR/target/x86_64-unknown-redox/release/intel-npu"
echo "C API lib:     /mnt/d/DEV/EVA-OS/driver-c-api/target/x86_64-unknown-redox/release/libeva_npu.a"
echo ""
echo "Next: Copy these to the Redox VM disk image"
