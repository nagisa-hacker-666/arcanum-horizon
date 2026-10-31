// Runs once when a world opens to players: adds Arcanum's commands.
fn register_commands(
    game: &mut crate::game::Game<'_>,
    commands: &mut Vec<crate::game::native_server::NativeCommand>,
) {
    crate::game::arcanum::on_register_commands(game, commands);
}
