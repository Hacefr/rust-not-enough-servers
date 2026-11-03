fn nes_mp_list(saves: &Path) -> ::core::ops::ControlFlow<Screen> {
    use core::ops::ControlFlow;
    use crate::multiplayer::native_not_enough_servers as m;
    if m::LIST.enter() {
        return ControlFlow::Continue(());
    }
    m::ensure_loaded(saves);
    let mut screen = Screen::list(saves);
    m::LIST.leave();
    if let Screen::List { servers, .. } = &mut screen {
        let state = m::global().lock().unwrap_or_else(|e| e.into_inner());
        let mut indexed: Vec<(usize, ServerEntry)> = servers.drain(..).enumerate().collect();
        indexed.sort_by_key(|(idx, entry)| {
            if idx < &pinned() {
                (0, 0, *idx)
            } else {
                let (p, r, o) = state.sort_key(&entry.ip, *idx);
                (p + 1, r, o)
            }
        });
        *servers = indexed.into_iter().map(|(_, e)| e).collect();
    }
    ControlFlow::Break(screen)
}
