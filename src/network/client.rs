use std::{
    net::{IpAddr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use crate::{
    game::city_data::CityData,
    network::{
        message::{ClientData, ClientMessage, NetworkMessage, Players, ServerMessage},
        network_menu::NetworkMenuState,
    },
    prelude::*,
    GlobalRngSeed, NetworkState,
};
use bevy_renet::{
    netcode::{ClientAuthentication, NetcodeClientTransport},
    renet::{DefaultChannel, RenetClient},
};

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ClientSet;

#[derive(Reflect, States, Debug, Clone, Copy, Hash, Eq, PartialEq, Default)]
pub enum ClientNetworkState {
    #[default]
    AwaitingId,
    AwaitingStart,
    Started,
}

#[derive(Event)]
pub struct JoinEvent(pub String);

fn squad_up(
    join: On<JoinEvent>,
    mut commands: Commands,
    mut net: ResMut<NextState<NetworkState>>,
    mut menu_state: ResMut<NextState<NetworkMenuState>>,
) {
    net.set(NetworkState::Client);

    let local_ip = match local_ip_address::local_ip() {
        Ok(ip) => ip,
        Err(e) => {
            error!("Server failed to start: couldn't get local IP address {e}");
            return;
        }
    };

    let client_id = match local_ip {
        IpAddr::V4(ip) => ip.to_bits() as u64,
        _ => {
            error!("this psychopath only has an ipv6 address");
            return;
        }
    };

    let authentication = ClientAuthentication::Unsecure {
        server_addr: SocketAddr::new(join.0.parse().unwrap(), 5000),
        client_id,
        user_data: None,
        protocol_id: 0,
    };

    let socket = UdpSocket::bind(SocketAddr::new(local_ip, 5000)).unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    info!("set up client on ip {}", local_ip);
    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    commands.insert_resource(transport);
    menu_state.set(NetworkMenuState::Lobby);
}

#[derive(Reflect, Resource, Default)]
pub struct CityNameList(pub Vec<Vec<String>>);

pub fn plugin(app: &mut App) {
    app.init_state::<ClientNetworkState>()
        .add_systems(
            Update,
            (
                send_message_system_client,
                receive_message_system_client,
                receive_city_updates,
            )
                .in_set(ClientSet),
        )
        .add_systems(
            Update,
            (
                read_player_joined.run_if(in_state(ClientNetworkState::AwaitingStart)),
                await_id.run_if(in_state(ClientNetworkState::AwaitingId)),
                await_map.run_if(not(in_state(ClientNetworkState::Started))),
                await_start.run_if(not(in_state(ClientNetworkState::Started))),
            )
                .chain()
                .in_set(ClientSet),
        )
        .add_observer(squad_up);

    app.configure_sets(Update, ClientSet.run_if(in_state(NetworkState::Client)));
}

fn read_player_joined(mut messages: MessageReader<ServerMessage>, mut players: ResMut<Players>) {
    for message in messages.read() {
        if let NetworkMessage::Connected { player_id, .. } = **message {
            info!("read player joined: {player_id}");
            players.0.push(player_id);
        }
    }
}

fn await_start(
    mut messages: MessageReader<ServerMessage>,
    mut state: ResMut<NextState<ClientNetworkState>>,
    mut menu_state: ResMut<NextState<NetworkMenuState>>,
) {
    for message in messages.read() {
        if let NetworkMessage::GameStart = **message {
            info!("Starting the game");
            state.set(ClientNetworkState::Started);
            menu_state.set(NetworkMenuState::Starting);
        }
    }
}

fn await_id(
    mut commands: Commands,
    mut messages: MessageReader<ServerMessage>,
    mut state: ResMut<NextState<ClientNetworkState>>,
) {
    for message in messages.read() {
        if let NetworkMessage::Connected {
            player_id,
            ref existing_players,
        } = **message
        {
            info!("Received id from host, set own id to {player_id}");
            commands.insert_resource(ClientData { player_id });
            commands.insert_resource(Players(existing_players.clone()));
            state.set(ClientNetworkState::AwaitingStart);
        }
    }
}

fn await_map(
    mut messages: MessageReader<ServerMessage>,
    mut state: ResMut<NextState<ClientNetworkState>>,
    mut rng: ResMut<GlobalRngSeed>,
    mut my_city_names: ResMut<CityNameList>,
) {
    for message in messages.read() {
        if let NetworkMessage::Map { seed, city_names } = **message {
            info!("Received seed from host, set seed to {seed}");
            my_city_names = city_names;
            rng.0 = seed;
            state.set(ClientNetworkState::AwaitingStart);
        }
    }
}

fn send_message_system_client(
    mut client: ResMut<RenetClient>,
    mut reader: MessageReader<ClientMessage>,
) {
    for message in reader.read() {
        let msg = &message.0;
        info!("Sending message: {msg:?}");

        let serialized = serde_json::to_string(msg).unwrap();

        client.send_message(DefaultChannel::ReliableOrdered, serialized);
    }
    // Send a text message for all clients
    // The enum DefaultChannel describe the channels used by the default configuration
}

fn receive_message_system_client(
    mut client: ResMut<RenetClient>,
    mut writer: MessageWriter<ServerMessage>,
) {
    // Receive message from all clients
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        info!("Received message: {message:?}");

        let text: NetworkMessage = match serde_json::from_slice(&message) {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to parse message: {:?}", e);
                continue;
            }
        };

        writer.write(ServerMessage(text));
    }
}

fn receive_city_updates(
    mut reader: MessageReader<ClientMessage>,
    mut cities: Query<&mut CityData>,
) {
    for msg in reader.read() {
        let NetworkMessage::CityUpdated { updated_city } = &**msg else {
            continue;
        };

        for mut city in cities.iter_mut() {
            if city.id == updated_city.id {
                *city = updated_city.clone();
            }
        }
    }
}
