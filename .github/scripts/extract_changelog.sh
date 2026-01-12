#!/bin/bash
# Extract changelog section for a specific version

set -e

VERSION=$1
CHANGELOG_FILE=${2:-CHANGELOG.md}

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version> [changelog_file]"
    echo "Example: $0 2.1.0"
    exit 1
fi

# Remove 'v' prefix if present
VERSION=${VERSION#v}

# Extract the section for this version
# This finds the line starting with ## [VERSION] and prints until the next ## [
awk "/## \[$VERSION\]/,/## \[/" "$CHANGELOG_FILE" | sed '$d' | tail -n +2

exit 0
