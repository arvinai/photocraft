//! Windows Ink barrel-button mapping, independent of Win32 and pressure sampling.
//! A barrel gesture must never start a tip/left-button stroke. Once interrupted,
//! a tip stroke stays suppressed until the tip leaves the tablet.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TipPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Transitions {
    pub secondary: Option<bool>,
    pub tip: Option<TipPhase>,
}

impl Transitions {
    // Touch End clears egui's pointer; restore hover after that event. Started
    // and Moved already position it, so they must not also emit CursorMoved.
    pub fn cursor_before(&self) -> bool {
        self.tip.is_none() || self.tip == Some(TipPhase::Cancelled)
    }

    pub fn cursor_after(&self) -> bool {
        self.tip == Some(TipPhase::Ended)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct DeviceMapping {
    pub device_width: f64,
    pub device_height: f64,
    pub display_left: f64,
    pub display_top: f64,
    pub display_width: f64,
    pub display_height: f64,
}

pub(crate) fn pointer_position(
    mapping: Option<DeviceMapping>,
    himetric: (f64, f64),
    pixels: (f64, f64),
) -> (f64, f64) {
    if let Some(m) = mapping {
        if m.device_width > 0.0 && m.device_height > 0.0 {
            let x = m.display_left + himetric.0 * m.display_width / m.device_width;
            let y = m.display_top + himetric.1 * m.display_height / m.device_height;
            if x.is_finite() && y.is_finite() {
                return (x, y);
            }
        }
    }
    pixels
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PenButtons {
    secondary: bool,
    tip: bool,
    suppressed: bool,
}

impl PenButtons {
    pub fn update(&mut self, contact: bool, barrel: bool, cancelled: bool) -> Transitions {
        if cancelled {
            return self.reset();
        }
        let secondary = (barrel != self.secondary).then_some(barrel);
        self.secondary = barrel;
        let tip = if barrel {
            self.suppressed |= contact || self.tip;
            let was_tip = self.tip;
            self.tip = false;
            was_tip.then_some(TipPhase::Cancelled)
        } else if self.suppressed {
            None
        } else if contact {
            let phase = if self.tip { TipPhase::Moved } else { TipPhase::Started };
            self.tip = true;
            Some(phase)
        } else {
            let was_tip = self.tip;
            self.tip = false;
            was_tip.then_some(TipPhase::Ended)
        };
        if !contact {
            self.suppressed = false;
        }
        Transitions { secondary, tip }
    }

    pub fn reset(&mut self) -> Transitions {
        let events = Transitions {
            secondary: self.secondary.then_some(false),
            tip: self.tip.then_some(TipPhase::Cancelled),
        };
        *self = Self::default();
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hover_is_restored_after_lift_without_duplicate_stroke_positions() {
        for tip in [None, Some(TipPhase::Cancelled)] {
            let events = Transitions { secondary: None, tip };
            assert!(events.cursor_before());
            assert!(!events.cursor_after());
        }
        let end = Transitions { secondary: None, tip: Some(TipPhase::Ended) };
        assert!(!end.cursor_before());
        assert!(end.cursor_after());
        for phase in [TipPhase::Started, TipPhase::Moved] {
            let events = Transitions { secondary: None, tip: Some(phase) };
            assert!(!events.cursor_before() && !events.cursor_after());
        }
    }

    #[test]
    fn coordinate_mapping_preserves_monitor_offset_and_fractional_position() {
        let m = DeviceMapping {
            device_width: 1000.0,
            device_height: 500.0,
            display_left: -1920.0,
            display_top: 120.0,
            display_width: 1920.0,
            display_height: 1080.0,
        };
        assert_eq!(pointer_position(Some(m), (250.0, 125.0), (0.0, 0.0)), (-1440.0, 390.0));
        let p = pointer_position(Some(m), (0.5, 0.5), (0.0, 0.0));
        assert!((p.0 + 1919.04).abs() < 1e-6);
        assert!((p.1 - 121.08).abs() < 1e-6);
    }

    #[test]
    fn missing_invalid_or_nonfinite_device_rectangles_use_pixel_position() {
        let pixels = (-480.0, 240.0);
        assert_eq!(pointer_position(None, (50.0, 80.0), pixels), pixels);
        for width in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let m = DeviceMapping {
                device_width: width,
                device_height: 100.0,
                display_left: 0.0,
                display_top: 0.0,
                display_width: f64::INFINITY,
                display_height: 100.0,
            };
            assert_eq!(pointer_position(Some(m), (50.0, 80.0), pixels), pixels);
        }
    }

    #[test]
    fn hovering_barrel_emits_exactly_one_press_and_release() {
        let mut state = PenButtons::default();
        assert_eq!(
            state.update(false, true, false),
            Transitions { secondary: Some(true), tip: None }
        );
        assert_eq!(state.update(false, true, false), Transitions::default());
        assert_eq!(
            state.update(false, false, false),
            Transitions { secondary: Some(false), tip: None }
        );
        assert_eq!(state.update(false, false, false), Transitions::default());
    }

    #[test]
    fn barrel_contact_never_creates_a_left_stroke() {
        let mut state = PenButtons::default();
        assert_eq!(state.update(true, true, false).tip, None);
        assert_eq!(state.update(true, false, false).tip, None);
        assert_eq!(state.update(false, false, false).tip, None);
        assert_eq!(state.update(true, false, false).tip, Some(TipPhase::Started));
    }

    #[test]
    fn pressing_barrel_during_a_stroke_cancels_the_tip_once() {
        let mut state = PenButtons::default();
        assert_eq!(state.update(true, false, false).tip, Some(TipPhase::Started));
        assert_eq!(
            state.update(true, true, false),
            Transitions { secondary: Some(true), tip: Some(TipPhase::Cancelled) }
        );
        assert_eq!(state.update(true, true, false), Transitions::default());
        assert_eq!(
            state.update(true, false, false),
            Transitions { secondary: Some(false), tip: None }
        );
        assert_eq!(state.update(false, false, false).tip, None);
    }

    #[test]
    fn regular_tip_keeps_start_move_end_and_can_start_again() {
        let mut state = PenButtons::default();
        for phase in [TipPhase::Started, TipPhase::Moved, TipPhase::Moved] {
            assert_eq!(
                state.update(true, false, false),
                Transitions { secondary: None, tip: Some(phase) }
            );
        }
        assert_eq!(state.update(false, false, false).tip, Some(TipPhase::Ended));
        assert_eq!(state.update(true, false, false).tip, Some(TipPhase::Started));
    }

    #[test]
    fn cancel_and_leave_release_buttons_and_allow_the_next_gesture() {
        let mut state = PenButtons::default();
        state.update(true, true, false);
        assert_eq!(
            state.update(true, true, true),
            Transitions { secondary: Some(false), tip: None }
        );
        assert_eq!(state.reset(), Transitions::default());
        assert_eq!(state.update(false, true, false).secondary, Some(true));
        assert_eq!(state.reset().secondary, Some(false));
        assert_eq!(state.update(true, false, false).tip, Some(TipPhase::Started));
        assert_eq!(state.reset().tip, Some(TipPhase::Cancelled));
    }

    #[test]
    fn right_and_tip_never_stay_down_together_for_any_sequence() {
        for sequence in 0_u32..4096 {
            let mut state = PenButtons::default();
            for shift in (0..12).step_by(2) {
                let sample = (sequence >> shift) & 3;
                state.update(sample & 1 != 0, sample & 2 != 0, false);
                assert!(!(state.secondary && state.tip));
            }
            state.reset();
            assert!(!state.secondary && !state.tip && !state.suppressed);
        }
    }
}
