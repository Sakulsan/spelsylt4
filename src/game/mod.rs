//! The game's main screen states and transitions between them.

pub mod scene;
pub mod strategic_map;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(strategic_map::plugin);
}
