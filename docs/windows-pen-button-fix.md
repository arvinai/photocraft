# Windows pen barrel-button repair reference

Base: PhotoCraft v0.7.0, `79261615956ab21334e49ca1d384390272e647da`.

The Windows Ink path in winit 0.30.13 forwards pen pressure as touch input but
discards `PEN_FLAG_BARREL`. A Wacom barrel button can consequently fail to open
the brush picker and barrel contact can produce a left-button stroke instead.

The vendored dependency maps barrel changes to balanced secondary-button events.
Normal tip events retain pressure. Barrel contact suppresses the tip until lift;
pressing the barrel during a stroke releases the tip first. Cancellation, capture
loss, leaving the window and focus loss release tracked buttons. The UI opens
the picker on secondary press, preserving Alt resize, Ctrl layer picking and the
right-button erase preference. No shape, gradient, symmetry or format changes
are included. Dependency provenance is in `vendor/winit/PHOTOCRAFT-PATCH.md`.

## Validation on Windows, 2026-10-12

- Six pure pen-state tests pass, including 4096 button sequences.
- Four real egui pointer-event tests pass. Application canvas tests cover a held
  secondary press without painting and Alt resize priority.
- UI library: 1796 passed, 10 ignored, no failures.
- Application binary tests, formatting, all-target Clippy with warnings denied,
  dependency-layer checks and all 24 wasm compatibility checks completed in the
  successful build/install run.
- The release build completed. The installed executable was hash-checked against
  its build output before updating the local launcher.
- The user reported that the installed repair appears to work on their Wacom.
  Extended hardware checks remain useful; coverage of every tablet is not claimed.

Reproduce the checks:

```text
rustc --test scripts/pen-button-mapping-tests.rs -o target/pen-button-mapping-tests.exe
target/pen-button-mapping-tests.exe
cargo test -p photocraft-ui-egui --lib
cargo clippy -p photocraft-ui-egui -p photocraft --all-targets -- -D warnings
cargo xtask layers
cargo xtask wasm
```

For hardware acceptance, check hover barrel press, holding while moving, barrel
contact without an unintended stroke, normal pressure, mouse right-click, Alt
resize and switching away from and back to the window. Check any Wacom profile
against the executable being tested.

## Upstream contribution status

PR #2475 compatibility changes have been selectively ported to this v0.7.0
reference: hover restoration after lift, pixel-coordinate fallback when device
rectangles are unavailable, generic secondary-button flag support, and a picker
regression at the canvas edge. Local tip suppression until lift and balanced
cancel/capture-loss handling are retained. This does not merge the PR's entire
branch or change the official base version. Three new pure input tests accompany
the compatibility port; the UI regression verifies the clamped picker stays open
without recording a paint stroke.

Compatibility-port validation on Windows: 9 pure input tests, 1797 UI library
tests (10 ignored), and 92 application tests passed. Formatting, all-target
Clippy with warnings denied, dependency-layer checks and the 24 wasm checks passed.
The initial Wacom acceptance above describes the previous repair build; this
compatibility follow-up still requires a new physical-tablet check after installation.

Official main was checked at `84aa8df05ab26b41283e1192791764d545cf34a2`.
Several open proposals overlap this work:

- [#2475](https://github.com/storytold/photocraft/pull/2475): a vendored winit
  Windows Ink repair with essentially the same approach.
- [#1911](https://github.com/storytold/photocraft/pull/1911): responsive stylus
  right-click and Wintab input.
- [#3838](https://github.com/storytold/photocraft/pull/3838): Windows Ink reader
  improvements stacked on #1911.

Publish this released-version branch as a tested reference in a personal fork.
Before a new upstream PR, identify a distinct remaining defect or test gap,
adapt only that change to current main and reference the overlapping proposals.
The v0.7.0 build does not establish validation of current main.
