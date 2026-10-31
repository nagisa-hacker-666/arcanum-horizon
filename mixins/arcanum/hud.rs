// Runs from the game's HUD hook, after Horizon's own extras.
fn hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut crate::hud::HudBatch,
    corners: &mut crate::extras_hud::Corners,
) {
    crate::game::arcanum::on_hud(game, batch, corners);
}
