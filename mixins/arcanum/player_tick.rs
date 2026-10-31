// Runs once a tick for every living player, on the authoritative server.
fn player_tick(game: &mut crate::game::Game<'_>) {
    crate::game::arcanum::on_player_tick(game);
}
