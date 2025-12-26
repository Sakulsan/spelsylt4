use bevy::prelude::*;
use std::marker::PhantomData;

use bevy::ui_widgets::{observe, ValueChange};

use super::market::Resources;
use crate::assets::Sylt;
use crate::GameState;

// This plugin will contain the game. In this case, it's just be a screen that will
// display the current settings for 5 seconds before returning to the menu
#[derive(Resource, Deref)]
struct SelectedCity(String);

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Game), strategic_setup)
        .insert_resource(SelectedCity("Unkown".to_string()))
        .init_state::<StrategicState>()
        .init_state::<HUDPosition>()
        .add_systems(OnEnter(StrategicState::HUDOpen), hud_setup)
        //        .add_systems(OnEnter(HUDPosition::Actions), open_actions)
        //        .add_systems(OnEnter(HUDPosition::Buildings), open_buildings)
        //        .add_systems(OnEnter(HUDPosition::Market), open_market)
        .add_systems(Update, (city_interaction_system, kill_button));
}

/*#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum StrategicState<T: Send + Sync + Eq + std::fmt::Debug + std::hash::Hash + Clone + 'static> {
    #[default]
    Map,
    HUDOpen(HUDPosition, T),
}*/

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum StrategicState {
    #[default]
    Map,
    HUDOpen,
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum HUDPosition {
    #[default]
    Buildings,
    Actions,
    Market,
}

fn strategic_setup(
    mut commands: Commands,
    //    display_quality: Res<DisplayQuality>,
    //    volume: Res<Volume>,
    mut sylt: Sylt,
) {
    commands.spawn((
        Node {
            width: Val::Vw(100.0),
            height: Val::Vh(100.0),
            ..default()
        },
        Sprite {
            image: sylt.get_sprite("map").image,
            ..default()
        },
        children![(
            Button,
            CityData {
                id: "Capital".to_string(),
                population: 2,
                districts: vec![DistrictType::Farm, DistrictType::Mine],
            },
            CityIcon {
                id: "Capital".to_string()
            },
            Node {
                width: Val::Px(20.0),
                height: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Srgba::new(1.0, 0.1, 0.1, 1.0).into()),
        )],
    ));
}

fn kill_button(
    mut interaction_query: Query<(&Interaction, &HudButton), (Changed<Interaction>, With<Button>)>,
    mut menu_state: ResMut<NextState<StrategicState>>,
    mut tab_state: ResMut<NextState<HUDPosition>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                HudButton::KillHud => {
                    menu_state.set(StrategicState::Map);
                }

                HudButton::OperationAction => {
                    tab_state.set(HUDPosition::Actions);
                }
                HudButton::EconomyTabAction => {
                    tab_state.set(HUDPosition::Market);
                }
                HudButton::BuldingTabAction => {
                    tab_state.set(HUDPosition::Buildings);
                }

                _ => {}
            }
        }
    }
}

#[derive(Component)]
enum HudButton {
    KillHud,
    ConstructionAction,
    OperationAction,
    EconomyTabAction,
    BuldingTabAction,
}

fn create_resource_icon(parent: &mut ChildSpawnerCommands, resource: Resources) {
    parent.spawn((
        Node {
            width: px(128),
            height: px(64),
            margin: UiRect::all(px(4)),
            ..default()
        },
        BackgroundColor(Srgba::new(0.7, 0.7, 0.7, 1.0).into()),
        children![
            (
                Node {
                    width: px(64),
                    height: px(64),
                    margin: UiRect::all(px(4)),
                    ..default()
                },
                BackgroundColor(Srgba::new(0.7, 0.7, 0.7, 1.0).into()),
            ),
            (
                Node {
                    width: px(64),
                    height: px(64),
                    margin: UiRect::all(px(4)),
                    ..default()
                },
                BackgroundColor(Srgba::new(0.7, 0.5, 0.5, 1.0).into()),
            )
        ],
    ));
}

