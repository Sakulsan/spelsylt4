use super::city_data::CityData;
use crate::game::city_graph::{CityGraph, Node as CityNode};
use crate::game::strategic_map::{BuildinTable, PlayerStats};
use crate::prelude::*;

#[derive(Event)]
pub struct TurnEnd;

#[derive(Event)]
pub struct GameEnd;

pub fn market_updater(ev: On<TurnEnd>, nodes: Query<&mut CityData>, building_table: Res<BuildinTable>) {
    println!("we ended the turn!!!!");
    for mut node in nodes {
        node.update_market(&building_table);
    }
}

pub fn caravan_updater(
    ev: On<TurnEnd>,
    mut player: ResMut<PlayerStats>,
    building_table: Res<BuildinTable>,
    graph: Res<CityGraph>,
    mut nodes: Query<(&CityNode, &mut CityData)>,
) {
    let PlayerStats { caravans, money } = &mut *player;
    println!("{:?}", caravans[0].orders);
    let _ = caravans.iter_mut().map(|x| x.update_orders(&graph, &mut nodes, &building_table, money));
}

pub fn debt_collector(ev: On<TurnEnd>, mut player: ResMut<PlayerStats>, mut commands: Commands) {
    if player.money < 0.0 {
        player.money = player.money * 1.1;
    }

    if player.money < 10000.0 {
        commands.trigger(GameEnd);
    }
}
