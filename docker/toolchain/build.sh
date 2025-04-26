#!/bin/bash
set -e

# Default to Linux if no platform specified
PLATFORM=${1:-linux}
ARCH=${2:-}

case $PLATFORM in
  linux)
    echo "Building for Linux platform"
    DOCKERFILE="linux.Dockerfile"
    BINARY="rivet-x86-linux"
    ;;
  macos)
    if [ "$ARCH" == "arm64" ] || [ "$ARCH" == "aarch64" ]; then
      echo "Building for macOS ARM64 platform"
      DOCKERFILE="macos.Dockerfile"
      BINARY="rivet-aarch64-mac"
    else
      echo "Building for macOS x86_64 platform"
      DOCKERFILE="macos.Dockerfile" 
      BINARY="rivet-x86-mac"
    fi
    ;;
  windows)
    echo "Building for Windows platform"
    DOCKERFILE="windows.Dockerfile"
    BINARY="rivet-x86-windows.exe"
    ;;
  *)
    echo "Unsupported platform: $PLATFORM"
    echo "Usage: $0 [linux|macos|windows] [arch]"
    echo "  For macOS, specify 'arm64' as second parameter for ARM64 build"
    exit 1
    ;;
esac

# Build docker image
DOCKER_BUILDKIT=1 docker build -f docker/toolchain/$DOCKERFILE -t rivet-cli-builder-$PLATFORM .

# Extract binary
CONTAINER_ID=$(docker create rivet-cli-builder-$PLATFORM)
mkdir -p dist
docker cp "$CONTAINER_ID:/artifacts/$BINARY" dist/
docker rm "$CONTAINER_ID"

# Make binary executable (skip for Windows .exe files)
if [[ ! "$BINARY" == *.exe ]]; then
  chmod +x dist/$BINARY
fi

echo "Binary saved to: dist/$BINARY"