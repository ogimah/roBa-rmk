# Personal firmware: roBa_RMK_GT

This branch migrates the working personal GT1/GT2/GT3/ST firmware to the
RMK 0.9.0 stable release, pinned to commit
`986017d901c7609a9a79167686524f014effa68a`.
It does not change main or the public distribution/release branches.

## Preserved behavior

- Existing physical matrix, split wiring, battery configuration and trackball settings.
- Eight layers, personal default keymap, 32 tap-dance entries, macros and encoder assignments.
- GT1/GT2/GT3, their twelve virtual keys, live Vial assignments and 150 ms repeat interval.
- Vertical ST scrolling, with gestures taking priority over ST.
- Vial/Pipette configuration; Rynk is explicitly disabled.
- Vial keyboard UID and virtual-key coordinates, for layout import compatibility.

## Migration changes

- Update Embassy/Bluetooth dependencies to the RMK 0.9 baseline and commit lockfiles.
- trouble-host 0.8.0 includes upstream PR #647: an absent local IRK is no longer distributed as zero.
- Convert `layout.matrix_map` to `layout.map` and move layers to `keymap.layer`.
- Let RMK's BLE transport own the stack and split managers, as required by 0.9.
- Attach the Vial service to USB and BLE transports instead of running a separate host task.
- Use public Vial sessions in host tests after KeyboardContext became private.
- Use distinct device, Vial and UF2 names; preserve USB vendor/product IDs.
- Enable left/peripheral battery ADC on P0.31 using the existing XIAO divider
  (510/1510) and existing active-low P0.14 measurement-enable output. The right
  central continues to measure its own battery. BLE battery user descriptions
  are `Right` and `Left`; the central exposes both readings. The host OS/app
  determines whether both readings are shown in its user interface.
- Backport RMK PR #1277 to the checked-in stable library: retain encoder
  transitions across timed release/retry waits and initialize pin state without
  a phantom step. All other library sources and dependency versions stay fixed.

The firmware and host tests both use `vendor/rmk`. See `vendor/README.md` for
the upstream source manifest, original patch and encoder regression tests.

Encoder wheel mappings now emit one discrete wheel report per input instead
of entering the held-key repeat state. A missing encoder release can no longer
leave wheel auto-repeat running indefinitely. Physical wheel keys still repeat
while held. The current phase/resolution/debounce settings are unchanged.

## Before flashing

1. Export the CURRENT live settings to a `.vil` file. The checked-in personal
   defaults may predate later changes made through Vial/Pipette.
2. Keep the known-working old UF2 pair for rollback.
3. Flash BOTH halves with the matching pair from this branch.
4. For USB testing, connect the central half (right).
5. Import the saved `.vil` if necessary. RMK's existing build-hash check can
   reinitialize stored settings/bonds after a firmware change.
6. Forget the previous Bluetooth device on the host and pair again: RMK 0.9
   changes HID report IDs. This is required independently of the IRK fix.

## Hardware verification still required

- USB typing, personal layout/TD/macros, encoder and both halves.
- Vial/Pipette unlock, export/import and live reassignment.
- GT1/GT2/GT3 all four directions, ST, normal pointer and buttons.
- Bluetooth typing/reconnection and simultaneous registration of both keyboards
  on the Windows host. Build/host-test success does not prove the reported
  two-device pairing problem is resolved on that host.
