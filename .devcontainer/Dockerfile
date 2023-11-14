# Start from the Nix image
FROM nixos/nix as builder

# Install Git
RUN nix-env -iA nixpkgs.git

# Clone the Rivet repository
RUN git clone https://github.com/rivet-gg/rivet.git

# Change into the repo directory
WORKDIR /rivet

# Make sure bolt is set up. We're just using --help to make sure that bolt is
# set up.
RUN nix-shell --run "bolt --version"

FROM mcr.microsoft.com/devcontainers/base:bullseye

# Copy the nix cache from the builder stage
COPY --from=builder /nix /nix

# Set the working directory
WORKDIR /workspace
