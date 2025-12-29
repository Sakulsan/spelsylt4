use std::collections::HashMap;

use crate::{
    game::{city_data::CityData, strategic_map::Caravan},
    prelude::*,
};
use serde::{Deserialize, Serialize};

#[derive(Message, Deref, DerefMut)]
pub struct ClientMessage(pub NetworkMessage);

#[derive(Message, Deref, DerefMut, Clone)]
pub struct ServerMessage(pub NetworkMessage);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetworkMessage {
    Connected {
        player_id: PlayerId,
        existing_players: Vec<PlayerId>,
    },
    Map {
        seed: u64,
        city_names: Vec<Vec<String>>,
    },
    GameStart,
    TurnEnded {
        player_id: PlayerId,
        caravans: Vec<Caravan>,
    },
    CityUpdated {
        updated_city: CityData,
    },
    NewCaravan {
        player_id: PlayerId,
    },
    TurnFinished {
        caravans: Vec<Caravan>,
        economy: HashMap<PlayerId, isize>,
    },
}

#[derive(Resource)]
pub struct ClientData {
    pub player_id: PlayerId,
}

#[derive(Resource, Default)]
pub struct Players(pub Vec<PlayerId>);

pub type PlayerId = u64;
