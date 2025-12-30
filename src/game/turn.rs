use super::city_data::CityData;
use crate::game::strategic_map::{ActivePlayer, BuildinTable, Player};
use crate::network::message::PlayerId;
use crate::prelude::*;

#[derive(Event)]
pub struct TurnEndSinglePlayer;

#[derive(Component, Copy, Clone, Debug)]
pub struct TurnEnded;

#[derive(Event)]
pub struct TurnEnd(PlayerId);

pub(super) fn plugin(app: &mut App) {
    app.add_observer(market_updater)
        .add_observer(debt_collector)
        .add_observer(debt_collector_server);
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
    mut player: Single<&mut Player>,
    mut commands: Commands,
) {
    println!("player has {} money", player.money);

    if player.money < 0.0 {
        player.money *= 1.02;
    }

    if player.money < -10000.0 {
        commands.trigger(GameEnd);
    }
}
