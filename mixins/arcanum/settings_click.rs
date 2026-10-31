// A button on Arcanum's settings page was clicked.
fn settings_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    crate::game::arcanum::on_settings_click(game, mod_id, button);
}
