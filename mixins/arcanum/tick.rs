// Runs once a tick in the window.
fn tick(game: &mut crate::game::Game<'_>) {
    crate::game::arcanum::on_tick(game);
}
