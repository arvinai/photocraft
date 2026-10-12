# Windows pen barrel-button patch

Base: winit 0.30.13 from crates.io (Apache-2.0, LICENSE retained).
Crate SHA256: a6755fa58a9f8350bd1e472d4c3fcc25f824ec358933bba33306d0b63df5978d.

PhotoCraft v0.7.0 uses WM_POINTER pressure, but upstream discards PEN_FLAG_BARREL.
This patch maps that flag to secondary MouseInput transitions, including hover.
Pressure-bearing tip events stay Touch events. Barrel contact suppresses the tip
until lift; switching to a barrel during a stroke cancels the tip first. Leave,
pointer capture loss and focus loss release state. State is local to each window.
The mapping helper is safe Rust; existing dependency FFI is reused, no unsafe
code added. Touch devices and ordinary mouse messages keep their original paths.

The application opens the brush picker on secondary press so a held/moving
barrel gesture is usable. Erase preference, Alt resize and layer picking retain
priority. No shape, gradient, mirror or project-format customizations included.

Compatibility follow-up adapted selectively from Kindleshard/photocraft,
PR https://github.com/storytold/photocraft/pull/2475 at
ed3618835970ab224bf5fa0c7264fe1a57202a31 (PhotoCraft MIT OR Apache-2.0):
restore hover after Touch End, avoid duplicate tip motion events, retain pixel
coordinates when device rectangles are missing/invalid, and accept the generic
secondary-button flag. Out-of-range/leave input clears hover. The original local
cancel/capture-loss handling and suppression until lift remain in use; the PR's
entire input state machine and older UI code were not imported.

Tests: rustc --test scripts/pen-button-mapping-tests.rs; actual egui pointer-event
tests in brush_picker_press.rs; application paint_mouse regressions.
The complete UI library passed 1796 tests (10 ignored) and the release build completed on 2026-10-12. The user reported the installed repair appears to work on their Wacom; other tablets still need hardware testing.
