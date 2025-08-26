#!/bin/bash
# file: tools/copilot-agent-util-rust/distribute-binaries.sh
# version: 1.1.0
# guid: c3d4e5f6-a7b8-9012-cdef-345678901234

# Distribute cross-platform binaries to all repositories

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_DIR="$SCRIPT_DIR/dist"

# Repository paths (relative to ghcommon)
REPOS=(
    "../../../gcommon"
    "../../../apt-cacher-go"
    "../../../audiobook-organizer"
    "../../../subtitle-manager"
)

if [ ! -d "$DIST_DIR" ]; then
    echo "Error: dist directory not found. Run ./build-cross-platform.sh first."
    exit 1
fi

echo "Distributing copilot-agent-util binaries to repositories..."

for repo in "${REPOS[@]}"; do
    repo_path="$SCRIPT_DIR/$repo"
    bin_dir="$repo_path/bin"

    if [ -d "$repo_path" ]; then
        echo "Updating $repo..."
        mkdir -p "$bin_dir"

        # Copy all platform binaries
        cp "$DIST_DIR/copilot-agent-util-macos-arm64" "$bin_dir/"
        cp "$DIST_DIR/copilot-agent-util-macos-x86_64" "$bin_dir/"
        cp "$DIST_DIR/copilot-agent-util-linux-x86_64" "$bin_dir/"
        cp "$DIST_DIR/copilot-agent-util-linux-arm64" "$bin_dir/"

        # Copy the wrapper script
        cp "$SCRIPT_DIR/copilot-agent-util-wrapper.sh" "$repo_path/copilot-agent-util"

        # Make binaries executable
        chmod +x "$bin_dir"/copilot-agent-util-*
        chmod +x "$repo_path/copilot-agent-util"

        echo "  ✓ Copied binaries to $repo/bin/"
    else
        echo "  ⚠ Repository not found: $repo_path"
    fi
done

echo ""
echo "Distribution complete!"
echo "Each repository now has:"
echo "  bin/copilot-agent-util-macos-arm64"
echo "  bin/copilot-agent-util-macos-x86_64"
echo "  bin/copilot-agent-util-linux-x86_64"
echo "  bin/copilot-agent-util-linux-arm64"
echo "  copilot-agent-util (platform-detecting wrapper)"
echo ""
echo "You can now use './copilot-agent-util' in any repository and it will"
echo "automatically use the correct binary for the current platform."
