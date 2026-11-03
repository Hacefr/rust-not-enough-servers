fn nes_mp_click(
    screen: &mut Screen,
    id: Option<u8>,
    x: f32,
    y: f32,
    size: (f32, f32),
    saves: &Path,
) -> ::core::ops::ControlFlow<Outcome> {
    use core::ops::ControlFlow;
    use crate::multiplayer::native_not_enough_servers as m;
    if m::CLICK.enter() {
        return ControlFlow::Continue(());
    }
    m::ensure_loaded(saves);

    if let Screen::List { servers, selected, .. } = screen {
        match id {
            Some(m::FILTER_BTN) => {
                let mut state = m::global().lock().unwrap_or_else(|e| e.into_inner());
                state.filter = state.filter.next();
                return ControlFlow::Break(Outcome::None);
            }
            Some(m::VIEW_BTN) => {
                let mut state = m::global().lock().unwrap_or_else(|e| e.into_inner());
                state.view = state.view.toggle();
                m::save_disk(saves, &state);
                return ControlFlow::Break(Outcome::None);
            }
            Some(m::PIN_BTN) => {
                if let Some(idx) = *selected
                    && let Some(entry) = servers.get(idx)
                {
                    let ip = entry.ip.clone();
                    let mut state = m::global().lock().unwrap_or_else(|e| e.into_inner());
                    state.toggle_pin(&ip);
                    m::save_disk(saves, &state);
                    drop(state);
                    *screen = Screen::list(saves);
                }
                return ControlFlow::Break(Outcome::None);
            }
            None => {
                let state = m::global().lock().unwrap_or_else(|e| e.into_inner());
                if state.view == m::ViewMode::Compact {
                    let visible: Vec<usize> = servers
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| state.matches_filter(&s.ip))
                        .map(|(i, _)| i)
                        .collect();
                    let (w, h) = (size.0.ceil(), size.1.ceil());
                    let (top, bottom, _) = list_area(w, h);
                    if let Some(slot) = m::compact_slot_at(w, top, bottom, (x, y), visible.len()) {
                        let real_idx = visible[slot];
                        if *selected == Some(real_idx) {
                            let addr = servers[real_idx].ip.clone();
                            drop(state);
                            let now = horizon_os::time::SystemTime::now()
                                .duration_since(horizon_os::time::UNIX_EPOCH)
                                .map_or(0, |d| d.as_secs());
                            let mut st = m::global().lock().unwrap_or_else(|e| e.into_inner());
                            st.record_played(&addr, now);
                            m::save_disk(saves, &st);
                            return ControlFlow::Break(Outcome::Connect { address: addr, lan: false });
                        }
                        *selected = Some(real_idx);
                        return ControlFlow::Break(Outcome::None);
                    }
                }
            }
            _ => {}
        }
    }

    let outcome = screen.click(id, x, y, size, saves);
    m::CLICK.leave();

    if let Outcome::Connect { address, lan: false } = &outcome {
        let now = horizon_os::time::SystemTime::now()
            .duration_since(horizon_os::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let mut state = m::global().lock().unwrap_or_else(|e| e.into_inner());
        state.record_played(address, now);
        m::save_disk(saves, &state);
    }

    ControlFlow::Break(outcome)
}
