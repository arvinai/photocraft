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
