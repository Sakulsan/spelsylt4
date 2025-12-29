//! Everything about networking

pub mod client;
pub mod message;
pub mod network_menu;
pub mod server;
use bevy::prelude::*;

use crate::game::turn::*;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NetworkingSet;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((network_menu::plugin));
}
