use crate::prelude::*;
use crate::game::strategic_map::{BuildinTable};
use super::city_data::CityData;

#[derive(Event)]
pub struct TurnEnd;

pub fn market_updater(ev: On<TurnEnd>, nodes: Query<&mut CityData>, building_table: Res<BuildinTable>) {
    println!("we ended the turn!!!!");
    for mut node in nodes {
        node.update_market(&building_table);
    }
}
