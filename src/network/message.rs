use std::collections::HashMap;

use crate::{
    game::{
        city_data::CityData,
        strategic_map::{Caravan, CaravanId},
    },
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
        money: f64,
    },
    CityUpdated {
        updated_city: CityData,
    },
    CaravanRequest {
        player_id: PlayerId,
        caravan: Caravan,
    },
    CaravanCreated {
        player_id: PlayerId,
        caravan_id: CaravanId,
        caravan: Caravan,
    },
    CaravanUpdated {
        caravan_id: CaravanId,
        caravan: Caravan,
    },
    TurnFinished {
        caravans: Vec<(CaravanId, Caravan)>,
        economy: HashMap<PlayerId, f64>,
    },
    CityViewing {
        player_id: PlayerId,
        city_id: String,
    },
    NotCityViewing {
        player_id: PlayerId,
        city_id: String,
    },
}

#[derive(Resource)]
pub struct ClientData {
    pub player_id: PlayerId,
}

#[derive(Resource, Default)]
pub struct Players(pub Vec<PlayerId>);

pub type PlayerId = u64;
