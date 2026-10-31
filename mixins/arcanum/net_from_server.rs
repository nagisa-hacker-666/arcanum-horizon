// A mod message the server sent this window.
fn net_from_server(game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    crate::game::arcanum::on_net_from_server(game, channel, data);
}
