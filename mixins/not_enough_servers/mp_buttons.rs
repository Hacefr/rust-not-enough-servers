fn nes_mp_buttons(screen: &Screen, w: f32, h: f32) -> ::core::ops::ControlFlow<Vec<ButtonSpec>> {
    use core::ops::ControlFlow;
    use crate::multiplayer::native_not_enough_servers as m;
    if m::BUTTONS.enter() {
        return ControlFlow::Continue(());
    }
    let mut buttons = screen.buttons(w, h);
    m::BUTTONS.leave();
    if let Screen::List { servers, selected, .. } = screen {
        let state = m::global().lock().unwrap_or_else(|e| e.into_inner());
        let (w, _) = (w.ceil(), h.ceil());
        let left = ((w - 310.0) / 2.0).floor();
        let right = left + 310.0;
        let pin_label = match selected.and_then(|i| servers.get(i)) {
            Some(s) if state.is_pinned(&s.ip) => "Unpin",
            _ => "Pin",
        };
        let can_pin = selected.is_some_and(|i| i < servers.len());
        buttons.push((left, 6.0, 76.0, state.filter.label().to_owned(), true, m::FILTER_BTN));
        buttons.push((right - 132.0, 6.0, 78.0, state.view.label().to_owned(), true, m::VIEW_BTN));
        buttons.push((right - 50.0, 6.0, 50.0, pin_label.to_owned(), can_pin, m::PIN_BTN));
    }
    ControlFlow::Break(buttons)
}
