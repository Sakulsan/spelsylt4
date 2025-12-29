use crate::{
    network::message::{ClientMessage, NetworkMessage, ServerMessage},
    prelude::*,
};
use bevy_renet::renet::{DefaultChannel, RenetClient};

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ClientSet;

#[derive(Resource)]
struct ClientData {
    player_id: usize,
}

#[derive(States, Debug, Clone, Copy, Hash, Eq, PartialEq, Default)]
enum ClientNetworkState {
    #[default]
    AwaitingId,
    WaitingForSeed,
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (send_message_system_client, receive_message_system_client).in_set(ClientSet),
    );

    app.configure_sets(Update, ClientSet.run_if(resource_exists::<RenetClient>));
}

fn send_message_system_client(
    mut client: ResMut<RenetClient>,
    mut reader: MessageReader<ClientMessage>,
) {
    let channel_id = 0;

    for message in reader.read() {
        let msg = &message.0;

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
