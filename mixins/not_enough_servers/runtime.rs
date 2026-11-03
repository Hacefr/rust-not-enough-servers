use std::path::{Path, PathBuf};
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};

pub const ID: &str = "not_enough_servers";
pub const NAME: &str = "Not Enough Servers";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Pins favorite servers, tracks recently played history, filters by category, and adds a 2-column compact view showing 10+ servers without scrolling.";

pub const PIN_BTN: u8 = 75;
pub const FILTER_BTN: u8 = 76;
pub const VIEW_BTN: u8 = 77;

pub const COMPACT_COLS: usize = 2;
pub const COMPACT_ROWS_PER_COL: usize = 5;
pub const COMPACT_MAX_VISIBLE: usize = COMPACT_COLS * COMPACT_ROWS_PER_COL;
pub const COMPACT_CARD_W: f32 = 152.0;
pub const COMPACT_CARD_H: f32 = 28.0;
pub const COMPACT_GAP_X: f32 = 6.0;
pub const COMPACT_GAP_Y: f32 = 3.0;

pub struct Guard(AtomicBool);

impl Guard {
    pub const fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    pub fn enter(&self) -> bool {
        self.0.swap(true, Ordering::SeqCst)
    }

    pub fn leave(&self) {
        self.0.store(false, Ordering::SeqCst)
    }
}

pub static LIST: Guard = Guard::new();
pub static BUTTONS: Guard = Guard::new();
pub static CLICK: Guard = Guard::new();
pub static DRAW: Guard = Guard::new();

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FilterMode {
    #[default]
    All,
    Pinned,
    Recent,
}

impl FilterMode {
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Pinned,
            Self::Pinned => Self::Recent,
            Self::Recent => Self::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "Tab: All",
            Self::Pinned => "Tab: Pinned",
            Self::Recent => "Tab: Recent",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    Standard,
    Compact,
}

impl ViewMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Standard => Self::Compact,
            Self::Compact => Self::Standard,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "View: List",
            Self::Compact => "View: Grid (10)",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NesState {
    pub pinned: Vec<String>,
    pub recent: Vec<(String, u64)>,
    pub filter: FilterMode,
    pub view: ViewMode,
    pub loaded: bool,
}

pub fn normalize_addr(ip: &str) -> String {
    ip.trim().to_ascii_lowercase()
}

impl NesState {
    pub fn is_pinned(&self, ip: &str) -> bool {
        let key = normalize_addr(ip);
        self.pinned.iter().any(|p| p == &key)
    }

    pub fn toggle_pin(&mut self, ip: &str) -> bool {
        let key = normalize_addr(ip);
        if let Some(pos) = self.pinned.iter().position(|p| p == &key) {
            self.pinned.remove(pos);
            false
        } else {
            self.pinned.push(key);
            true
        }
    }

    pub fn last_played(&self, ip: &str) -> Option<u64> {
        let key = normalize_addr(ip);
        self.recent.iter().find(|(a, _)| a == &key).map(|(_, ts)| *ts)
    }

    pub fn record_played(&mut self, ip: &str, now_secs: u64) {
        let key = normalize_addr(ip);
        if key.is_empty() {
            return;
        }
        self.recent.retain(|(a, _)| a != &key);
        self.recent.insert(0, (key, now_secs));
        if self.recent.len() > 32 {
            self.recent.truncate(32);
        }
    }

    pub fn matches_filter(&self, ip: &str) -> bool {
        match self.filter {
            FilterMode::All => true,
            FilterMode::Pinned => self.is_pinned(ip),
            FilterMode::Recent => self.last_played(ip).is_some(),
        }
    }

    pub fn sort_key(&self, ip: &str, original_index: usize) -> (u8, u64, usize) {
        let pinned_rank = if self.is_pinned(ip) { 0 } else { 1 };
        let recency_rank = u64::MAX - self.last_played(ip).unwrap_or(0);
        (pinned_rank, recency_rank, original_index)
    }

