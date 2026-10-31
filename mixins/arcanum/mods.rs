// Options > Mods lists the native mods: adds Arcanum and its keys.
fn mods(game: &mut crate::game::Game<'_>, mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    crate::game::arcanum::on_mods(game, mods);
}
