// A player's command on the server, before Horizon's own tree sees it.
fn perform_command(
    game: &mut crate::game::Game<'_>,
    player: &crate::game::native_server::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    crate::game::arcanum::on_perform_command(game, player, command, ran);
}
