#!/bin/bash
set -e

# Both platform and arch are required
if [ "$#" -lt 2 ]; then
  echo "Usage: $0 <platform> <arch>"
  echo "  platform: linux, mac, windows"
  echo "  arch: x86, aarch64"
  exit 1
fi

PLATFORM="$1"
ARCH="$2"

# Determine dockerfile and binary name based on platform and arch
case "$PLATFORM-$ARCH" in
  linux-x86)
    echo "Building for Linux x86_64 platform"
    DOCKERFILE="linux-x86.Dockerfile"
    BINARY="rivet-x86-linux"
    ;;
  linux-aarch64)
    echo "Building for Linux ARM64 platform"
    DOCKERFILE="linux-aarch64.Dockerfile" 
    BINARY="rivet-aarch64-linux"
    ;;
  mac-x86)
    echo "Building for macOS x86_64 platform"
    DOCKERFILE="macos-x86.Dockerfile" 
    BINARY="rivet-x86-mac"
    ;;
  mac-aarch64)
    echo "Building for macOS ARM64 platform"
    DOCKERFILE="macos-aarch64.Dockerfile"
    BINARY="rivet-aarch64-mac"
    ;;
  windows-x86)
    echo "Building for Windows platform"
    DOCKERFILE="windows-x86.Dockerfile"
    BINARY="rivet-x86-windows.exe"
    ;;
  *)
    echo "Unsupported platform-arch combination: $PLATFORM-$ARCH"
    echo "Usage: $0 <platform> <arch>"
    echo "  platform: linux, mac, windows"
    echo "  arch: x86, aarch64"
    exit 1
    ;;
esac

# Build docker image
DOCKER_BUILDKIT=1 docker build -f docker/toolchain/$DOCKERFILE -t rivet-cli-builder-$PLATFORM-$ARCH .

# Extract binary
CONTAINER_ID=$(docker create rivet-cli-builder-$PLATFORM-$ARCH)
mkdir -p dist
docker cp "$CONTAINER_ID:/artifacts/$BINARY" dist/
docker rm "$CONTAINER_ID"

# Make binary executable (skip for Windows .exe files)
if [[ ! "$BINARY" == *.exe ]]; then
  chmod +x dist/$BINARY
fi

echo "Binary saved to: dist/$BINARY"