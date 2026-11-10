# Three Vial gesture triggers and output crosses

Based on the hardware-tested assignable-trigger build 8e52c68. The COROPIT
variant uses the same implementation with the existing X/Y polarity difference.
The previous single-trigger branches remain unchanged.

Vial custom keycodes:

| Trigger | User ID / VIA code | Output cross |
| --- | --- | --- |
| Gesture Trigger | 10 / 0x7E0A (unchanged) | Left |
| Gesture Trigger 2 | 11 / 0x7E0B | Middle |
| Gesture Trigger 3 | 12 / 0x7E0C | Right |

Reconnect Vial after flashing to reload its definition. Place each trigger on
any physical key and layer, then assign that layer's corresponding cross.
These are three independent sets of four editable outputs on the same layer.
Their defaults are all No. The original trigger defaults to layer-3 V; triggers
2 and 3 are not assigned to physical keys by default. Existing saved physical
layouts, macros and tap dances need not be deliberately reset. Save a new .vil
including the new assignments after configuration.

Motion rules remain >30 summed absolute accumulated X/Y counts, dominant axis
(horizontal ties), immediate firing, 150ms shared cooldown, no cursor leakage
while held. Multiple physical copies of the same trigger are supported. If two
or three different trigger numbers are held, capture stays active but no bank
fires. Once only one number remains, its accumulator starts fresh. Switching
bank or layer also discards unfinished motion; releasing alone emits nothing.
Outputs still use RMK's ordinary current-layer/transparent resolution and paired
press/release, including live Vial edits, modifiers and macros.

## Keep the existing storage dimensions

The original 5x11 keymap has 43 physical positions. The unlisted (0,5) cell and
all eleven row-4 cells give exactly twelve virtual positions, without increasing
rows, columns, layers or changing physical split matrices, pins or key positions.
The previous four output coordinates are retained.

| Cross | Left | Right | Up | Down |
| --- | --- | --- | --- | --- |
| 1 | 4,0 | 4,1 | 4,2 | 4,3 |
| 2 | 4,4 | 4,5 | 4,6 | 4,7 |
| 3 | 4,8 | 4,9 | 4,10 | 0,5 |

The twelfth position is a virtual output, not a new physical key. The trigger
monitor excludes all twelve output positions from trigger detection, preventing
recursion even if a trigger code is mistakenly assigned to an output. User0..9,
including Bluetooth controls and split-peer clearing, retain their IDs.
The keymap shape stays the same; no explicit storage reset is added. Stored
bindings still override compiled defaults, as before.

Host integration tests run the pinned RMK Keyboard, live host keymap edits,
monitor and pointing router concurrently. They verify all twelve distinct HID
outputs, release, both new banks' live output edits, conflict capture, reset on
bank changes, normal cursor recovery and rejection of synthetic trigger actions.
Existing threshold, timing, backpressure, remapping, macro and modifier tests
remain in place. Hardware verification is reported separately from builds.
