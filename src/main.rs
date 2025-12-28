//! This example will display a simple menu using Bevy UI where you can start a new game,
//! change some settings or quit. There is no actual game, it will just display the current
//! settings for 5 seconds before going back to the menu.

use crate::game::city_data::CityData;
use crate::game::namelists::*;
use crate::game::strategic_map::{CityImageMarker, CityNodeMarker};
use bevy::feathers::FeathersPlugins;
use bevy::prelude::*;
use bevy_simple_text_input::TextInputPlugin;
use bevy_ui_anchor::AnchorUiPlugin;
use bevy_ui_anchor::{AnchorPoint, AnchorUiConfig, AnchoredUiNodes};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::time::SystemTime;

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
mod game;

mod assets;
mod prelude;

#[derive(Resource, DerefMut, Deref)]
pub struct GlobalRng(StdRng);

// Enum that will be used as a global state for the game
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Splash,
    Menu,
    Game,
}

// One of the two settings that can be set through the menu. It will be a resource in the app
#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
enum DisplayQuality {
    Low,
    Medium,
    High,
}

// One of the two settings that can be set through the menu. It will be a resource in the app
#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
struct Volume(u32);

fn move_camera(
    keys: Res<ButtonInput<KeyCode>>,
    mut cam_pos: Query<&mut Transform, With<Camera>>,
    mut proj: Query<&mut Projection>,
) {
    let Ok(mut c) = cam_pos.single_mut() else {
        error!("we have multiple cameras or something");
        return;
    };

    let c = &mut c.translation;

    let v = |x, y| Vec3::new(x, y, 0.0);

    use KeyCode as K;

    let x = [
        (K::ArrowRight, v(1.0, 0.0)),
        (K::ArrowLeft, v(-1.0, 0.0)),
        (K::ArrowUp, v(0.0, 1.0)),
        (K::ArrowDown, v(0.0, -1.0)),
    ];
    let shift = keys.pressed(K::ShiftLeft);

    for (key, dir) in x {
        if keys.pressed(key) {
            *c += dir * if shift { 60.0 } else { 20.0 };
        }
    }
}

fn zoom_camera(keys: Res<ButtonInput<KeyCode>>, mut proj: Query<&mut Projection>) {
    use KeyCode as K;

    let Ok(mut proj) = proj.single_mut() else {
        error!("we have multiple cameras or something");
        return;
    };
    let Projection::Orthographic(proj) = &mut *proj else {
        error!("projection wasn't orthographic :(");
        return;
    };

    let shift = keys.pressed(K::ShiftLeft);

    if keys.pressed(K::KeyZ) {
        proj.scale += 0.05 * if shift { 3.0 } else { 1.0 };
    } else if keys.pressed(K::KeyX) {
        proj.scale -= 0.05 * if shift { 3.0 } else { 1.0 };
    }

    proj.scale = 0.05f32.max(proj.scale);
}

fn scale_city_nodes(
    proj: Query<&Projection>,
    city_nodes: Query<&AnchoredUiNodes, With<CityData>>,
    mut city_image_nodes: Query<(&mut Node, Option<&CityNodeMarker>, Option<&CityImageMarker>)>,
) {
    let Ok(proj) = proj.single() else {
        error!("we have multiple cameras or something");
        return;
    };
    let Projection::Orthographic(proj) = proj else {
        error!("projection wasn't orthographic :(");
        return;
    };

    let Vec2 { x: w, y: h } = Vec2::splat(16.0) / proj.scale;
    //Might be a horrible perforer
    for city in city_nodes {
        for node in city.collection() {
            let Ok((mut node, clickable, sprite)) = city_image_nodes.get_mut(*node) else {
                error!("child wasn't real");
                continue;
            };

            if sprite.is_some() {
                node.width = px(w);
                node.height = px(h);
            } else if clickable.is_some() {
                node.min_width = px(w.max(32.0));
                node.min_height = px(h.max(32.0));
            }
        }
    }
}

#[derive(Component)]
struct DefaultUiCameraMarker;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            FeathersPlugins,
            TextInputPlugin,
            AnchorUiPlugin::<DefaultUiCameraMarker>::new(),
        ))
        .add_plugins((
            splash::splash_plugin,
            menu::menu_plugin,
            game::plugin,
            assets::plugin,
        ))
        // Insert as resource the initial value for the settings resources
        .insert_resource(DisplayQuality::Medium)
        .insert_resource(GlobalRng(StdRng::from_seed([0; 32])))
        //.insert_resource(GlobalRng(StdRng::seed_from_u64(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("Error in system time.").as_secs())))
        .insert_resource(Volume(7))
        // Declare the game state, whose starting value is determined by the `Default` trait
        .init_state::<GameState>()
        .add_systems(Startup, debug_city_names)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_camera,
                zoom_camera,
                scale_city_nodes.run_if(any_match_filter::<Changed<Projection>>),
            ),
        )
        // Adds the plugins for each state
        .run();
}

