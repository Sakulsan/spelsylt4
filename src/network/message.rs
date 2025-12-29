use crate::{game::strategic_map::Caravan, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Message)]
pub struct ClientMessage(pub NetworkMessage);

#[derive(Message)]
pub struct ServerMessage(pub NetworkMessage);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetworkMessage {
    Connected {
        player_id: usize,
    },
    Map {
        seed: u64,
    },
    TurnEnded {
        player_id: usize,
        caravans: Vec<Caravan>,
    },
    TurnFinished {
        caravans: Vec<Caravan>,
    },
}