    pub fn parse(text: &str) -> Self {
        let mut state = Self {
            loaded: true,
            ..Self::default()
        };
        for line in text.lines() {
            let line = line.trim();
            if let Some(addr) = line.strip_prefix("pin:") {
                let key = normalize_addr(addr);
                if !key.is_empty() && !state.pinned.contains(&key) {
                    state.pinned.push(key);
                }
            } else if let Some(rest) = line.strip_prefix("recent:") {
                if let Some((addr, ts_str)) = rest.rsplit_once('|') {
                    if let Ok(ts) = ts_str.parse::<u64>() {
                        let key = normalize_addr(addr);
                        if !key.is_empty() {
                            state.recent.push((key, ts));
                        }
                    }
                }
            } else if let Some(v) = line.strip_prefix("view:") {
                state.view = if v.trim() == "compact" {
                    ViewMode::Compact
                } else {
                    ViewMode::Standard
                };
            }
        }
        state
    }

    pub fn serialize(&self) -> String {
        let mut out = String::new();
        out.push_str(match self.view {
            ViewMode::Standard => "view:standard\n",
            ViewMode::Compact => "view:compact\n",
        });
        for p in &self.pinned {
            out.push_str(&format!("pin:{p}\n"));
        }
        for (addr, ts) in &self.recent {
            out.push_str(&format!("recent:{addr}|{ts}\n"));
        }
        out
    }
}

pub fn state_file(saves: &Path) -> PathBuf {
    saves
        .parent()
        .unwrap_or(saves)
        .join("not-enough-servers.txt")
}

pub fn global() -> &'static Mutex<NesState> {
    static STATE: OnceLock<Mutex<NesState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(NesState::default()))
}

pub fn ensure_loaded(saves: &Path) {
    let mut guard = global().lock().unwrap_or_else(|e| e.into_inner());
    if !guard.loaded {
        let path = state_file(saves);
        if let Ok(text) = std::fs::read_to_string(&path) {
            *guard = NesState::parse(&text);
        } else {
            guard.loaded = true;
        }
    }
}

pub fn save_disk(saves: &Path, state: &NesState) {
    let path = state_file(saves);
    let _ = std::fs::write(path, state.serialize());
}

pub fn format_age(now_secs: u64, played_secs: u64) -> String {
    let diff = now_secs.saturating_sub(played_secs);
    if diff < 60 {
        "Just now".into()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

pub fn compact_card_rect(w: f32, top: f32, slot: usize) -> [f32; 4] {
    let total_w = COMPACT_COLS as f32 * COMPACT_CARD_W + (COMPACT_COLS - 1) as f32 * COMPACT_GAP_X;
    let start_x = ((w - total_w) / 2.0).floor();
    let col = slot % COMPACT_COLS;
    let row = slot / COMPACT_COLS;
    let x = start_x + col as f32 * (COMPACT_CARD_W + COMPACT_GAP_X);
    let y = top + 4.0 + row as f32 * (COMPACT_CARD_H + COMPACT_GAP_Y);
    [x, y, COMPACT_CARD_W, COMPACT_CARD_H]
}

pub fn compact_slot_at(w: f32, top: f32, bottom: f32, (cx, cy): (f32, f32), count: usize) -> Option<usize> {
    if cy < top || cy >= bottom {
        return None;
    }
    let limit = count.min(COMPACT_MAX_VISIBLE);
    for slot in 0..limit {
        let [x, y, cw, ch] = compact_card_rect(w, top, slot);
        if cx >= x && cx < x + cw && cy >= y && cy < y + ch {
            return Some(slot);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_prevents_recursion() {
        let g = Guard::new();
        assert!(!g.enter());
        assert!(g.enter());
        g.leave();
        assert!(!g.enter());
    }

    #[test]
    fn pinning_and_recent_sorting_work() {
        let mut s = NesState::default();
        assert!(s.toggle_pin("Play.Example.com"));
        assert!(s.is_pinned("play.example.com"));
        s.record_played("mc.other.org", 1000);
        s.record_played("play.example.com", 500);

        let text = s.serialize();
        let loaded = NesState::parse(&text);
        assert!(loaded.is_pinned("play.example.com"));
        assert_eq!(loaded.last_played("mc.other.org"), Some(1000));
        assert!(loaded.sort_key("play.example.com", 1) < loaded.sort_key("mc.other.org", 0));
    }

    #[test]
    fn compact_grid_fits_ten_servers_without_scrolling() {
        let (w, top, bottom) = (480.0, 33.0, 210.0);
        for slot in 0..COMPACT_MAX_VISIBLE {
            let [x, y, cw, ch] = compact_card_rect(w, top, slot);
            assert!(y >= top && y + ch <= bottom);
            assert_eq!(
                compact_slot_at(w, top, bottom, (x + cw * 0.5, y + ch * 0.5), 10),
                Some(slot)
            );
        }
    }
}