fn debug_city_names(mut rng: ResMut<GlobalRng>) {
    let names = generate_city_names((10, 10, 10, 10), &mut rng);
    let mut display = |t: String, list: &Vec<String>| {
        println!("{0} cities:", t);
        for x in list {
            println!("{0}", x);
        }
    };
    display("dwarven".to_string(), &names[0]);
    display("elven".to_string(), &names[1]);
    display("goblin".to_string(), &names[2]);
    display("human".to_string(), &names[3]);
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 2.5,
            ..OrthographicProjection::default_2d()
        }),
        DefaultUiCameraMarker,
        IsDefaultUiCamera,
        Transform::default(),
    ));
}

mod splash {
    use bevy::prelude::*;

    use super::GameState;

    // This plugin will display a splash screen with Bevy logo for 1 second before switching to the menu
    pub fn splash_plugin(app: &mut App) {
        // As this plugin is managing the splash screen, it will focus on the state `GameState::Splash`
        app
            // When entering the state, spawn everything needed for this screen
            .add_systems(OnEnter(GameState::Splash), splash_setup)
            // While in this state, run the `countdown` system
            .add_systems(Update, countdown.run_if(in_state(GameState::Splash)));
    }

    // Tag component used to tag entities added on the splash screen
    #[derive(Component)]
    struct OnSplashScreen;

    // Newtype to use a `Timer` for this screen as a resource
    #[derive(Resource, Deref, DerefMut)]
    struct SplashTimer(Timer);

    fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        let icon = asset_server.load("sprites/mascot.png");
        // Display the logo
        commands.spawn((
            // This entity will be despawned when exiting the state
            DespawnOnExit(GameState::Splash),
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            OnSplashScreen,
            children![(
                ImageNode::new(icon),
                Node {
                    // This will set the logo to be 200px wide, and auto adjust its height
                    width: px(200),
                    ..default()
                },
            )],
        ));
        // Insert the timer as a resource
        commands.insert_resource(SplashTimer(Timer::from_seconds(2.0, TimerMode::Once)));
    }

    // Tick the timer, and change state when finished
    fn countdown(
        mut game_state: ResMut<NextState<GameState>>,
        time: Res<Time>,
        mut timer: ResMut<SplashTimer>,
    ) {
        if timer.tick(time.delta()).is_finished() {
            game_state.set(GameState::Menu);
        }
    }
}

mod menu {
    use bevy::{
        app::AppExit,
        color::palettes::css::CRIMSON,
        ecs::spawn::{SpawnIter, SpawnWith},
        prelude::*,
    };

    use super::{DisplayQuality, GameState, Volume, TEXT_COLOR};

    // This plugin manages the menu, with 5 different screens:
    // - a main menu with "New Game", "Settings", "Quit"
    // - a settings menu with two submenus and a back button
    // - two settings screen with a setting that can be set and a back button
    pub fn menu_plugin(app: &mut App) {
        app
            // At start, the menu is not enabled. This will be changed in `menu_setup` when
            // entering the `GameState::Menu` state.
            // Current screen in the menu is handled by an independent state from `GameState`
            .init_state::<MenuState>()
            .add_systems(OnEnter(GameState::Menu), menu_setup)
            // Systems to handle the main menu screen
            .add_systems(OnEnter(MenuState::Main), main_menu_setup)
            // Systems to handle the settings menu screen
            .add_systems(OnEnter(MenuState::Settings), settings_menu_setup)
            // Systems to handle the display settings screen
            .add_systems(
                OnEnter(MenuState::SettingsDisplay),
                display_settings_menu_setup,
            )
            .add_systems(
                Update,
                (setting_button::<DisplayQuality>.run_if(in_state(MenuState::SettingsDisplay)),),
            )
            // Systems to handle the sound settings screen
            .add_systems(OnEnter(MenuState::SettingsSound), sound_settings_menu_setup)
            .add_systems(
                Update,
                setting_button::<Volume>.run_if(in_state(MenuState::SettingsSound)),
            )
            // Common systems to all screens that handles buttons behavior
            .add_systems(
                Update,
                (menu_action, button_system).run_if(in_state(GameState::Menu)),
            );
    }

