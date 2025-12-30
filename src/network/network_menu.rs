use bevy::color::palettes::css::{CRIMSON, LIGHT_SLATE_GRAY};
use bevy_renet::netcode::{NetcodeClientPlugin, NetcodeServerPlugin};
use bevy_renet::renet::RenetClient;
use bevy_renet::{renet::ConnectionConfig, RenetClientPlugin, RenetServerPlugin};
use bevy_simple_text_input::{TextInput, TextInputValue};

use crate::network::client::JoinEvent;
use crate::network::message::{PlayerId, Players};
use crate::network::server::ServerState;
use crate::{prelude::*, GameState};

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct NetworkMenuLabel;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        RenetServerPlugin,
        RenetClientPlugin,
        NetcodeServerPlugin,
        NetcodeClientPlugin,
    ))
    .init_resource::<Players>()
    .init_state::<NetworkMenuState>() //Feels weird to have duplicate names, but it works
    .add_systems(OnEnter(GameState::NetworkMenu), spawn_network_menu)
    .add_systems(OnEnter(NetworkMenuState::Main), spawn_network_menu)
    .add_systems(
        Update,
        (
            update_players.run_if(resource_changed::<Players>),
            update_lobby_ip,
            button_functionality,
            button_hover_system.run_if(in_state(GameState::NetworkMenu)),
        )
            .in_set(NetworkMenuLabel),
    )
    .add_systems(
        OnEnter(NetworkMenuState::Lobby),
        (lobby_menu_setup, update_players).chain(),
    )
    .add_systems(
        Update,
        update_players.run_if(resource_changed::<Players>.and(in_state(GameState::NetworkMenu))),
    )
    .add_systems(OnEnter(NetworkMenuState::Join), join_menu_setup)
    .configure_sets(
        Update,
        NetworkMenuLabel.run_if(not(in_state(NetworkMenuState::Disabled))),
    );
}

// All actions that can be triggered from a button click
#[derive(Component)]
pub enum NetworkMenuButton {
    MainButton,
    HostButton,
    JoinButton,
    StartButton,
    ConnectToServerButton,
    QuitButton,
}

// State used for the current menu screen
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum NetworkMenuState {
    Main,
    Join,
    Lobby,
    Starting,
    #[default]
    Disabled,
}

#[derive(Event)]
pub struct CityUpdateReceived;

#[derive(Event)]
pub struct CityMenuEntered {
    pub player: PlayerId,
    pub city: String,
}

#[derive(Event)]
pub struct CityMenuExited {
    pub player: PlayerId,
    pub city: String,
}

fn spawn_network_menu(mut commands: Commands, mut state: ResMut<NextState<NetworkMenuState>>) {
    state.set(NetworkMenuState::Main);

    commands.spawn((
        //AudioPlayer::new(asset_server.load("music/Bellsachiming.ogg")),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            ..default()
        },
    ));

    // Common style for all buttons on the screen
    let button_node = Node {
        width: px(300),
        height: px(65),
        margin: UiRect::all(px(20)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_font = TextFont {
        font_size: 33.0,
        ..default()
    };

    commands.spawn((
        DespawnOnExit(NetworkMenuState::Main),
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(CRIMSON.into()),
            children![
                // Display the game name
                (
                    Text::new("Multiplayer LAN"),
                    TextFont {
                        font_size: 67.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    Node {
                        margin: UiRect::all(px(50)),
                        ..default()
                    },
                ),
                // Display three buttons for each action available from the main menu:
                // - new game
                // - settings
                // - quit
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    NetworkMenuButton::HostButton,
                    children![
                        //(ImageNode::new(right_icon), button_icon_node.clone()),
                        (
                            Text::new("Host server"),
                            button_text_font.clone(),
                            TextColor(TEXT_COLOR),
                        ),
                    ]
                ),
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    NetworkMenuButton::JoinButton,
                    children![
                        //                            (ImageNode::new(right_icon), button_icon_node.clone()),
                        (
                            Text::new("Join server"),
                            button_text_font.clone(),
                            TextColor(TEXT_COLOR),
                        ),
                    ]
                ),
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    NetworkMenuButton::QuitButton,
                    children![(
                        Text::new("Return"),
                        button_text_font.clone(),
                        TextColor(TEXT_COLOR),
                    ),]
                ),
            ]
        )],
    ));
}

// Button hover system
fn button_hover_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color) in &mut interaction_query {
        *background_color = match *interaction {
            Interaction::Pressed => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_PRESSED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

fn button_functionality(
    mut commands: Commands,
    interaction_query: Query<
        (&Interaction, &NetworkMenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut loading_container: Query<&mut Visibility, With<LoadingContainer>>,
    mut menu_state: ResMut<NextState<NetworkMenuState>>,
    ip_address_field: Option<Single<&TextInputValue, With<IPField>>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut network_state: ResMut<NextState<NetworkState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                NetworkMenuButton::MainButton => {
                    menu_state.set(NetworkMenuState::Main);
                }
                NetworkMenuButton::HostButton => {
                    network_state.set(NetworkState::Host);
                    menu_state.set(NetworkMenuState::Lobby);
                }
                NetworkMenuButton::JoinButton => {
                    menu_state.set(NetworkMenuState::Join);
                }
                NetworkMenuButton::ConnectToServerButton => {
                    for mut loading_box in loading_container.iter_mut() {
                        *loading_box = Visibility::Visible;
                    }

                    commands.trigger(JoinEvent(ip_address_field.as_ref().unwrap().0.to_string()));
                    info!("Connecting to ip: {}", ip_address_field.as_ref().unwrap().0);
                }
                NetworkMenuButton::StartButton => {
                    menu_state.set(NetworkMenuState::Starting);
                }
                NetworkMenuButton::QuitButton => {
                    println!("Lol");
                    menu_state.set(NetworkMenuState::Disabled);
                    game_state.set(GameState::Menu);
                }
            }
        }
    }
}

