use bevy::color::palettes::css::{CRIMSON, LIGHT_SLATE_GRAY};
use bevy_simple_text_input::{TextInput, TextInputValue};

use crate::{prelude::*, GameState};

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::NetworkMenu),
        (crate::kill_music, spawn_network_menu),
    )
    .add_systems(
        OnEnter(NetworkMenuState::Lobby),
        (lobby_menu_setup, update_players),
    )
    .add_systems(OnEnter(NetworkMenuState::Join), join_menu_setup)
    .init_state::<NetworkMenuState>() //Feels weird to have duplicate names, but it works
    .add_systems(
        Update,
        (button_hover_system, button_functionality).run_if(in_state(GameState::NetworkMenu)),
    );
}

// All actions that can be triggered from a button click
#[derive(Component)]
enum NetworkMenuButton {
    MainButton,
    HostButton,
    JoinButton,
    StartButton,
    ConnectToServerButton,
    QuitButton,
}

// State used for the current menu screen
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum NetworkMenuState {
    Main,
    Join,
    Lobby,
    #[default]
    Disabled,
}

fn spawn_network_menu(
    mut commands: Commands,
    mut sylt: Sylt,
    mut state: ResMut<NextState<NetworkMenuState>>,
) {
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
    let button_icon_node = Node {
        width: px(30),
        // This takes the icons out of the flexbox flow, to be positioned exactly
        position_type: PositionType::Absolute,
        // The icon will be close to the left border of the button
        left: px(10),
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
    interaction_query: Query<
        (&Interaction, &NetworkMenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut menu_state: ResMut<NextState<NetworkMenuState>>,
    ip_address_field: Option<Single<&TextInputValue, (With<IPField>, Changed<TextInputValue>)>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                NetworkMenuButton::MainButton => {
                    menu_state.set(NetworkMenuState::Main);
                }
                NetworkMenuButton::HostButton => {
                    //DO HOST CODE HERE
                    menu_state.set(NetworkMenuState::Lobby);
                }
                NetworkMenuButton::JoinButton => {
                    menu_state.set(NetworkMenuState::Join);
                }
                NetworkMenuButton::ConnectToServerButton => {
                    println!(
                        "Connecting to ip: {}",
                        &ip_address_field.as_ref().unwrap().0
                    );
                }
                NetworkMenuButton::StartButton => {
                    todo!()
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

fn lobby_menu_setup(mut commands: Commands) {
    let button_node = Node {
        width: px(200),
        height: px(65),
        margin: UiRect::all(px(20)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        flex_direction: FlexDirection::Column,
        ..default()
    };

    let button_text_style = (
        TextFont {
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

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
        BackgroundColor(LIGHT_SLATE_GRAY.into()),
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
            (Text::new("IP: 192.128......")),
            (Text::new("World seed: SEED")),
            (
                PlayerContainer,
                Node {
                    width: vw(100),
                    height: vh(70),
                    ..default()
                },
            )
        ],
    ));
}

fn update_players(mut commands: Commands, players_container: Query<Entity, With<PlayerContainer>>) {
    for container in players_container.iter() {
        let mut container = commands.get_entity(container).unwrap();

        container.despawn_children();
        container.with_children(|parent| {
            for player in vec!["Player one", "Player two"] {
                parent.spawn((
                    Node {
                        left: vw(10),
                        width: vw(80),
                        height: px(128),
                        ..default()
                    },
                    Text::new(player),
                ));
            }
        });
    }
}

#[derive(Component, Default)]
pub struct PlayerContainer;

#[derive(Component, Default)]
pub struct IPField;

fn join_menu_setup(mut commands: Commands) {
    let button_node = Node {
        width: px(200),
        height: px(65),
        margin: UiRect::all(px(20)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (
        TextFont {
            font_size: 33.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    commands.spawn((
        DespawnOnExit(NetworkMenuState::Join),
        Node {
            width: percent(100),
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
