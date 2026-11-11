# ST: Vial-assignable vertical scroll trigger

Based on the GT1/GT2/GT3 label build de370e0 (stock) / 5ecfeb2 (COROPIT).
Existing branches and main are retained. No dependency update is required.

After flashing, reconnect Vial/Pipette to reload the firmware definition and
assign **ST** to a physical key on any layer. ST is User13 / VIA 0x7E0D;
User0..12, including GT1..GT3 and Bluetooth functions, retain their identities.
No physical ST binding is installed by default. Multiple copies may be assigned.

Hold ST and move the ball vertically to scroll. Horizontal motion produces
neither pan nor cursor motion. Release the last ST key to resume the normal
cursor. ST itself sends no ordinary keyboard key. Existing mouse buttons are
preserved in scroll reports. Stored layouts, encoder defaults, virtual crosses,
macros, tap dances, unlock and sensor settings are retained unchanged.

## Initial speed and direction

The pinned RMK `ScrollConfig` defaults are retained for Y: multiplier 1,
divisor 8, invert_y false. Eight signed accumulated logical Y sensor counts
produce one wheel unit; positive Y produces negative wheel. Screen direction
and scrolling distance also depend on the operating system/application settings.
The sensor CPI remains 800. X is disabled with multiplier_x=0 and divisor_x=0.
Speed and direction are firmware defaults, not new Vial-adjustable settings.

## Priority and state reset

Priority is any held GT > ST > ordinary cursor. All three gesture banks retain
their threshold, 150ms cooldown, release behavior and conflict handling. If
multiple different gesture banks are held, motion remains captured without
scrolling. Pressing/releasing ST during a gesture does not reset that gesture's
unfinished stroke. When a gesture ends while ST is still held, scrolling resumes
with a fresh accumulator.

ST release/repress, gesture preemption/resume, and layer changes discard old
scroll fractions, including when no motion occurred between key transitions.
There is no timer-driven scroll repetition or inertia. Release cannot generate
extra wheel output. Already delivered OS input is not retracted.

RMK's `PointingProcessor::set_pointing_mode` does not clear its private
accumulator. This implementation uses a separate PointingProcessor for scroll
and reconstructs only that object using its public constructor on a scroll or
gesture epoch change. It reuses RMK's transformation, accumulation, button and
HID report path. The original pointer processor and generated devices/tasks
remain unchanged; the new object is used via Processor::process, without a
second subscription or input task.

Host integration tests exercise the actual pinned RMK Keyboard, live host
keymap edits, trigger monitor and router concurrently. They check vertical-only
wheel values, sub-unit accumulation, release/repress without intermediate motion,
all three gesture priorities, conflicting gestures, live removal of a held ST,
multiple ST copies, layer transparency, normal cursor recovery, mouse buttons,
quick taps and full keyboard queues. Virtual gesture slots cannot arm ST.
Build/test success is separate from hardware validation of direction and feel.
