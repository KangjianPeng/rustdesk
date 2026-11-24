#!/usr/bin/env bash
set -euo pipefail

log=$(mktemp)
trap 'rm -f "$log"' EXIT

if "$@" 2>&1 | tee "$log"; then
    exit 0
else
    status=$?
fi

# Dioxus does not emit Actions annotations for Cargo or Gradle errors.
diagnostics=$(sed -E $'s/\x1b\\[[0-9;]*[A-Za-z]//g' "$log" |
    awk 'tolower($0) ~ /error|failed|undefined symbol|fatal|could not/ { print substr($0, 1, 400) }' |
    tail -n 12)
diagnostics=${diagnostics:-Installer command failed with exit code $status. See the job log.}
diagnostics=${diagnostics//%/%25}
diagnostics=${diagnostics//$'\r'/%0D}
diagnostics=${diagnostics//$'\n'/%0A}
printf '::error title=Installer build failed::%s\n' "$diagnostics"
exit "$status"
