#!/usr/bin/env bash
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo"
args=(--locked --manifest-path vendor/rmk/Cargo.toml --no-default-features --features std,log,async_matrix,vial,storage,host_lock --target x86_64-unknown-linux-gnu --lib)
# Acceptance and retained encoder fixes use the exact production constants.
cargo nextest run "${args[@]}" -E 'test(override) | test(test_fork) | test(encoder_wheel_tests) | test(rotary_encoder)'
# Upstream auto-mouse unit tests construct up to two entries, while this
# keyboard disables auto-mouse and resolves its capacity to zero. Give only
# this broad library test run two slots; do not enable it in firmware.
fixture=$(mktemp /tmp/roba-override-library-XXXXXX.toml)
trap 'rm -f "$fixture"' EXIT
python3 - "$repo/keyboard.toml" "$fixture" <<'PY'
import pathlib, sys
source = pathlib.Path(sys.argv[1]).read_text()
pathlib.Path(sys.argv[2]).write_text(source.replace('[rmk]', '[rmk]\nauto_mouse_layer_max_num = 2', 1))
PY
KEYBOARD_TOML_PATH="$fixture" cargo nextest run "${args[@]}"
