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
    GlobalRngSeed, NetworkState,
    game::{
        city_data::CityData,
        namelists::CityNameList,
        strategic_hud::LockedCities,
        strategic_map::{BelongsTo, Caravan, CaravanId, Player, SelectedCity},
    },
    network::{
        message::{ClientData, ClientMessage, NetworkMessage, PlayerId, Players, ServerMessage},
        network_menu::{CityMenuEntered, CityMenuExited, CityUpdateReceived, NetworkMenuState},
    },
    prelude::*,
};

pub type Reader<'a, 'b> = MessageReader<'a, 'b, ClientMessage>;
pub type Writer<'a> = MessageWriter<'a, ServerMessage>;

#[derive(Reflect, Resource, Default)]
pub struct ServerState {
    pub id_map: HashMap<u64, PlayerId>,
    pub next_id: PlayerId,
    pub ip: String,
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ServerSet;

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
        Err(_e) => {
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
        OnEnter(NetworkMenuState::Starting),
        broadcast_seed_and_start_before_mapgen.in_set(ServerSet),
    )
    .add_systems(
        Update,
        (
            send_message_system_server,
            receive_message_system_server,
            handle_events_system,
            broadcast_city_updates,
            broadcast_city_menu_entered,
            broadcast_city_menu_exited,
            read_caravan_requests,
            update_and_echo_caravan_edits,
        )
            .run_if(in_state(NetworkState::Host)),
    )
    .add_systems(
        PostUpdate,
        broadcast_created_caravan.run_if(in_state(NetworkState::Host)),
    )
    .add_observer(send_message_city_menu_entered)
    .add_observer(send_message_city_menu_exited);
}

fn server_config(mut commands: Commands) {
    let ip = match local_ip_address::local_ip() {
        Ok(ip) => ip,
        Err(_e) => {
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
    commands.insert_resource(Players(vec![0]));
}

fn handle_events_system(
    mut server_events: MessageReader<ServerEvent>,
    mut writer: MessageWriter<ServerMessage>,
    mut state: ResMut<ServerState>,
    mut players: ResMut<Players>,
) {
    for event in server_events.read() {
        // TODO: use client id
        match event {
            ServerEvent::ClientConnected { client_id } => {
                let player_id = state.add_player(*client_id);
                let existing_players = state.current_players();
                players.0.push(player_id);
                writer.write(ServerMessage(NetworkMessage::Connected {
                    player_id,
                    existing_players,
                }));
                info!("Client {client_id} connected");
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                // TODO: add disconnecting
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

        info!("Sending message: {msg:?}");

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
            info!("Received message: {message:?}");

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

// ------------------------
// GAME START FUNCTIONS
// ------------------------
fn broadcast_seed_and_start_before_mapgen(
    mut writer: MessageWriter<ServerMessage>,
    seed: Res<GlobalRngSeed>,
    city_names: ResMut<CityNameList>,
) {
    writer.write(ServerMessage(NetworkMessage::Map {
        seed: seed.0,
        city_names: city_names.0.clone(),
    }));
    writer.write(ServerMessage(NetworkMessage::GameStart));
}

// ------------------------
// GAME RUNNING FUNCTIONS
// ------------------------
fn broadcast_city_updates(
    mut reader: MessageReader<ClientMessage>,
    mut writer: MessageWriter<ServerMessage>,
    mut cities: Query<&mut CityData>,
    mut selected_city: ResMut<SelectedCity>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let upd @ NetworkMessage::CityUpdated { updated_city } = &**msg else {
            continue;
        };

        //if updated_city.id == selected_city.0.id {
        //    selected_city.0.overwrite(updated_city);
        //    commands.trigger(CityUpdateReceived);
        //}

        for mut city in cities.iter_mut() {
            if city.id == updated_city.id {
                *city = updated_city.clone();
            }
        }

        writer.write(ServerMessage(upd.clone()));
    }
}

fn send_message_city_menu_entered(
    ev: On<CityMenuEntered>,
    mut writer: MessageWriter<ServerMessage>,
) {
    let text = NetworkMessage::CityViewing {
        player_id: ev.player,
        city_id: ev.city.clone(),
    };
    writer.write(ServerMessage(text));
}

fn broadcast_city_menu_entered(
    mut reader: MessageReader<ClientMessage>,
    mut writer: MessageWriter<ServerMessage>,
    mut locked_cities: ResMut<LockedCities>,
) {
    for msg in reader.read() {
        let pass_on @ NetworkMessage::CityViewing {
            player_id: player_viewing,
            city_id: city_viewed,
        } = &**msg
        else {
            continue;
        };

        locked_cities.lock(*player_viewing, city_viewed);

        writer.write(ServerMessage(pass_on.clone()));
    }
}

fn send_message_city_menu_exited(ev: On<CityMenuExited>, mut writer: MessageWriter<ServerMessage>) {
    let text = NetworkMessage::NotCityViewing {
        player_id: ev.player,
        city_id: ev.city.clone(),
    };
    writer.write(ServerMessage(text));
}

fn broadcast_city_menu_exited(
    mut reader: Reader,
    mut writer: Writer,
    mut locked_cities: ResMut<LockedCities>,
) {
    for msg in reader.read() {
        let pass_on @ NetworkMessage::NotCityViewing {
            player_id: player_viewing,
            city_id: city_viewed,
        } = &**msg
        else {
            continue;
        };

        locked_cities.unlock(city_viewed);

        writer.write(ServerMessage(pass_on.clone()));
    }
}

fn read_caravan_requests(
    mut reader: Reader,
    mut commands: Commands,
    players: Query<(Entity, &Player)>,
) {
    for msg in reader.read() {
        let NetworkMessage::CaravanRequest { player_id, caravan } = &**msg else {
            continue;
        };

        let Some((ent, _)) = players.iter().find(|(e, p)| &p.player_id == player_id) else {
            error!("wtf");
            continue;
        };

        commands.spawn((caravan.clone(), BelongsTo(ent)));
    }
}

fn broadcast_created_caravan(
    mut writer: Writer,
    caravans: Query<(&Caravan, &CaravanId, &BelongsTo), Added<CaravanId>>,
    players: Query<&Player>,
) {
    for (caravan, caravan_id, pl_ent) in caravans {
        let player_id = players.get(pl_ent.0).expect("wtf").player_id;

        writer.write(ServerMessage(NetworkMessage::CaravanCreated {
            player_id,
            caravan_id: *caravan_id,
            caravan: caravan.clone(),
        }));
    }
}

fn update_and_echo_caravan_edits(
    mut reader: Reader,
    mut writer: Writer,
    mut caravans: Query<(&mut Caravan, &CaravanId), Added<CaravanId>>,
) {
    for msg in reader.read() {
        let msg @ NetworkMessage::CaravanUpdated {
            caravan_id,
            caravan,
        } = &**msg
        else {
            continue;
        };

        let Some((mut c, _)) = caravans.iter_mut().find(|(_, id)| id.0 == caravan_id.0) else {
            error!("wtf");
            continue;
        };

        *c = caravan.clone();

        writer.write(ServerMessage(msg.clone()));
    }
}
