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
