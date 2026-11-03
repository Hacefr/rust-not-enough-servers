fn nes_mp_draw(
    screen: &Screen,
    atlas: &HudAtlas,
    batch: &mut HudBatch,
    caret: &mut crate::ui::Caret,
    focused_at: Instant,
    cursor: (f32, f32),
) -> ::core::ops::ControlFlow<()> {
    use core::ops::ControlFlow;
    use crate::multiplayer::native_not_enough_servers as m;
    if m::DRAW.enter() {
        return ControlFlow::Continue(());
    }

    let state = m::global().lock().unwrap_or_else(|e| e.into_inner()).clone();
    let now_secs = horizon_os::time::SystemTime::now()
        .duration_since(horizon_os::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());

    if let Screen::List {
        servers,
        selected,
        pings,
        icons,
        ..
    } = screen
    {
        if state.view == m::ViewMode::Compact {
            let (w, h) = (batch.width.ceil(), batch.height.ceil());
            crate::widgets::begin_screen(|| screen.narration());
            crate::ui::menu_background(batch, atlas, 0.0, 0.0, w, h, false);
            let title = crate::lang::get_or("multiplayer.title", "Play Multiplayer");
            let tw = HudBatch::text_width(atlas, &title);
            batch.text(atlas, &title, centered_x(w, tw), ((HEADER - 9.0) / 2.0).floor(), u32::MAX, true);

            let (top, bottom, _) = list_area(w, h);
            crate::ui::list_background(batch, atlas, top, bottom, false);

            let pings_guard = pings.lock().unwrap();
            let icons_guard = icons.lock().unwrap();
            let visible: Vec<(usize, &ServerEntry)> = servers
                .iter()
                .enumerate()
                .filter(|(_, s)| state.matches_filter(&s.ip))
                .take(m::COMPACT_MAX_VISIBLE)
                .collect();

            for (slot, &(real_idx, server)) in visible.iter().enumerate() {
                let [cx, cy, cw, ch] = m::compact_card_rect(w, top, slot);
                let is_sel = *selected == Some(real_idx);
                let border = if is_sel { u32::MAX } else { rgba(70, 78, 92, 200) };
                batch.fill(atlas, cx, cy, cw, ch, border);
                batch.fill(atlas, cx + 1.0, cy + 1.0, cw - 2.0, ch - 2.0, rgba(16, 20, 26, 235));

                let icon = icons_guard.get(&server.ip);
                draw_server_icon(atlas, batch, icon.map(|p| p.as_slice()), slot, cx + 2.0, cy + 2.0);

                let pinned_tag = if state.is_pinned(&server.ip) { "[*] " } else { "" };
                let label = format!("{pinned_tag}{}", server.name);
                let name_color = if state.is_pinned(&server.ip) {
                    rgba(255, 215, 70, 255)
                } else {
                    u32::MAX
                };
                batch.text(atlas, &label, cx + 28.0, cy + 4.0, name_color, true);

                let sub = match pings_guard.get(&server.ip) {
                    Some(Ping::Ok { players: Some((on, max)), ms, .. }) => format!("{on}/{max} • {ms}ms"),
                    Some(Ping::Ok { ms, .. }) => format!("Online • {ms}ms"),
                    Some(Ping::Failed(_)) => "Offline".into(),
                    _ => state
                        .last_played(&server.ip)
                        .map(|ts| m::format_age(now_secs, ts))
                        .unwrap_or_else(|| server.ip.clone()),
                };
                batch.text(atlas, &sub, cx + 28.0, cy + 15.0, rgba(160, 168, 180, 255), true);
            }

            crate::ui::separators(batch, atlas, top, bottom, false);
            return ControlFlow::Break(());
        }
    }

    screen.draw(atlas, batch, caret, focused_at, cursor);
    m::DRAW.leave();

    if let Screen::List { servers, scroll, .. } = screen {
        let (w, h) = (batch.width.ceil(), batch.height.ceil());
        let (top, bottom, left) = list_area(w, h);
        for (i, server) in servers.iter().enumerate() {
            let y = top + 2.0 - scroll.amount.trunc() + i as f32 * ROW_HEIGHT;
            if y + ROW_HEIGHT < top || y > bottom {
                continue;
            }
            let mut badge_x = left + 37.0 + atlas.font.width(&server.name) + 6.0;
            if state.is_pinned(&server.ip) {
                let tag = "[PINNED]";
                batch.text(atlas, tag, badge_x, y + 3.0, rgba(255, 215, 70, 255), true);
                badge_x += atlas.font.width(tag) + 6.0;
            }
            if let Some(ts) = state.last_played(&server.ip) {
                let age = m::format_age(now_secs, ts);
                batch.text(atlas, &age, badge_x, y + 3.0, rgba(110, 205, 255, 255), true);
            }
        }
    }

    ControlFlow::Break(())
}
