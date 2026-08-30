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
# Skip the header line, then print until the next ## [
# Then unwrap hard-wrapped lines: GitHub renders newlines in release bodies
# as hard breaks, so 80-column continuation lines come out jagged.
awk "/## \[$VERSION\]/{found=1; next} found && /## \[/{exit} found" "$CHANGELOG_FILE" | awk '
  function trim(s) { sub(/^[[:space:]]+/, "", s); return s }
  /^```/ { if (buf != "") { print buf; buf = "" }; print; fence = !fence; next }
  fence { print; next }
  /^[[:space:]]*$/ { if (buf != "") { print buf; buf = "" }; print; next }
  /^(#{1,6} |\||[-*] |[0-9]+\. )/ { if (buf != "") { print buf }; buf = $0; next }
  { t = trim($0); buf = (buf == "" ? t : buf " " t) }
  END { if (buf != "") print buf }
'

exit 0
