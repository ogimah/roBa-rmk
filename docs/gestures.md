# Layer 3 + V trackball gestures

This branch starts from the user's working unlock build,
`24dbb3cfb997b61dc21640dd33017665412647b6`. Main, dependency versions,
Rust 1.96.0, physical wiring, Bluetooth settings, unlock keys and encoder
assignments are unchanged. The earlier incomplete gesture scaffold is not used.

## Behavior

- Layer 3 alone uses the normal cursor; its former automatic scroll mode is removed.
- Hold the **physical V position** (matrix row 2, column 3) while layer 3 is active
  to capture trackball motion. Other layers keep their previous key actions.
- Accumulate signed X/Y sensor counts. Fire as soon as `abs(x) + abs(y) > 30`.
- Use the larger absolute axis; horizontal wins ties. Logical +Y is down.
  Existing PMW3610 sensor inversion is retained, not applied twice.
- Fire immediately, without waiting for V release. Reset the accumulator after
  each threshold crossing. Crossings within 150ms of the last output are
  discarded. Keep holding V to perform further strokes at least 150ms apart.
- Captured movement never goes to the normal pointing processor, including
  subthreshold movement and movement during cooldown. Release V or leave layer 3
  to cancel unfinished motion; release itself does not produce an output.

## Vial: four actual editable output slots

The keymap's logical dimensions increase from 4x11 to 5x11; the physical split
matrices remain 4x5 and 4x6. Existing row/column coordinates and eight layer
indices are unchanged. The new row is virtual, not additional GPIO wiring.

Below the original keyboard Vial displays four extra editable keys in a cross:

```text
               Up (4,2)
Left (4,0)                 Right (4,1)
              Down (4,3)
```

Select **layer 3** in Vial and assign these four slots. They start as `KC_NO`;
an unassigned slot deliberately sends nothing. Ordinary keys, modified keys
and RMK-supported Vial macros are supported and covered by host integration
tests. Timing-dependent tap-hold/tap-dance actions are not validated as gesture
outputs. The gesture sends an immediate press/release, not a held action.

No special OS application or fixed F13-F16 output is required. Vial's current
binding is read by the normal RMK keymap path on every gesture. No direction
User keycodes are added: in this revision User9 is reserved for clearing BLE
split peers and must not be treated as a gesture key.

## Restore an existing .vil

Keep the working firmware and original `.vil` as backups. The new keymap has a
different storage shape; do not rely on the old on-device keymap carrying over.

The manual procedure (no Python required):

1. Flash the matching right/left UF2s and reconnect the right half over USB.
2. Reconnect Vial and load the old `.vil`. If requested, unlock with the right-hand
   top-row endpoints (default Y and P positions).
3. Select layer 3 and set the **physical V key to `KC_NO`** (Any keycode entry
   accepts `KC_NO`). This prevents V text; the firmware still sees the physical
   press. Do this after restoring an old layout, which may restore transparency
   or another action at that position.
4. On layer 3 assign the four new direction slots, then export a new `.vil`.

Optional automatic migration makes a separate copy and preserves the original:

```sh
python scripts/migrate_gesture_vil.py saved-layout.vil
```

It adds the virtual row and changes only layer-3 V to `KC_NO`. It retains existing
keys, encoder mappings, macros and settings, validates the keyboard UID, and
refuses to overwrite existing files. Import the resulting `*-gesture.vil`, assign
the four direction slots and save it. Already assigned gesture slots are retained
if this migration is run on a five-row layout.

The official Vial GUI restore routine was inspected and exercised with a mock
transport: old coordinates can be restored into the expanded layout. Client
version compatibility and the user's actual `.vil` still require hardware checks.

## Implementation and checks

- `src/gesture.rs`: deterministic trigger/capture/threshold/cooldown state.
- `src/pointingproccontroller.rs`: separate trigger monitor plus the sole pointing
  subscriber/router. Separating the monitor avoids a self-deadlock when synthetic
  key publication is backpressured by the keyboard channel.
- `src/central.rs`: RMK's existing `Overwritten(entry)` hook retains every generated
  device and background task, replacing only the pointing processor runnable.
  All core tasks are awaited; a source guard and `deny(unused_must_use)` protect
  against accidentally dropping their futures.
- Keyboard channel capacity is extended to three publishers and one additional
  subscriber for the synthetic output and trigger monitor. Existing physical
  matrices, keymaps and aliases are checked against the baseline in tests.

```sh
python3 -m unittest discover -s tests -p 'test_*.py' -v
cargo test --locked --manifest-path tests/gesture-host/Cargo.toml \
  --target x86_64-unknown-linux-gnu -- --test-threads=1
cargo make uf2
```

The host suite uses the actual pinned RMK processor, keyboard, live host remapping,
macro parser and HID channels, with a mock clock instead of nRF hardware. It tests
all directions, live changes, modified keys/macros and releases, layer changes,
capture vs cursor, and concurrent runnable tasks under keyboard backpressure.

The workflow builds on pushes only to `feat/gesture-vial-live` and on pull requests
or manual dispatch. It does not publish a release or modify main. Host tests and
firmware builds are separate from hardware acceptance.

Hardware acceptance is still required: USB/BLE input, Vial slot visibility and
save/reload, the four assigned outputs, physical sensitivity/directions, no cursor
leakage, normal motion without V, encoder operation and no held keys. A UF2 copy
error must be investigated as a transfer failure: compare CURRENT.UF2 with the
intended image before blaming firmware behavior. Do not format the bootloader
drive or erase pairings as part of gesture testing.
