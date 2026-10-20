#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# SCRAWL_APP_DIR builds the signed bundle somewhere else (staging for remote installs, see
# install-app-remote.sh); SCRAWL_NO_LAUNCH=1 skips the ~/bin link and the restart.
APP_DIR="${SCRAWL_APP_DIR:-${HOME}/Applications/Remark.app}"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"
IDENTITY="-"

cd "${REPO_DIR}"
taskpolicy -c utility cargo build -j 3 --offline

mkdir -p "${MACOS_DIR}" "${RESOURCES_DIR}" "${HOME}/bin"

rm -f "${MACOS_DIR}/scrawl" "${MACOS_DIR}/scrawl-helper"
cp -f "${REPO_DIR}/target/debug/scrawl" "${MACOS_DIR}/scrawl"
cp -f "${REPO_DIR}/target/debug/scrawl-helper" "${MACOS_DIR}/scrawl-helper"
chmod +x "${MACOS_DIR}/scrawl" "${MACOS_DIR}/scrawl-helper"

if [[ -f "${REPO_DIR}/assets/Scrawl.icns" ]]; then
  cp -f "${REPO_DIR}/assets/Scrawl.icns" "${RESOURCES_DIR}/Scrawl.icns"
fi

cat > "${CONTENTS_DIR}/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>scrawl</string>
  <key>CFBundleIconFile</key>
  <string>Scrawl</string>
  <key>CFBundleIdentifier</key>
  <string>dev.altura.remark</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>Remark</string>
  <key>CFBundleDisplayName</key>
  <string>Remark</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.3.6</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSMinimumSystemVersion</key>
  <string>26.0</string>
  <key>LSUIElement</key>
  <true/>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>NSScreenCaptureUsageDescription</key>
  <string>Remark captures your screen to build annotated agent bundles.</string>
  <key>NSMicrophoneUsageDescription</key>
  <string>Remark records your voice to align transcripts with keyframes.</string>
  <key>NSCameraUsageDescription</key>
  <string>Remark mirrors and records a cabled iPhone's screen.</string>
  <key>NSSpeechRecognitionUsageDescription</key>
  <string>Remark transcribes your narration on this Mac.</string>
  <key>NSAppleEventsUsageDescription</key>
  <string>Remark reads the address of the browser tab you captured, to send a capture of a dev site to its project's session.</string>
</dict>
</plist>
PLIST

codesign --force --sign "${IDENTITY}" --identifier "dev.altura.remark.helper" "${MACOS_DIR}/scrawl-helper"
codesign --force --sign "${IDENTITY}" --identifier "dev.altura.remark" "${APP_DIR}"

BROWSER_DIR="${SCRAWL_BROWSER_DIR:-${HOME}/Library/Application Support/remark/browser}"
mkdir -p "${BROWSER_DIR}"
cp "${REPO_DIR}/browser/manifest.json" "${REPO_DIR}/browser/background.js" "${REPO_DIR}/browser/broker.js" "${REPO_DIR}/browser/context.js" "${REPO_DIR}/browser/picker.js" "${BROWSER_DIR}/"
"${MACOS_DIR}/scrawl" connect
"${MACOS_DIR}/scrawl" browser "${BROWSER_DIR}"

if [[ "${SCRAWL_NO_LAUNCH:-0}" == "1" ]]; then
  exit 0
fi

ln -sf "${MACOS_DIR}/scrawl" "${HOME}/bin/remark"

pkill -f "Remark.app/Contents/MacOS/scrawl" 2>/dev/null || true
for _ in {1..20}; do
  pgrep -f "Remark.app/Contents/MacOS/scrawl" >/dev/null || break
  sleep 0.05
done
open -a "${APP_DIR}" || (sleep 0.3 && open -n -a "${APP_DIR}")
sleep 0.3
pgrep -fl "Remark.app/Contents/MacOS/scrawl"
