#!/usr/bin/env bash
set -euo pipefail

log=$(mktemp)
trap 'rm -f "$log"' EXIT

if "$@" 2>&1 | tee "$log"; then
    exit 0
else
    status=$?
fi

# Keep the missing symbols and their callers together in Apple linker errors.
diagnostics=$(sed -E $'s/\x1b\\[[0-9;]*[A-Za-z]//g' "$log" |
    awk '
        /^[[:space:]]*(ld: )?Undefined symbols for architecture / {
            block = $0
            remaining = 11
            next
        }
        /^[[:space:]]*(ld|clang):/ { remaining = 0 }
        remaining > 0 {
            block = block "\n" substr($0, 1, 400)
            remaining--
        }
        END { print block }
    ')
if [[ -z "$diagnostics" ]]; then
    # Dioxus does not emit Actions annotations for Cargo or Gradle errors.
    diagnostics=$(sed -E $'s/\x1b\\[[0-9;]*[A-Za-z]//g' "$log" |
        awk 'tolower($0) ~ /error|failed|undefined symbol|fatal|could not/ { print substr($0, 1, 400) }' |
        tail -n 12)
fi
diagnostics=${diagnostics:-Installer command failed with exit code $status. See the job log.}
diagnostics=${diagnostics//%/%25}
diagnostics=${diagnostics//$'\r'/%0D}
diagnostics=${diagnostics//$'\n'/%0A}
printf '::error title=Installer build failed::%s\n' "$diagnostics"
exit "$status"
