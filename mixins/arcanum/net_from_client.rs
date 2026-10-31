// A mod message a player's client sent, with the hub back in the game.
fn net_from_client(game: &mut crate::game::Game<'_>, player: u128, channel: &str, data: &[u8]) {
    crate::game::arcanum::on_net_from_client(game, player, channel, data);
}
