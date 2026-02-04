#!/bin/bash
# Setup script for PX4 SITL with the exhibition interface

set -e

echo "UAV Exhibition - PX4 SITL Setup"
echo "================================"

# Check if PX4 directory is provided
PX4_DIR="${1:-../PX4-Autopilot-SurfaceReferential}"

if [ ! -d "$PX4_DIR" ]; then
    echo "Error: PX4 directory not found at $PX4_DIR"
    echo "Usage: $0 [PX4_DIRECTORY]"
    echo ""
    echo "To clone PX4:"
    echo "  git clone https://github.com/aabizri/PX4-Autopilot-SurfaceReferential"
    exit 1
fi

echo "PX4 directory: $PX4_DIR"
cd "$PX4_DIR"

# Check if PX4 is built
if [ ! -f "build/px4_sitl_default/bin/px4" ]; then
    echo "Building PX4 SITL..."
    make px4_sitl
fi

echo ""
echo "Setup complete!"
echo ""
echo "To start PX4 SITL:"
echo "  cd $PX4_DIR"
echo "  make px4_sitl jmavsim"
echo ""
echo "Then start the exhibition server:"
echo "  cd $(dirname $0)"
echo "  cargo run --release"
