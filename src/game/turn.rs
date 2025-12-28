use super::city_data::CityData;
use crate::game::city_graph::{CityGraph, Node as CityNode};
use crate::game::strategic_map::{ActivePlayer, BuildinTable, Player};
use crate::prelude::*;

#[derive(Event)]
pub struct TurnEnd;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(market_updater);
    app.add_observer(debt_collector);
}
#[derive(Event)]
pub struct GameEnd;

pub fn market_updater(
    ev: On<TurnEnd>,
    nodes: Query<&mut CityData>,
    building_table: Res<BuildinTable>,
) {
    println!("we ended the turn!!!!");
    for mut node in nodes {
        node.update_market(&building_table);
    }
}

pub fn debt_collector(
    ev: On<TurnEnd>,
    mut player: Single<&mut Player, With<ActivePlayer>>,
    mut commands: Commands,
) {
    if player.money < 0.0 {
        player.money = player.money * 1.1;
    }

    if player.money < -10000.0 {
        commands.trigger(GameEnd);
    }
}
