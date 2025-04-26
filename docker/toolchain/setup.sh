#!/bin/bash
set -e

# Install dependencies for cross-compilation
apt-get update && apt-get install -y \
    gcc-x86_64-linux-gnu \
    libc6-dev-amd64-cross \
    gcc-mingw-w64-x86-64 \
    clang \
    cmake \
    patch \
    libxml2-dev \
    wget \
    xz-utils \
    curl \
    git-lfs \
    awscli \
    && rm -rf /var/lib/apt/lists/*

# Install macOS cross-compilation tools (osxcross)
git config --global --add safe.directory '*'
git clone https://github.com/tpoechtrager/osxcross /root/osxcross
cd /root/osxcross
wget -nc https://github.com/phracker/MacOSX-SDKs/releases/download/11.3/MacOSX11.3.sdk.tar.xz
mv MacOSX11.3.sdk.tar.xz tarballs/
UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh
echo 'export PATH="/root/osxcross/target/bin:$PATH"' >> ~/.bashrc

# Install Rust targets
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Configure Cargo for cross-compilation
mkdir -p /root/.cargo
cat > /root/.cargo/config.toml << 'EOF'
[target.x86_64-unknown-linux-gnu]
linker = "x86_64-linux-gnu-gcc"

[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"

[target.x86_64-apple-darwin]
linker = "x86_64-apple-darwin20.4-clang"
ar = "x86_64-apple-darwin20.4-ar"

[target.aarch64-apple-darwin]
linker = "aarch64-apple-darwin20.4-clang"
ar = "aarch64-apple-darwin20.4-ar"
EOF