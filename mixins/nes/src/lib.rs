use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerEntry {
    pub name: String,
    pub address: String,
    pub pinned: bool,
    pub last_played: u64,
}

#[derive(Default, Serialize, Deserialize)]
pub struct ServerManager {
    pub servers: Vec<ServerEntry>,
}

impl ServerManager {
    pub fn global() -> &'static Mutex<Self> {
        static INSTANCE: OnceLock<Mutex<ServerManager>> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let mut mgr = ServerManager::default();
            mgr.servers.push(ServerEntry {
                name: "Official Horizon Public Server".into(),
                address: "play.playhorizon.dev".into(),
                pinned: true,
                last_played: 0,
            });
            Mutex::new(mgr)
        })
    }

    pub fn pin_server(&mut self, address: &str) {
        if let Some(s) = self.servers.iter_mut().find(|s| s.address == address) {
            s.pinned = !s.pinned;
        } else {
            self.servers.push(ServerEntry {
                name: address.to_owned(),
                address: address.to_owned(),
                pinned: true,
                last_played: 0,
            });
        }
    }

    pub fn record_played(&mut self, name: &str, address: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if let Some(s) = self.servers.iter_mut().find(|s| s.address == address) {
            s.last_played = now;
        } else {
            self.servers.push(ServerEntry {
                name: name.to_owned(),
                address: address.to_owned(),
                pinned: false,
                last_played: now,
            });
        }
        self.servers.sort_by(|a, b| {
            if a.pinned != b.pinned {
                b.pinned.cmp(&a.pinned)
            } else {
                b.last_played.cmp(&a.last_played)
            }
        });
    }
}

#[no_mangle]
pub extern "C" fn native_screen_key_pressed_pre(
    _screen: *core::ffi::c_void,
    key_name: &str,
    _repeat: bool,
    canceled: &mut bool,
) {
    if key_name == "key.keyboard.p" {
        let mut mgr = ServerManager::global().lock().unwrap();
        if let Some(first) = mgr.servers.first_mut() {
            first.pinned = !first.pinned;
            *canceled = true;
        }
    }
}
