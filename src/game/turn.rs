use crate::game::city_graph::{CityGraph, Node as CityNode};
use crate::prelude::*;
use crate::game::strategic_map::{BuildinTable, PlayerStats};
use super::city_data::CityData;

#[derive(Event)]
pub struct TurnEnd;

pub fn market_updater(ev: On<TurnEnd>, nodes: Query<&mut CityData>, building_table: Res<BuildinTable>) {
    println!("we ended the turn!!!!");
    for mut node in nodes {
        node.update_market(&building_table);
    }
}

pub fn caravan_updater(ev: On<TurnEnd>, mut player: ResMut<PlayerStats>, building_table: Res<BuildinTable>, graph: Res<CityGraph>, mut nodes: Query<(&CityNode, &mut CityData)>) {
    let PlayerStats { caravans, money} = &mut *player;
    println!("{:?}", caravans[0].orders);
    let _ = caravans.iter_mut().map(|x| x.update_orders(&graph, &mut nodes, &building_table, money));
}