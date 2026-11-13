#!/bin/bash
# Xcode build phase: builds the trusted chrome (Bun + Parcel) and copies it
# into the app bundle. Debug builds also get the design-fixture preview
# bundle (ChromePreview) for native screenshot comparisons; Release never does.
set -euo pipefail
export PATH="$HOME/.bun/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"
REPO="$(cd "$SRCROOT/../.." && pwd)"
DEST="$TARGET_BUILD_DIR/$UNLOCALIZED_RESOURCES_FOLDER_PATH"
if ! command -v bun >/dev/null; then
  echo "error: Bun not found (https://bun.sh). It builds the chrome UI from apps/browser-chrome." >&2
  exit 1
fi
cd "$REPO"
bun install --frozen-lockfile >/dev/null
bun run build:ui
mkdir -p "$DEST/Chrome"
rsync -a --delete apps/browser-chrome/dist/app/ "$DEST/Chrome/"
if [ "${CONFIGURATION:-Debug}" = "Debug" ]; then
  bun run build:preview
  rsync -a --delete apps/browser-chrome/dist/preview/ "$DEST/ChromePreview/"
else
  rm -rf "$DEST/ChromePreview"
fi
