#!/usr/bin/env bash
set -euo pipefail

: "${VERSION:?VERSION must be set}"

./build.py --macos-app "$@"

app="target/dx/rustdesk/bundle/macos/Rustdesk.app"
if [[ ! -d "$app" ]]; then
  printf 'Dioxus macOS bundle was not found: %s\n' "$app" >&2
  exit 1
fi

if [[ -n "${MACOS_CODESIGN_IDENTITY:-}" ]]; then
  codesign --force --options runtime -s "$MACOS_CODESIGN_IDENTITY" --deep --strict "$app" -vvv
fi

dmg="rustdesk-${VERSION}.dmg"
create-dmg --icon "Rustdesk.app" 200 190 --hide-extension "Rustdesk.app" --window-size 800 400 --app-drop-link 600 185 "$dmg" "$app"

if [[ -n "${MACOS_CODESIGN_IDENTITY:-}" ]]; then
  codesign --force --options runtime -s "$MACOS_CODESIGN_IDENTITY" --deep --strict "$dmg" -vvv
  if [[ -f "${MACOS_NOTARIZE_API_KEY:-$HOME/.p12/api-key.json}" ]]; then
    rcodesign notary-submit --api-key-path "${MACOS_NOTARIZE_API_KEY:-$HOME/.p12/api-key.json}" --staple "$dmg"
  fi
fi