fn hud_setup(
    mut commands: Commands,
    mut sylt: Sylt,
    city_data: Query<&CityData>,
    selected_city: Res<SelectedCity>,
) {
    for city in city_data {
        if city.id == selected_city.0 {
            //Map quit upon click
            commands.spawn((
                DespawnOnExit(StrategicState::HUDOpen),
                Node {
                    top: Val::Vh(0.0),
                    width: Val::Vw(60.0),
                    height: Val::Vh(70.0),
                    ..default()
                },
                Button,
                HudButton::KillHud, //Feels like a clunky way to quit the menu
            ));

            //Market values
            commands
                .spawn((
                    DespawnOnExit(StrategicState::HUDOpen),
                    Node {
                        top: Val::Vh(0.0),
                        left: Val::Vh(100.0),
                        width: Val::Vw(40.0),
                        height: Val::Vh(70.0),
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::Start,
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Srgba::new(0.2, 0.2, 0.2, 1.0).into()),
                ))
                .with_children(|parent| {
                    for resource in vec![Resources::Water, Resources::Stone, Resources::Souls] {
                        create_resource_icon(parent, resource);
                    }
                });

            //Action menu
            commands.spawn((
                DespawnOnExit(StrategicState::HUDOpen),
                Node {
                    top: Val::Vh(70.0),
                    width: Val::Vw(100.0),
                    height: Val::Vh(40.0),
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    ..default()
                },
                BackgroundColor(Srgba::new(0.2, 0.2, 0.2, 1.0).into()),
                children![
                    (
                        Node {
                            width: percent(100.0),
                            height: percent(20.0),
                            align_items: AlignItems::Start,
                            justify_content: JustifyContent::Start,
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        children![
                            (
                                Node {
                                    width: percent(40.0),
                                    ..default()
                                },
                                // Title
                                Text::new(city.id.clone()),
                                TextFont { ..default() },
                            ),
                            (
                                Button,
                                HudButton::BuldingTabAction,
                                Node {
                                    width: percent(20.0),
                                    ..default()
                                },
                                BackgroundColor(Srgba::new(0.2, 0.2, 0.9, 1.0).into()),
                                Text::new("Buildings"),
                                TextFont { ..default() },
                            ),
                            (
                                Button,
                                HudButton::OperationAction,
                                Node {
                                    width: percent(20.0),
                                    ..default()
                                },
                                BackgroundColor(Srgba::new(0.2, 0.2, 0.9, 1.0).into()),
                                Text::new("Actions"),
                                TextFont { ..default() },
                            ),
                            (
                                Button,
                                HudButton::EconomyTabAction,
                                Node {
                                    width: percent(20.0),
                                    ..default()
                                },
                                BackgroundColor(Srgba::new(0.2, 0.2, 0.9, 1.0).into()),
                                Text::new("Market"),
                                TextFont { ..default() },
                            )
                        ]
                    ),
                    (
                        DespawnOnExit(HUDPosition::Buildings),
                        Node {
                            width: percent(100.0),
                            height: percent(100.0),
                            align_items: AlignItems::Start,
                            justify_content: JustifyContent::Start,
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        Children::spawn((SpawnWith({
                            let districts = city.districts.clone();

                            move |parent: &mut bevy::ecs::relationship::RelatedSpawner<ChildOf>| {
                                //let length = 2;
                                for i in 0..5 {
                                    if i < districts.len() {
                                        parent.spawn((
                                            Node {
                                                width: percent(18),
                                                height: percent(80),
                                                margin: UiRect::all(percent(1)),
                                                ..default()
                                            },
                                            Text::new(match districts[i] {
                                                DistrictType::Farm => "Farm district",
                                                DistrictType::Wizard => "Arcane district",
                                                DistrictType::Smith => "Blacksmith district",
                                                DistrictType::Mine => "Mining district",
                                            }),
                                            BackgroundColor(Srgba::new(0.1, 0.9, 0.1, 1.0).into()),
                                        ));
                                    } else {
                                        parent.spawn((
                                            Node {
                                                width: percent(18),
                                                height: percent(80),
                                                margin: UiRect::all(percent(1)),
                                                ..default()
                                            },
                                            Text::new("Unbuilt district"),
                                            BackgroundColor(Srgba::new(0.9, 0.1, 0.1, 1.0).into()),
                                        ));
                                    }
                                }
                            }
                        }),))
                    ),
                ],
            ));
        }
    }
}

#[derive(Component)]
struct CityIcon {
    id: String,
}

struct Dwarf;
struct Goblin;
struct Elf;
struct Human;
//#[derive(Component)]
//struct Demographic<T>();

#[derive(Clone)]
enum DistrictType {
    Farm,
    Wizard,
    Smith,
    Mine,
}
#[derive(Component)]
struct CityData {
    id: String,
    population: u8,
    districts: Vec<DistrictType>,
}

#[derive(Component)]
struct Market {
    population: u8,
    districts: Vec<DistrictType>,
}

fn city_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &CityIcon),
        Changed<Interaction>,
    >,
    mut menu_state: ResMut<NextState<StrategicState>>,
    mut selected_city: ResMut<SelectedCity>,
) {
    for (interaction, mut node_color, city) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                println!("Pressed the city {}", city.id);
                selected_city.0 = city.id.clone();
                menu_state.set(StrategicState::HUDOpen);
            }
            Interaction::Hovered => *node_color = Srgba::new(1.0, 0.1, 0.1, 1.0).into(),
            _ => {}
        }
    }
}
