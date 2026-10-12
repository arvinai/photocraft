//! Recognize a held/moving pen barrel gesture without waiting for a click release.

pub(super) fn opens(response: &egui::Response, blocked: bool) -> bool {
    !blocked
        && (response.secondary_clicked() || (response.contains_pointer() && response.ctx.input(|i| i.pointer.button_pressed(egui::PointerButton::Secondary))))
}

#[cfg(test)]
mod tests {
    use egui::{Context, Event, Modifiers, PointerButton, RawInput, Rect, Sense, pos2, vec2};

    use super::opens;

    fn frame(ctx: &Context, events: Vec<Event>, blocked: bool) -> bool {
        let mut opened = false;
        let mut output =
            ctx.run_ui(RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 300.0))), events, ..Default::default() }, |ui| {
                let response = ui.allocate_response(vec2(250.0, 200.0), Sense::click_and_drag());
                opened |= opens(&response, blocked) && response.interact_pointer_pos().is_some();
            });
        output.textures_delta.clear();
        opened
    }

    fn button(pressed: bool) -> Event {
        Event::PointerButton { pos: pos2(60.0, 60.0), button: PointerButton::Secondary, pressed, modifiers: Modifiers::NONE }
    }

    #[test]
    fn hovering_barrel_opens_on_press_and_not_on_held_frames() {
        let ctx = Context::default();
        frame(&ctx, vec![], false);
        frame(&ctx, vec![Event::PointerMoved(pos2(60.0, 60.0))], false);
        assert!(frame(&ctx, vec![button(true)], false));
        assert!(!frame(&ctx, vec![], false));
        assert!(!frame(&ctx, vec![Event::PointerMoved(pos2(150.0, 100.0))], false));
    }

    #[test]
    fn resize_erase_and_layer_menu_policies_can_reserve_the_press() {
        let ctx = Context::default();
        frame(&ctx, vec![], true);
        frame(&ctx, vec![Event::PointerMoved(pos2(60.0, 60.0))], true);
        assert!(!frame(&ctx, vec![button(true)], true));
        assert!(!frame(&ctx, vec![button(false)], true));
    }

    #[test]
    fn presses_outside_the_canvas_do_not_open_the_picker() {
        let ctx = Context::default();
        frame(&ctx, vec![], false);
        let pos = pos2(350.0, 250.0);
        frame(&ctx, vec![Event::PointerMoved(pos)], false);
        assert!(!frame(&ctx, vec![Event::PointerButton { pos, button: PointerButton::Secondary, pressed: true, modifiers: Modifiers::NONE }], false));
    }

    #[test]
    fn a_mouse_right_click_still_opens_the_picker() {
        let ctx = Context::default();
        frame(&ctx, vec![], false);
        frame(&ctx, vec![Event::PointerMoved(pos2(60.0, 60.0))], false);
        assert!(frame(&ctx, vec![button(true), button(false)], false));
    }
}
