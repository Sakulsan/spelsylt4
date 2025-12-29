use std::collections::HashMap;

use bevy_renet::renet::{DefaultChannel, RenetServer, ServerEvent};

use crate::{
    network::message::{ClientMessage, NetworkMessage, PlayerId, ServerMessage},
    prelude::*,
};

type ClientId = u64;

#[derive(Event)]
pub struct ServerHosted;

#[derive(Resource, Default)]
struct ServerState {
    id_map: HashMap<u64, PlayerId>,
    next_id: PlayerId,
}

impl ServerState {
    pub fn add_player(&mut self, client_id: u64) -> PlayerId {
        let player_id = self.next_id;
        self.id_map.insert(client_id, player_id);
        self.next_id += 1;
        player_id
    }

    pub fn current_players(&self) -> Vec<PlayerId> {
        self.id_map.values().cloned().collect()
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (send_message_system_server, receive_message_system_server)
            .run_if(resource_exists::<RenetServer>),
    )
    .add_systems(Update, handle_events_system)
    .add_observer(on_host);
}

fn on_host(_: On<ServerHosted>, mut commands: Commands) {
    commands.insert_resource(ServerState {
        id_map: HashMap::from([(0, 0)]),
        next_id: 1,
    });
}

fn handle_events_system(
    mut server_events: MessageReader<ServerEvent>,
    mut writer: MessageWriter<ServerMessage>,
    mut state: ResMut<ServerState>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                let player_id = state.add_player(*client_id);
                let existing_players = state.current_players();
                writer.write(ServerMessage(NetworkMessage::Connected {
                    player_id,
                    existing_players,
                }));
                info!("Client {client_id} connected");
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Client {client_id} disconnected: {reason}");
            }
        }
    }
}

fn send_message_system_server(
    mut server: ResMut<RenetServer>,
    mut reader: MessageReader<ServerMessage>,
) {
    for message in reader.read() {
        let msg = &message.0;

        let serialized = serde_json::to_string(msg).unwrap();

        server.broadcast_message(DefaultChannel::ReliableOrdered, serialized);
    }
    // Send a text message for all clients
    // The enum DefaultChannel describe the channels used by the default configuration
}

fn receive_message_system_server(
    mut server: ResMut<RenetServer>,
    mut writer: MessageWriter<ClientMessage>,
) {
    // Receive message from all clients
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered)
        {
            let text: NetworkMessage = match serde_json::from_slice(&message) {
                Ok(m) => m,
                Err(e) => {
                    error!("Failed to parse message: {:?}", e);
                    continue;
                }
            };

            writer.write(ClientMessage(text));
        }
    }
}
