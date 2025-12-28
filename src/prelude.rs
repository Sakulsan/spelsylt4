pub use crate::assets::Sylt;
pub use crate::GlobalRng;

pub use bevy::prelude::*;
pub use rand::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapGenSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeGenSet;