    // State used for the current menu screen
    #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
    enum MenuState {
        Main,
        Settings,
        SettingsDisplay,
        SettingsSound,
        #[default]
        Disabled,
    }

    // Tag component used to tag entities added on the main menu screen
    #[derive(Component)]
    struct OnMainMenuScreen;

    // Tag component used to tag entities added on the settings menu screen
    #[derive(Component)]
    struct OnSettingsMenuScreen;

    // Tag component used to tag entities added on the display settings menu screen
    #[derive(Component)]
    struct OnDisplaySettingsMenuScreen;

    // Tag component used to tag entities added on the sound settings menu screen
    #[derive(Component)]
    struct OnSoundSettingsMenuScreen;

    const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
    const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
    const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
    const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

    // Tag component used to mark which setting is currently selected
    #[derive(Component)]
    struct SelectedOption;

    // All actions that can be triggered from a button click
    #[derive(Component)]
    enum MenuButtonAction {
        Play,
        Settings,
        Credits,
        SettingsDisplay,
        SettingsSound,
        BackToMainMenu,
        BackToSettings,
        Quit,
    }

    // This system handles changing all buttons color based on mouse interaction
    fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, mut background_color, selected) in &mut interaction_query {
            *background_color = match (*interaction, selected) {
                (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
                (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
                (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
                (Interaction::None, None) => NORMAL_BUTTON.into(),
            }
        }
    }

    // This system updates the settings when a new value for a setting is selected, and marks
    // the button as the one currently selected
    fn setting_button<T: Resource + Component + PartialEq + Copy>(
        interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
        selected_query: Single<(Entity, &mut BackgroundColor), With<SelectedOption>>,
        mut commands: Commands,
        mut setting: ResMut<T>,
    ) {
        let (previous_button, mut previous_button_color) = selected_query.into_inner();
        for (interaction, button_setting, entity) in &interaction_query {
            if *interaction == Interaction::Pressed && *setting != *button_setting {
                *previous_button_color = NORMAL_BUTTON.into();
                commands.entity(previous_button).remove::<SelectedOption>();
                commands.entity(entity).insert(SelectedOption);
                *setting = *button_setting;
            }
        }
    }

    fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
        menu_state.set(MenuState::Main);
    }

    fn main_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        commands.spawn((
            AudioPlayer::new(asset_server.load("music/Bellsachiming.ogg")),
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

        let right_icon = asset_server.load("textures/Game Icons/right.png");
        let wrench_icon = asset_server.load("textures/Game Icons/wrench.png");
        let exit_icon = asset_server.load("textures/Game Icons/exitRight.png");

        commands.spawn((
            DespawnOnExit(MenuState::Main),
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnMainMenuScreen,
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
                        Text::new("Spelsylt game"),
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
                        MenuButtonAction::Play,
                        children![
                            (ImageNode::new(right_icon), button_icon_node.clone()),
                            (
                                Text::new("New Game"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ),
                        ]
                    ),
                    (
                        Button,
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        MenuButtonAction::Settings,
                        children![
                            (ImageNode::new(wrench_icon), button_icon_node.clone()),
                            (
                                Text::new("Settings"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ),
                        ]
                    ),
                    (
                        Button,
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        MenuButtonAction::Credits,
                        children![(
                            Text::new("Credits"),
                            button_text_font.clone(),
                            TextColor(TEXT_COLOR),
                        ),]
                    ),
                    (
                        Button,
                        button_node,
                        BackgroundColor(NORMAL_BUTTON),
                        MenuButtonAction::Quit,
                        children![
                            (ImageNode::new(exit_icon), button_icon_node),
                            (Text::new("Quit"), button_text_font, TextColor(TEXT_COLOR),),
                        ]
                    ),
                ]
            )],
        ));
    }

    fn settings_menu_setup(mut commands: Commands) {
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
            DespawnOnExit(MenuState::Settings),
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnSettingsMenuScreen,
            children![(
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(CRIMSON.into()),
                Children::spawn(SpawnIter(
                    [
                        (MenuButtonAction::SettingsDisplay, "Display"),
                        (MenuButtonAction::SettingsSound, "Sound"),
                        (MenuButtonAction::BackToMainMenu, "Back"),
                    ]
                    .into_iter()
                    .map(move |(action, text)| {
                        (
                            Button,
                            button_node.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            action,
                            children![(Text::new(text), button_text_style.clone())],
                        )
                    })
                ))
            )],
        ));
    }

    fn display_settings_menu_setup(mut commands: Commands, display_quality: Res<DisplayQuality>) {
        fn button_node() -> Node {
            Node {
                width: px(200),
                height: px(65),
                margin: UiRect::all(px(20)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            }
        }
        fn button_text_style() -> impl Bundle {
            (
                TextFont {
                    font_size: 33.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            )
        }

        let display_quality = *display_quality;
        commands.spawn((
            DespawnOnExit(MenuState::SettingsDisplay),
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnDisplaySettingsMenuScreen,
            children![(
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(CRIMSON.into()),
                children![
                    // Create a new `Node`, this time not setting its `flex_direction`. It will
                    // use the default value, `FlexDirection::Row`, from left to right.
                    (
                        Node {
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(CRIMSON.into()),
                        Children::spawn((
                            // Display a label for the current setting
                            Spawn((Text::new("Display Quality"), button_text_style())),
                            SpawnWith(move |parent: &mut ChildSpawner| {
                                for quality_setting in [
                                    DisplayQuality::Low,
                                    DisplayQuality::Medium,
                                    DisplayQuality::High,
                                ] {
                                    let mut entity = parent.spawn((
                                        Button,
                                        Node {
                                            width: px(150),
                                            height: px(65),
                                            ..button_node()
                                        },
                                        BackgroundColor(NORMAL_BUTTON),
                                        quality_setting,
                                        children![(
                                            Text::new(format!("{quality_setting:?}")),
                                            button_text_style(),
                                        )],
                                    ));
                                    if display_quality == quality_setting {
                                        entity.insert(SelectedOption);
                                    }
                                }
                            })
                        ))
                    ),
                    // Display the back button to return to the settings screen
                    (
                        Button,
                        button_node(),
                        BackgroundColor(NORMAL_BUTTON),
                        MenuButtonAction::BackToSettings,
                        children![(Text::new("Back"), button_text_style())]
                    )
                ]
            )],
        ));
    }

    fn sound_settings_menu_setup(mut commands: Commands, volume: Res<Volume>) {
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

        let volume = *volume;
        let button_node_clone = button_node.clone();
        commands.spawn((
            DespawnOnExit(MenuState::SettingsSound),
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnSoundSettingsMenuScreen,
            children![(
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(CRIMSON.into()),
                children![
                    (
                        Node {
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(CRIMSON.into()),
                        Children::spawn((
                            Spawn((Text::new("Volume"), button_text_style.clone())),
                            SpawnWith(move |parent: &mut ChildSpawner| {
                                for volume_setting in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
                                    let mut entity = parent.spawn((
                                        Button,
                                        Node {
                                            width: px(30),
                                            height: px(65),
                                            ..button_node_clone.clone()
                                        },
                                        BackgroundColor(NORMAL_BUTTON),
                                        Volume(volume_setting),
                                    ));
                                    if volume == Volume(volume_setting) {
                                        entity.insert(SelectedOption);
                                    }
                                }
                            })
                        ))
                    ),
                    (
                        Button,
                        button_node,
                        BackgroundColor(NORMAL_BUTTON),
                        MenuButtonAction::BackToSettings,
                        children![(Text::new("Back"), button_text_style)]
                    )
                ]
            )],
        ));
    }

    fn menu_action(
        interaction_query: Query<
            (&Interaction, &MenuButtonAction),
            (Changed<Interaction>, With<Button>),
        >,
        mut app_exit_writer: MessageWriter<AppExit>,
        mut menu_state: ResMut<NextState<MenuState>>,
        mut game_state: ResMut<NextState<GameState>>,
    ) {
        for (interaction, menu_button_action) in &interaction_query {
            if *interaction == Interaction::Pressed {
                match menu_button_action {
                    MenuButtonAction::Quit => {
                        app_exit_writer.write(AppExit::Success);
                    }
                    MenuButtonAction::Play => {
                        game_state.set(GameState::Game);
                        menu_state.set(MenuState::Disabled);
                    }
                    MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                    MenuButtonAction::SettingsDisplay => {
                        menu_state.set(MenuState::SettingsDisplay);
                    }
                    MenuButtonAction::SettingsSound => {
                        menu_state.set(MenuState::SettingsSound);
                    }
                    MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                    MenuButtonAction::BackToSettings => {
                        menu_state.set(MenuState::Settings);
                    }
                    MenuButtonAction::Credits => {
                        todo!();
                    }
                }
            }
        }
    }
}

fn kill_music(mut commands: Commands, mut audio_sources: Query<Entity, With<AudioPlayer>>) {
    for entity in audio_sources.iter_mut() {
        commands.entity(entity).despawn();
    }
}
