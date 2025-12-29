//! Everything about networking

pub mod client;
pub mod message;
pub mod network_menu;
pub mod server;
use bevy::prelude::*;

use message::{ClientData, Players};
use network_menu::NetworkMenuState;

use crate::{
    game::strategic_map::{ActivePlayer, Player},
    GameState,
};

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NetworkingSet;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((network_menu::plugin, client::plugin, server::plugin))
        .add_message::<message::ClientMessage>()
        .add_message::<message::ServerMessage>();

    app.add_systems(
        OnEnter(NetworkMenuState::Starting),
        (spawn_players, start_game)
            .after(client::ClientSet)
            .after(server::ServerSet)
            .chain(),
    );
}

fn spawn_players(mut commands: Commands, players: Res<Players>, client_data: Res<ClientData>) {
    for &player_id in &players.0 {
        let mut cmd = commands.spawn(Player {
            player_id,
            money: 5000.0,
        });
        if player_id == client_data.player_id {
            cmd.insert(ActivePlayer);
        }
    }
}

fn start_game(mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::Game);
}
