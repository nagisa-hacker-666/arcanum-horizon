// Runs on every key or mouse button change, by vanilla name.
fn key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    crate::game::arcanum::on_key(game, key, pressed);
}
