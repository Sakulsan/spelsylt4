//! Everything about networking

pub mod network_menu;
pub mod server;
pub mod client;
pub mod message;
use bevy::prelude::*;

use crate::game::turn::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((network_menu::plugin));
}
