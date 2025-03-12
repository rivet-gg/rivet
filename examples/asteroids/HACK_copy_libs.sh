#\!/bin/bash
set -e

# Get the current directory
CURRENT_DIR=$(pwd)

# Create vendor directory if it doesn't exist
mkdir -p vendor

# The specific dependencies to vendor
DEPS=(
  "actor-core-lobby-manager:../../../../actor-core/packages/components/lobby-manager/"
  "actor-core-rivet:../../../../actor-core/packages/platforms/rivet/"
  "actor-core:../../../../actor-core/packages/actor-core/"
  "actor-core-cli:../../../../actor-core/packages/actor-core-cli/"
)

for DEP_INFO in "${DEPS[@]}"; do
  # Split the string by colon
  DEP="${DEP_INFO%%:*}"
  DEP_PATH="${DEP_INFO#*:}"
  
  echo "Processing $DEP from $DEP_PATH..."
  
  # cd to the dependency path
  cd "$CURRENT_DIR/$DEP_PATH"
  
  # Run yarn pack to a temporary file in the current directory
  yarn pack --out "$CURRENT_DIR/vendor/$DEP.tgz"
  
  # Go back to original directory
  cd "$CURRENT_DIR"
done

yarn

echo "Done! All dependencies have been packed to the vendor/ directory."
