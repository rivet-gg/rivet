# Start from the base image
FROM mcr.microsoft.com/devcontainers/base:bullseye

# Install Git
RUN apt-get update && apt-get install -y git

# Clone the Rivet repository
RUN git clone https://github.com/rivet-gg/rivet.git

# # Boot the cluster
# RUN nix-shell --run "bolt init dev --yes"

# # Promote the user to admin
# RUN nix-shell --run "bolt db sh db-user --query 'UPDATE users SET is_admin = true'"