use super::city_data::CityData;
use crate::NetworkState;
use crate::game::strategic_map::{ActivePlayer, BuildinTable, Player};
use crate::network::message::PlayerId;
use crate::prelude::*;

#[derive(Event)]
pub struct TurnEndSinglePlayer;

#[derive(Component, Copy, Clone, Debug)]
pub struct TurnEnded;

#[derive(Event, Debug)]
pub struct TurnEnd(pub PlayerId);

pub(super) fn plugin(app: &mut App) {
    app.add_observer(market_updater)
        .add_observer(debt_collector)
        .add_observer(update_turnend)
        .add_observer(client::update_turnend)
        .add_observer(server::update_turnend);
}

fn update_turnend(player: On<TurnEnd>, mut commands: Commands, players: Query<(Entity, &Player)>) {
    info!("Ending turn for {}", player.0);

    let (player, _) = players
        .iter()
        .find(|(_, p)| p.player_id == player.0)
        .expect("no player :(");

    commands.entity(player).insert(TurnEnded);
}

mod client {
    use crate::network::message::ClientMessage;

    use super::*;

    pub fn update_turnend(
        player: On<TurnEnd>,
        mut writer: crate::network::client::Writer,
        you: Query<&Player, With<ActivePlayer>>,
        network_state: Res<State<NetworkState>>,
    ) {
        if *network_state != NetworkState::Client {
            return;
        }

        let you = you.single().unwrap();
        if player.0 == you.player_id {
            writer.write(ClientMessage(
                crate::network::message::NetworkMessage::TurnEnded {
                    player_id: you.player_id,
                    money: you.money,
                },
            ));
        }
    }
}

mod server {
    use crate::network::message::ServerMessage;

    use super::*;

    pub fn update_turnend(
        player: On<TurnEnd>,
        mut writer: crate::network::server::Writer,
        you: Query<&Player, With<ActivePlayer>>,
        network_state: Res<State<NetworkState>>,
    ) {
        if *network_state != NetworkState::Host {
            return;
        }

        let you = you.single().unwrap();
        if player.0 == you.player_id {
            writer.write(ServerMessage(
                crate::network::message::NetworkMessage::TurnEnded {
                    player_id: you.player_id,
                    money: you.money,
                },
            ));
        }
    }
}

#[derive(Event)]
pub struct GameEnd;

pub fn market_updater(
    _ev: On<TurnEndSinglePlayer>,
    nodes: Query<&mut CityData>,
    building_table: Res<BuildinTable>,
    mut players: Query<&mut Player>,
) {
    println!("we ended the turn!!!!");
    for mut node in nodes {
        node.update_market(&building_table, &mut players);
    }
}

pub fn debt_collector(
    _ev: On<TurnEndSinglePlayer>,
    mut players: Query<&mut Player>,
    mut commands: Commands,
) {
    for mut player in players.iter_mut() {
        println!("player has {} money", player.money);
        if player.money < 0.0 {
            player.money *= 1.02;
        }

        if player.money < -10000.0 {
            commands.trigger(GameEnd);
        }
    }
}
