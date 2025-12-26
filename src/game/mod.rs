//! The game's main screen states and transitions between them.

pub mod graph_test;
pub mod scene;
pub mod strategic_map;
pub mod namelists;
pub mod market;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((strategic_map::plugin, graph_test::plugin));
}