use crate::NetworkState;

#[derive(Component, Default)]
pub struct LobbyNode;
fn lobby_menu_setup(mut commands: Commands, network_state: Res<State<NetworkState>>) {
    let button_node = Node {
        width: px(200),
        height: px(65),
        margin: UiRect::all(px(20)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        flex_direction: FlexDirection::Column,
        ..default()
    };

    commands.spawn((
        DespawnOnExit(NetworkMenuState::Lobby),
        Node {
            width: vw(100),
            height: vh(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        //BackgroundColor(LIGHT_SLATE_GRAY.into()),
        //OnSettingsMenuScreen,
        children![
            (
                Text::new("Lobby"),
                Node {
                    width: vw(100),
                    height: vh(20),
                    ..default()
                },
                BackgroundColor(CRIMSON.into()),
            ),
            (
                Text::new("IP: Unkown"),
                IPField,
                if *network_state == NetworkState::Host {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
            ),
            (
                PlayerContainer,
                Node {
                    width: vw(100),
                    height: vh(60),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ),
            (
                Text::new("Start game"),
                Button,
                NetworkMenuButton::StartButton,
                BackgroundColor(NORMAL_BUTTON),
                if *network_state == NetworkState::Host {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
                button_node.clone()
            )
        ],
    ));
}

fn update_players(
    mut commands: Commands,
    players_container: Query<Entity, With<PlayerContainer>>,
    players: Option<Res<Players>>,
    mut sylt: Sylt,
) {
    let Some(players) = players else {
        return;
    };

    for container in players_container.iter() {
        let mut container = commands.get_entity(container).unwrap();

        container.despawn_children();
        container.with_children(|parent| {
            for player in &players.0 {
                parent.spawn((
                    Node {
                        width: vw(60),
                        height: px(96),
                        margin: UiRect {
                            bottom: px(8),
                            ..default()
                        },
                        ..default()
                    },
                    BackgroundColor(Srgba::new(0.0, 0.0, 0.0, 0.2).into()),
                    children![
                        (
                            Node {
                                top: px(16),
                                left: px(16),
                                width: px(64),
                                height: px(64),
                                ..default()
                            },
                            ImageNode {
                                image: sylt.get_image(match player {
                                    0 => {
                                        "player_red"
                                    }
                                    1 => {
                                        "player_blue"
                                    }
                                    2 => {
                                        "player_green"
                                    }
                                    3 => {
                                        "player_yellow"
                                    }
                                    _ => {
                                        "player_purple"
                                    }
                                }),
                                ..default()
                            }
                        ),
                        (
                            Node {
                                top: px(32),
                                left: px(64),
                                ..default()
                            },
                            Text::new(format!("Player: {}", player))
                        )
                    ],
                ));
            }
        });
    }
}

#[derive(Component, Default)]
pub struct PlayerContainer;

#[derive(Component, Default)]
pub struct IPField;

#[derive(Component, Default)]
pub struct LoadingContainer;

fn join_menu_setup(mut commands: Commands) {
    let button_node = Node {
        width: px(200),
        height: px(65),
        margin: UiRect::all(px(20)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let _button_text_style = (
        TextFont {
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    let client = RenetClient::new(ConnectionConfig::default());
    commands.insert_resource(client);

    commands.spawn((
        DespawnOnExit(NetworkMenuState::Join),
        Node {
            left: vw(25),
            width: vw(50),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        //OnSettingsMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                width: vw(50),
                ..default()
            },
            BackgroundColor(CRIMSON.into()),
            children![
                (
                    IPField,
                    TextInput,
                    Node {
                        padding: UiRect::all(Val::Px(5.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor::all(Color::BLACK),
                ),
                (
                    LoadingContainer,
                    button_node.clone(),
                    Visibility::Hidden,
                    Text::new("Connecting...")
                ),
                (
                    Button,
                    NetworkMenuButton::ConnectToServerButton,
                    BorderColor::all(Color::BLACK),
                    Text::new("Join server"),
                    button_node.clone()
                ),
                (
                    Button,
                    NetworkMenuButton::MainButton,
                    BorderColor::all(Color::BLACK),
                    Text::new("Return"),
                    button_node.clone()
                )
            ]
        ),],
    ));
}

fn update_lobby_ip(
    mut commands: Commands,
    srv: Option<Res<ServerState>>,
    field: Query<Entity, With<IPField>>,
) {
    let Some(srv) = srv else {
        return;
    };

    let Ok(field) = field.single() else {
        return;
    };

    commands
        .entity(field)
        .insert(Text(format!("Hosting server on: {}", srv.ip)));
}
