fn not_enough_servers_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::multiplayer::native_not_enough_servers as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
