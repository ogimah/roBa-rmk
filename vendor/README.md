# RMK 0.9.0 encoder backport

`rmk/` is the RMK library from stable commit
`986017d901c7609a9a79167686524f014effa68a`, with upstream
[PR #1277](https://github.com/rmk-rs/rmk/pull/1277) applied to
`src/input_device/rotary_encoder.rs`.

This keeps the stable Bluetooth/USB/Vial dependency baseline. It does not
follow upstream main. The root firmware and host-test Cargo manifests both
patch the same git source to this local library, so generated peripheral
encoder initialization uses the corrected driver automatically.

Changes relative to the recorded stable source:

- Encoder source: upstream #1277, plus a test-module declaration.
- Added encoder regression tests simulate GPIO changes during release/retry waits.
- Keyboard mouse dispatch: encoder wheel actions produce one isolated report per
  press, rather than entering the held-wheel repeat state. Encoder releases do
  not modify that state. Normal physical wheel keys retain hold-to-repeat.
- Mouse-state helper preserves current buttons and acceleration, emits no X/Y
  movement and leaves physical-key repeat state untouched.
- Cargo manifest: replace sibling paths with the exact same stable git commit;
  point package README at its local copy.
- Add original upstream license files and a lockfile for standalone library tests.

`rmk-upstream-sha256.json` records all original library file hashes.
`encoder-1277.patch` preserves the original upstream diff for review.
The configuration test checks that all other original source files remain identical.

Run the encoder tests from the firmware root:

```sh
cargo test --locked --manifest-path vendor/rmk/Cargo.toml --no-default-features --features std,log,async_matrix --target x86_64-unknown-linux-gnu --lib input_device::rotary_encoder -- --test-threads=1
cargo test --locked --manifest-path vendor/rmk/Cargo.toml --no-default-features --features std,log,async_matrix --target x86_64-unknown-linux-gnu --lib keyboard::encoder_wheel_tests -- --test-threads=1
```

The missing-release test reproduces endless wheel repeats before the keyboard
dispatch fix and passes after it. This proves the reproduced failure mode is
removed, not where a release is lost on a particular physical keyboard.
Hardware confirmation is still required; arbitrary-speed transition capture
is not guaranteed by these tests.

## Stock-only Vial Key Override port

`rmk-types/` (0.4.0) and `rmk-macro/` (0.8.0) are also vendored from the same
stable commit. Original license files and upstream author metadata are retained.
The paired hash manifests constrain changes to reviewed files.

Vial override support is adapted from Ergohaven's implementation:
https://github.com/ergohaven/rmk/commit/751867859534445c22fd138ea07d9ce72712f4d3
(MIT OR Apache-2.0). Credit: Ergohaven / contributors. The original diff is
recorded in `vial-overrides-75186785.patch`; this is a port to stable 0.9.0,
not a switch to the Ergohaven dependency tree.

Port changes: 10-byte Vial GET/SET + 32-entry query, paired modifier matching,
layer masks, enable/options/suppression, live fork updates, mid-hold swaps,
press-time source-layer lookup, native TOML kept-modifier conversion. Existing
0.9.0 `KeyboardContext::set_fork` and storage worker supply flash persistence.
No third-party dependency versions change in any of the three lockfiles.

Additional corrections found by USB report acceptance tests:
- An inactive/negative override must not suppress its configured modifiers.
- On trigger release, publish restored held modifiers immediately after the
  replacement's release report, rather than waiting for a later physical event.

`keyboard/override_tests.rs` is test-only; `host/via/mod.rs` adds only a test-only
re-export. `storage/mod.rs` adds only a real flash-worker/reboot regression test.
The stock sensor, GT/ST implementation, encoder decoder and wheel helper are
unchanged. Override slots initially remain empty.

Run `bash tests/run_override_tests.sh` with cargo-nextest 0.9.152. The focused
acceptance tests use production constants; the broad upstream suite supplies
only auto-mouse test capacity (2 instead of disabled 0), without changing the
production keyboard configuration. This avoids the upstream auto-mouse tests'
zero-capacity return vectors; it does not add auto-mouse firmware behavior.

## MB1 drag-lock trial

This stock-only trial adds User14 (DD_ON / DD+) and User15 (DD_OFF / DD-).
`state.rs` owns a volatile latch synchronized with the existing connection lock.
`channel.rs` overlays it on every mouse report at enqueue time, re-sampling it
following backpressure. Raw physical button state stays independent. Both
keyboard and mouse all-up reports are queued on an output/session change.
`keyboard.rs` dispatches the two tap commands and includes the latch in native
mouse-condition fork decisions. Pointing, gesture, and wheel processing remain
unchanged. Normal duplicate-MB1 holder behavior outside DD is not reimplemented.
No code from the draft #1245 with its newer Macro event positions is copied.

See `docs/drag-lock-mb1.md` and `keyboard/drag_tests.rs`. USB/BLE/profile and queue
backpressure tests run with `_ble,split` enabled. The host routing suite also
covers DD with live Vial assignments and all three GT banks plus ST.
