Current ST branch: `feat/scroll-trigger-coropit`. See [ST setup](scroll-trigger.md).
GT labels are GT1..GT3; the reversed encoder defaults are retained.

Current branch: `feat/gesture-labels-coropit`. Gesture labels are now
Gesture1..Gesture3 and initial encoder wheel bindings are reversed.
Sensor configuration and gesture processing remain unchanged.

**Three-trigger branch: `feat/gesture-three-coropit`.**
See [three-trigger setup](gesture-three.md). This extends the working
assignable-trigger COROPIT firmware; its sensor settings below are retained.

# Assignable-trigger COROPIT build

This branch `feat/gesture-trigger-coropit` applies the same User10 trigger
upgrade as `feat/gesture-trigger-vial`, with only the sensor axis polarities
changed. See [Vial setup](gesture-trigger.md). The physical V position on layer 3
is merely the default binding, no longer a fixed trigger. Other initial layout,
macros and tap dances remain unchanged. The following orientation reference
remains applicable; the old branches are retained as hardware-tested backups.

# COROPIT trackball variant

This branch is for roBaish/moNa2 with the COROPIT replacement trackball. The
stock-trackball build is retained on `feat/gesture-user-defaults` at
`e323304c8f7c208f38c685d1348d07d53f3a9a0e`, which the user confirmed working.

Reference supplied by the user:
https://github.com/black-trooper/zmk-config-moNa2/commit/c1f7dcc5453e3a5b96738811c68e6c66e976c586

That commit changes only PMW3610 orientation from 180 degrees to 0 degrees.
Its west manifest references sayu-hub/zmk-pmw3610-driver on moving main.
The driver was inspected at 2618e8550876e4a3e06215bf0138c4c424d0398b:
orientation 180 produces `(raw_x, -raw_y)`, orientation 0 produces
`(-raw_x, raw_y)`. Both axes therefore reverse, without swapping X and Y.
This source inspection is not a claim to have reconstructed the unpinned
driver version used in 2025.

The same relative 180-degree change in the pinned RMK sensor driver toggles
the two hardware inversion flags:

| Setting | Stock | COROPIT |
| --- | --- | --- |
| invert_x | false | true |
| invert_y | true | false |
| swap_xy | false | false |
| CPI | 800 | 800 |

No reference ZMK CPI, pin or scroll setting is copied. Those are not changed
by the supplied commit. RMK's sensor transforms occur before motion events,
so both ordinary cursor and gesture directions use the corrected orientation
without adding another inversion in the gesture processor.

The initial 261109_robaish layout, tap dances, macros, encoder, blank virtual
gesture slots, Layer 3 + V trigger, threshold 30, 150ms repeat, unlock keys,
USB/BLE, physical split matrices and dependencies remain unchanged. Tests
compare the entire parsed configuration and relevant source files against the
working stock build, permitting only these two sensor settings.

In this branch's successful Actions run, download `roBa-central.uf2` for the
right half and `roBa-peripheral.uf2` for the left. The orientation change is
right-hand sensor configuration; a peripheral image is also produced for a
matched pair. No explicit storage reset is added and normal Vial assignments
remain editable.

Compilation and automated tests do not validate the replacement hardware.
With COROPIT installed, check normal cursor up/down/left/right, the four
Vial-configured gesture outputs, and ordinary keyboard/encoder operation.
Use the stock branch's UF2s when reverting to the original trackball.
