//! The game's main screen states and transitions between them.

pub mod city_graph;
pub mod market;
pub mod namelists;
pub mod scene;
pub mod strategic_hud;
pub mod strategic_map;
use bevy::prelude::*;

use crate::game::turn::*;
pub mod city_data;
pub mod tooltip;
pub mod turn;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        strategic_map::plugin,
        city_graph::plugin,
        strategic_hud::plugin,
        tooltip::plugin,
    ));
    app.add_observer(market_updater);
    app.add_observer(caravan_updater);
    app.add_observer(debt_collector);
}
