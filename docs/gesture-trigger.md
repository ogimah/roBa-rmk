# Vial-assignable gesture trigger

Based on the hardware-tested `feat/gesture-user-defaults` commit e323304.
The previous stock and COROPIT branches remain available unchanged.

## Configure in Vial / Pipette

1. Flash the matching hardware variant and reconnect Vial to reload the definition.
2. Select the desired layer and physical key. In the custom keycodes list choose
   **Gesture Trigger**. Its code is User10 / `0x7E0A`.
3. On that layer, assign the four existing virtual cross keys to the desired
   outputs. They are initially empty. The ordinary RMK active-layer/transparent
   resolution applies to both trigger and outputs.
4. Hold the assigned key and move the trackball. More than 30 summed absolute
   accumulated X/Y counts fires immediately; repeated strokes are allowed every
   150ms. Release cancels capture. No cursor movement leaks during capture.

The compiled default places Gesture Trigger on layer 3 at the physical V
position. Stored layouts override compiled defaults: if that key remains No
following an upgrade or loading the original `.vil`, assign Gesture Trigger
manually. Do not reset the saved layout just to enable the trigger. No storage
clear is introduced. Save a new `.vil` after making the assignment.

The old fixed-physical-V trigger no longer exists. A trigger can be placed on
any of the eight layers and on multiple physical keys; capture remains active
until all held trigger keys are released. These copies share the same four
virtual output positions, resolved in the current layer. Separate output banks
for different trigger keys on the same layer are not part of this change.
If the active layer changes while a trigger is held, its hold remains valid;
unfinished motion resets and subsequent outputs use the new active layer.

The legacy `migrate_gesture_vil.py` script was written for the fixed-trigger
firmware and sets layer-3 V to No. Do not use that script to configure this build.

## Verified API and implementation

RMK stays pinned to 23c0960962ada901780b3c8d36be21a91c3f1bbc.
`host/via/keycode_convert.rs` maps 0x7E00..0x7E0F to `Action::User(0..15)`.
`keyboard.rs::process_key_action_normal` publishes `ActionEvent` after the normal
live Vial keymap, transparent-layer and release-cache resolution.
`process_user` reserves User0..8 for BLE functions and User9 for clearing split
peers. User10 has no built-in BLE/HID action. The Vial definition keeps entries
0..8 byte-for-byte equivalent and labels User9 explicitly before adding User10.

The independent trigger task observes resolved ActionEvent presses, tracks each
physical held position, and drains raw KeyboardEvent releases as well. This
prevents a stuck trigger when Vial changes a held binding to No (which produces
no resolved release action). Before processing a delayed action, it drains the
raw queue so a completed quick tap cannot arm a held trigger. Virtual output
row 4 and encoders cannot arm capture. Output dispatch, threshold, cooldown,
geometry, wiring, macros, tap dances, unlock and dependency versions are retained.

Host tests run the actual pinned RMK Keyboard, live host keymap edits, trigger
monitor and pointing router concurrently, including queue backpressure, multiple
holds, transparency on layer 2, remapping while held, quick taps, output release,
modifiers/macros and normal V input. UF2 compilation and these tests do not
replace hardware verification with Vial / Pipette.
