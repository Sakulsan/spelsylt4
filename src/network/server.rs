use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy_renet::{
    netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    renet::{ConnectionConfig, DefaultChannel, RenetServer, ServerEvent},
};

use crate::{
    network::message::{ClientData, ClientMessage, NetworkMessage, PlayerId, ServerMessage},
    prelude::*,
    NetworkState,
};

type ClientId = u64;

#[derive(Event)]
pub struct ServerHosted;

#[derive(Resource, Default)]
pub struct ServerState {
    pub id_map: HashMap<u64, PlayerId>,
    pub next_id: PlayerId,
    pub ip: String,
}

#[derive(Event)]
pub struct GameStarted;

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

fn host_server(mut commands: Commands) {
    let server = RenetServer::new(ConnectionConfig::default());
    commands.insert_resource(server);

    let server = RenetServer::new(ConnectionConfig::default());
    commands.insert_resource(server);

    let local_ip = match local_ip_address::local_ip() {
        Ok(ip) => ip,
        Err(e) => {
            error!("Server failed to start: couldn't get local IP address");
            return;
        }
    };

    let server_addr = SocketAddr::new(local_ip, 5000);

    let socket = UdpSocket::bind(server_addr).unwrap();
    let server_config = ServerConfig {
        current_time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        max_clients: 64,
        protocol_id: 0,
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Unsecure,
    };
    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    commands.insert_resource(transport);
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(NetworkState::Host),
        (host_server, server_config).chain(),
    )
    .add_systems(
        Update,
        (
            send_message_system_server,
            receive_message_system_server,
            handle_events_system,
        )
            .run_if(resource_exists::<RenetServer>),
    );
}

fn server_config(mut commands: Commands) {
    let ip = match local_ip_address::local_ip() {
        Ok(ip) => ip,
        Err(e) => {
            error!("Server failed to start: couldn't get local IP address");
            "127.0.0.1".parse().unwrap()
        }
    };

    commands.insert_resource(ServerState {
        id_map: HashMap::from([(0, 0)]),
        next_id: 1,
        ip: ip.to_string(),
    });
    commands.insert_resource(ClientData { player_id: 0 });
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
