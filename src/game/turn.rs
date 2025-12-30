use std::collections::HashMap;

use super::city_data::CityData;
use crate::NetworkState;
use crate::game::strategic_hud::LockedCities;
use crate::game::strategic_map::{
    ActivePlayer, BuildinTable, Caravan, CaravanId, HostFixedTurnEnd, Player,
};
use crate::network::message::{PlayerId, ServerMessage};
use crate::prelude::*;

#[derive(Resource, Default, Deref, DerefMut)]
struct Turn(u64);

#[derive(Event)]
pub struct TurnEndSinglePlayer;

#[derive(Component, Copy, Clone, Debug)]
pub struct TurnEnded;

#[derive(Event, Debug)]
pub struct TurnEnd(pub PlayerId);

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Turn>()
        .add_systems(
            Update,
            every_turn_ended.run_if(in_state(NetworkState::Host)),
        )
        .add_systems(
            PreUpdate,
            send_turn_update.run_if(in_state(NetworkState::Host).and(resource_changed::<Turn>)),
        )
        .add_observer(market_updater)
        .add_observer(debt_collector)
        .add_observer(update_turnend)
        .add_observer(|_: On<TurnEndSinglePlayer>, mut turn: ResMut<Turn>| **turn += 1)
        .add_observer(client::update_turnend)
        .add_observer(server::update_turnend);
}

fn send_turn_update(
    mut commands: Commands,
    mut writer: crate::network::server::Writer,
    caravans: Query<(&CaravanId, &Caravan)>,
    players: Query<(Entity, &Player)>,
    mut locked_cities: ResMut<LockedCities>,
) {
    locked_cities.clear();
    let caravans: Vec<_> = caravans
        .into_iter()
        .map(|(id, c)| (id.clone(), c.clone()))
        .collect();
    let mut economy = HashMap::new();
    for (ent, player) in players {
        economy.insert(player.player_id, player.money);
        commands.entity(ent).remove::<TurnEnded>();
    }

    writer.write(ServerMessage(
        crate::network::message::NetworkMessage::TurnFinished { caravans, economy },
    ));

    commands.trigger(HostFixedTurnEnd);
}

fn every_turn_ended(mut commands: Commands, players: Query<(&Player, Option<&TurnEnded>)>) {
    if players.iter().count() == 0 {
        return;
    }
    if players.iter().all(|(_, p)| p.is_some()) {
        commands.trigger(TurnEndSinglePlayer);
    }
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
