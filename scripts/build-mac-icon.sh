#!/bin/sh
# Compiles src-tauri/icons/Icon.icon (Icon Composer, macOS 26) into src-tauri/icons/Assets.car,
# which tauri.macos.conf.json copies into the app. Run it after changing Icon.icon and commit
# the result. It's precompiled because actool crashes on GitHub's macOS runners.
# Needs Xcode 26+.
set -eu
cd "$(dirname "$0")/../src-tauri/icons"
out=$(mktemp -d)
xcrun actool Icon.icon --compile "$out" --output-format human-readable-text --notices --warnings \
  --output-partial-info-plist "$out/info.plist" --app-icon Icon --include-all-app-icons \
  --enable-on-demand-resources NO --target-device mac --minimum-deployment-target 26.0 --platform macosx
cp "$out/Assets.car" Assets.car
rm -rf "$out"
echo "wrote src-tauri/icons/Assets.car"
