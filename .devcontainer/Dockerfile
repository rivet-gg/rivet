# # Start from the Nix image
# FROM nixos/nix as builder

# # Install Git
# RUN nix-env -iA nixpkgs.git

# # Clone the Rivet repository
# RUN git clone https://github.com/rivet-gg/rivet.git

# # Change into the repo directory
# WORKDIR /rivet

# # Make sure bolt is set up. We're just using --version to make sure that bolt is
# # set up. (For now it's --help since --version isn't on main)
# RUN nix-shell --run "bolt --help"

FROM mcr.microsoft.com/devcontainers/base:bullseye

# # Create the nixbld group and add vscode user to it
# RUN groupadd -r nixbld && usermod -a -G nixbld vscode

# # Copy the nix cache from the builder stage
# COPY --from=builder /nix /nix

# # Change the owner of the /nix directory to vscode
# RUN chown -R vscode:vscode /nix

# # Set the working directory
# WORKDIR /workspace
