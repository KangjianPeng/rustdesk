#!/bin/sh

cd "$HOME"/rustdesk || exit 1
# shellcheck source=/dev/null
. "$HOME"/.cargo/env

target_next=0
for arg in "$@"; do
  if [ "$target_next" = 1 ]; then
    rustup target add "$arg" || exit 1
    target_next=0
  elif [ "$arg" = --target ]; then
    target_next=1
  fi
done

exec cargo build --locked "$@"
