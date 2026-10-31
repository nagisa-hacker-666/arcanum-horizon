// A player died: the mana and the resonance go, the progression stays.
fn player_died(game: &mut crate::game::Game<'_>, player: u128, cause: Option<&str>) {
    crate::game::arcanum::on_player_died(game, player, cause);
}
