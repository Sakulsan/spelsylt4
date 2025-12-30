use std::{
    net::{IpAddr, SocketAddr, UdpSocket},
    process::CommandArgs,
    time::SystemTime,
};

use crate::{
    GlobalRngSeed, NetworkState,
    game::{
        city_data::CityData, namelists::CityNameList, strategic_hud::LockedCities,
        strategic_map::SelectedCity,
    },
    network::{
        message::{ClientData, ClientMessage, NetworkMessage, Players, ServerMessage},
        network_menu::{CityMenuEntered, CityMenuExited, CityUpdateReceived, NetworkMenuState},
    },
    prelude::*,
};
use bevy_renet::{
    netcode::{ClientAuthentication, NetcodeClientTransport},
    renet::{DefaultChannel, RenetClient},
};

pub type Reader = MessageReader<ServerMessage>;
pub type Writer = MessageWriter<ClientMessage>;

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

pub fn plugin(app: &mut App) {
    app.init_state::<ClientNetworkState>()
        .add_systems(
            Update,
            (
                send_message_system_client,
                receive_message_system_client,
                receive_city_updates,
                receive_city_menu_entered,
                receive_city_menu_exited,
            )
                .chain()
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
        .add_observer(squad_up)
        .add_observer(send_city_menu_entered)
        .add_observer(send_city_menu_exited);

    app.configure_sets(Update, ClientSet.run_if(in_state(NetworkState::Client)));
}

fn read_player_joined(mut messages: MessageReader<ServerMessage>, mut players: ResMut<Players>) {
    for message in messages.read() {
        if let NetworkMessage::Connected { player_id, .. } = **message {
            info!("read player joined: {player_id}");

            if players.0.contains(&player_id) {
                info!("we already have this player, skipping");
                continue;
            }

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
        if let NetworkMessage::Map { seed, city_names } = &**message {
            info!("Received seed from host, set seed to {seed}");
            my_city_names.0 = city_names.clone();
            rng.0 = *seed;
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

fn send_city_menu_entered(ev: On<CityMenuEntered>, mut writer: MessageWriter<ClientMessage>) {
    let text = NetworkMessage::CityViewing {
        player_id: ev.player,
        city_id: ev.city.clone(),
    };
    writer.write(ClientMessage(text));
}

fn receive_city_menu_entered(
    mut reader: MessageReader<ServerMessage>,
    mut locked_cities: ResMut<LockedCities>,
) {
    for msg in reader.read() {
        info!("reading messages from city_menu!!");
        let NetworkMessage::CityViewing {
            player_id: player_viewing,
            city_id: city_viewed,
        } = &**msg
        else {
            continue;
        };

        locked_cities.lock(*player_viewing, city_viewed);
    }
}

fn send_city_menu_exited(ev: On<CityMenuExited>, mut writer: MessageWriter<ClientMessage>) {
    let text = NetworkMessage::NotCityViewing {
        player_id: ev.player,
        city_id: ev.city.clone(),
    };
    writer.write(ClientMessage(text));
}

fn receive_city_menu_exited(
    mut reader: MessageReader<ServerMessage>,
    mut locked_cities: ResMut<LockedCities>,
) {
    for msg in reader.read() {
        let NetworkMessage::NotCityViewing {
            player_id: player_viewing,
            city_id: city_viewed,
        } = &**msg
        else {
            continue;
        };

        locked_cities.unlock(city_viewed);
    }
}

fn receive_city_updates(
    mut reader: MessageReader<ServerMessage>,
    mut cities: Query<&mut CityData>,
    mut selected_city: ResMut<SelectedCity>,
    mut commands: Commands,
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
