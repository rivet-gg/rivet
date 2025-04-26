FROM rust:1.82.0

# Install dependencies for cross-compilation
RUN apt-get update && apt-get install -y \
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
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install FoundationDB client
RUN curl -Lf -o /usr/lib/libfdb_c.so "https://github.com/apple/foundationdb/releases/download/7.1.60/libfdb_c.x86_64.so"

# Install macOS cross-compilation tools (osxcross)
RUN git config --global --add safe.directory '*' && \
    git clone https://github.com/tpoechtrager/osxcross /root/osxcross && \
    cd /root/osxcross && \
    wget -nc https://github.com/phracker/MacOSX-SDKs/releases/download/11.3/MacOSX11.3.sdk.tar.xz && \
    mv MacOSX11.3.sdk.tar.xz tarballs/ && \
    UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh && \
    echo 'export PATH="/root/osxcross/target/bin:$PATH"' >> ~/.bashrc

# Add osxcross to PATH
ENV PATH="/root/osxcross/target/bin:$PATH"

# Install Rust targets
RUN rustup target add x86_64-unknown-linux-gnu && \
    rustup target add x86_64-pc-windows-gnu && \
    rustup target add x86_64-apple-darwin && \
    rustup target add aarch64-apple-darwin

# Configure Cargo for cross-compilation
RUN mkdir -p /root/.cargo && \
    echo '\
[target.x86_64-unknown-linux-gnu]\n\
linker = "x86_64-linux-gnu-gcc"\n\
\n\
[target.x86_64-pc-windows-gnu]\n\
linker = "x86_64-w64-mingw32-gcc"\n\
\n\
[target.x86_64-apple-darwin]\n\
linker = "x86_64-apple-darwin20.4-clang"\n\
ar = "x86_64-apple-darwin20.4-ar"\n\
\n\
[target.aarch64-apple-darwin]\n\
linker = "aarch64-apple-darwin20.4-clang"\n\
ar = "aarch64-apple-darwin20.4-ar"\n\
' > /root/.cargo/config.toml

# Set environment variables for cross-compilation
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER=x86_64-apple-darwin20.4-clang \
    CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=aarch64-apple-darwin20.4-clang \
    CC_x86_64_apple_darwin=x86_64-apple-darwin20.4-clang \
    CXX_x86_64_apple_darwin=x86_64-apple-darwin20.4-clang++ \
    CC_aarch64_apple_darwin=aarch64-apple-darwin20.4-clang \
    CXX_aarch64_apple_darwin=aarch64-apple-darwin20.4-clang++ \
    MACOSX_DEPLOYMENT_TARGET=10.7 \
    CARGO_INCREMENTAL=0 \
    RUSTFLAGS="--cfg tokio_unstable" \
    CARGO_NET_GIT_FETCH_WITH_CLI=true

# Set working directory
WORKDIR /build

# Build CLI instructions
COPY . .

# Build for all platforms by default
ARG PLATFORMS="x86_64-unknown-linux-gnu x86_64-pc-windows-gnu x86_64-apple-darwin aarch64-apple-darwin"

# Build the CLI for all platforms
RUN for platform in ${PLATFORMS}; do \
    echo "Building for $platform..." && \
    cargo build --bin rivet --release --target $platform; \
    done

# Create directory for finished binaries
RUN mkdir -p /artifacts && \
    cp target/x86_64-unknown-linux-gnu/release/rivet /artifacts/rivet-x86-linux && \
    cp target/x86_64-pc-windows-gnu/release/rivet.exe /artifacts/rivet-x86-windows.exe && \
    cp target/x86_64-apple-darwin/release/rivet /artifacts/rivet-x86-mac && \
    cp target/aarch64-apple-darwin/release/rivet /artifacts/rivet-aarch64-mac

# Default command to show help
CMD ["ls", "-la", "/artifacts"]