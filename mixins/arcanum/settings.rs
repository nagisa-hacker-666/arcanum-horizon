// The buttons of Arcanum's settings page, asked each time the page opens.
fn settings(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    crate::game::arcanum::on_settings(game, mod_id, buttons);
}
