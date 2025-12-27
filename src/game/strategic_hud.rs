use bevy::math::usize;
use bevy::ui::InteractionDisabled;

use super::market::*;
use super::strategic_map::{CityData, SelectedCity, StrategicState};
use crate::prelude::*;
pub fn plugin(app: &mut App) {
    app.init_state::<PopupHUD>()
        .add_systems(OnExit(PopupHUD::Off), remove_interaction)
        .add_systems(OnEnter(PopupHUD::Off), put_back_interaction)
        .add_systems(OnEnter(StrategicState::HUDOpen), city_hud_setup)
        .add_systems(OnEnter(PopupHUD::Buildings), building_menu)
        .add_systems(OnEnter(PopupHUD::Caravan), caravan_menu)
        .add_systems(OnEnter(PopupHUD::Wares), wares_menu)
        .add_systems(Update, no_popup_button.run_if(in_state(PopupHUD::Off)))
        .add_systems(Update, popup_button);
}

fn remove_interaction(mut commands: Commands, query: Query<Entity, With<Node>>) {
    for node in query {
        commands.entity(node).insert(InteractionDisabled);
    }
}
fn put_back_interaction(mut commands: Commands, query: Query<Entity, With<Node>>) {
    for node in query {
        commands.entity(node).remove::<InteractionDisabled>();
    }
}

#[derive(Component)]
struct PopUpItem;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum PopupHUD {
    #[default]
    Off,
    Buildings,
    Caravan,
    Wares,
}

fn no_popup_button(
    mut interaction_query: Query<(&Interaction, &HudButton), (Changed<Interaction>, With<Button>)>,
    mut menu_state: ResMut<NextState<StrategicState>>,
    mut tab_state: ResMut<NextState<PopupHUD>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                HudButton::KillHud => {
                    menu_state.set(StrategicState::Map);
                }

                HudButton::OperationAction => {
                    tab_state.set(PopupHUD::Caravan);
                }
                HudButton::EconomyTabAction => {
                    tab_state.set(PopupHUD::Wares);
                }
                HudButton::BuldingTabAction => {
                    tab_state.set(PopupHUD::Buildings);
                }

                _ => {}
            }
        }
    }
}

fn popup_button(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &PopupButton),
        (Changed<Interaction>, With<Button>),
    >,
    //mut menu_state: ResMut<NextState<StrategicState>>,
    mut tab_state: ResMut<NextState<PopupHUD>>,
    mut popup_items: Query<Entity, With<PopUpItem>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                PopupButton::KillHud => {
                    tab_state.set(PopupHUD::Off);
                    for entity in popup_items.iter() {
                        commands.entity(entity).despawn();
                    }
                }
                _ => {}
            }
        }
    }
}

fn popup_window(commands: &mut Commands, row_align: bool, mut sylt: &mut Sylt) -> Entity {
    commands.spawn((
        ZIndex(1),
        PopUpItem,
        Node {
            width: Val::Vw(100.0),
            height: Val::Vh(100.0),
            ..default()
        },
        BackgroundColor(Srgba::new(0.0, 0.0, 0.0, 0.7).into()),
    ));
    commands
        .spawn((
            ZIndex(2),
            PopUpItem,
            Node {
                top: Val::Vh(10.0),
                left: Val::Vw(10.0),
                width: Val::Vw(80.0),
                height: Val::Vh(80.0),
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::FlexStart,
                display: Display::Flex,
                flex_direction: match row_align {
                    true => FlexDirection::Row,
                    false => FlexDirection::Column,
                },
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            ImageNode::new(sylt.get_image("parchment")),
            //BackgroundColor(Srgba::new(0.2, 0.2, 0.2, 1.0).into()),
            BorderColor::all(Color::BLACK),
            children![(
                Button,
                ZIndex(3),
                PopupButton::KillHud,
                Node {
                    position_type: PositionType::Absolute,
                    top: px(0),
                    right: px(0),
                    width: px(32),
                    height: px(32),
                    ..default()
                },
                BackgroundColor(Srgba::new(0.9, 0.2, 0.2, 1.0).into())
            )],
        ))
        .id()
}

fn building_menu(mut commands: Commands, city: ResMut<SelectedCity>, mut sylt: Sylt) {
    let window = popup_window(&mut commands, false, &mut sylt);
    for tiers in 1..(city.0.population + 1) {
        commands.entity(window).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100),
                        height: percent(15),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    Text::new(format!("Tier {}", tiers)),
                    BackgroundColor(Srgba::new(0.2, 0.2, 1.0, 1.0).into()),
                ))
                .with_children(|parent| {
                    for building_slot in 0..tiers {
                        //println!("Tier {} has slot {}", tiers, building_slot);
                        if let Some(building) = match tiers {
                            1 => city.buildings_t1.get(building_slot as usize),
                            2 => city.buildings_t2.get(building_slot as usize),
                            3 => city.buildings_t3.get(building_slot as usize),
                            4 => city.buildings_t4.get(building_slot as usize),
                            5 => city.buildings_t5.get(building_slot as usize),
                            _ => None,
                        } {
                            println!("Found building {}", building.0);
                            parent.spawn((
                                Node {
                                    width: px(64),
                                    height: px(64),
                                    margin: UiRect::all(px(4)),
                                    ..default()
                                },
                                BackgroundColor(Srgba::new(0.5, 0.2, 0.9, 1.0).into()),
                            ));
                        } else {
                            parent.spawn((
                                Node {
                                    width: px(64),
                                    height: px(64),
                                    margin: UiRect::all(px(16)),
                                    ..default()
                                },
                                BackgroundColor(Srgba::new(0.2, 1., 0.2, 1.0).into()),
                            ));
                        }
                    }
                });
        });
    }
}

fn caravan_menu(mut commands: Commands, mut sylt: Sylt) {
    let window = popup_window(&mut commands, false, &mut sylt);

    commands.entity(window).with_children(|parent| {
        parent.spawn((
            Node {
                width: percent(100),
                height: percent(15),
                align_items: AlignItems::FlexEnd,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(Srgba::new(0.2, 0.2, 1.0, 1.0).into()),
        ));
    });
}

fn wares_menu(mut commands: Commands, mut sylt: Sylt) {
    let window = popup_window(&mut commands, true, &mut sylt);

    //Basic and exotic mats
    commands.entity(window).with_children(|parent| {
        //Basic and exotic mats
        parent
            .spawn((Node {
                top: px(32),
                width: percent(33),
                height: percent(100),
                margin: UiRect::all(px(4)),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                ..default()
            },))
            .with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            width: percent(100),
                            height: percent(60),
                            margin: UiRect::all(px(4)),
                            justify_content: JustifyContent::FlexStart,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(Srgba::new(0.05, 0.05, 0.05, 1.0).into()),
                    ))
                    .with_children(|parent| {
                        create_resource_list(
                            parent,
                            vec![(Resources::Water, 12, 200), (Resources::Food, 10, 30)],
                            "Basic materials".to_string(),
                            &mut sylt,
                        );
                    });

                parent
                    .spawn((
                        Node {
                            width: percent(100),
                            height: percent(20),
                            margin: UiRect::all(px(4)),
                            justify_content: JustifyContent::FlexStart,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(Srgba::new(0.05, 0.05, 0.05, 1.0).into()),
                    ))
                    .with_children(|parent| {
                        create_resource_list(
                            parent,
                            vec![(Resources::Stone, 25, 18), (Resources::Food, 10, 30)],
                            "Exotic materials".to_string(),
                            &mut sylt,
                        );
                    });
            });

        //Illegals and Advanced
        parent
            .spawn((Node {
                top: px(32),
                width: percent(33),
                height: percent(100),
                margin: UiRect::all(px(4)),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                ..default()
            },))
            .with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            width: percent(100),
                            height: percent(30),
                            margin: UiRect::all(px(4)),
                            justify_content: JustifyContent::FlexStart,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(Srgba::new(0.05, 0.05, 0.05, 1.0).into()),
                    ))
                    .with_children(|parent| {
                        create_resource_list(
                            parent,
                            vec![(Resources::Water, 12, 200), (Resources::Food, 10, 30)],
                            "Illegal materials".to_string(),
                            &mut sylt,
                        );
                    });

                parent
                    .spawn((
                        Node {
                            width: percent(100),
                            height: percent(50),
                            margin: UiRect::all(px(4)),
                            justify_content: JustifyContent::FlexStart,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(Srgba::new(0.05, 0.05, 0.05, 1.0).into()),
                    ))
                    .with_children(|parent| {
                        create_resource_list(
                            parent,
                            vec![(Resources::Stone, 25, 18), (Resources::Food, 10, 30)],
                            "Advanced materials".to_string(),
                            &mut sylt,
                        );
                    });
            });

        //Services
        //Illegals and Advanced
        parent
            .spawn((Node {
                top: px(32),
                width: percent(33),
                height: percent(100),
                margin: UiRect::all(px(4)),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                ..default()
            },))
            .with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            width: percent(100),
                            height: percent(50),
                            margin: UiRect::all(px(4)),
                            justify_content: JustifyContent::FlexStart,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(Srgba::new(0.05, 0.05, 0.05, 1.0).into()),
                    ))
                    .with_children(|parent| {
                        create_resource_list(
                            parent,
                            vec![(Resources::Water, 12, 200), (Resources::Food, 10, 30)],
                            "Services".to_string(),
                            &mut sylt,
                        );
                    });
            });
    });
}

#[derive(Component)]
enum HudButton {
    KillHud,
    OperationAction,
    EconomyTabAction,
    BuldingTabAction,
}

#[derive(Component)]
enum PopupButton {
    KillHud,
    BuldingTabAction,
}

fn create_resource_list(
    parent: &mut ChildSpawnerCommands,
    resources: Vec<(Resources, usize, usize)>,
    box_name: String,
    mut sylt: &mut Sylt,
) {
    parent.spawn((
        Node {
            width: percent(100),
            height: px(40),
            ..default()
        },
        Text::new(box_name.clone()),
    ));

    for (resouce_type, cost, available) in resources {
        create_resource_icon(parent, resouce_type, cost, available, &mut sylt);
    }
}

fn create_resource_icon(
    parent: &mut ChildSpawnerCommands,
    resource: Resources,
    cost: usize,
    amount: usize,
    sylt: &mut Sylt,
) {
    parent.spawn((
        Node {
            width: percent(100),
            height: px(40),
            margin: UiRect {
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(4),
            },
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::SpaceBetween,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderColor::all(Color::BLACK),
        children![
            (
                Node {
                    right: px(0),
                    width: px(40),
                    height: px(40),
                    ..default()
                },
                ImageNode {
                    image: sylt
                        .get_sprite(match resource {
                            Resources::Water => "resource_water",
                            Resources::Stone => "resource_stone",
                            Resources::Lumber => "resource_wood",
                            _ => "map",
                        })
                        .image,
                    ..default()
                },
            ),
            (Text::new(format!("{}x", amount)),),
            (Text::new(format!("{}$", cost)),)
        ],
    ));
}

fn city_hud_setup(mut commands: Commands, mut sylt: Sylt, selected_city: Res<SelectedCity>) {
    let city = selected_city.0.clone();
    //Map quit upon click
    commands.spawn((
        DespawnOnExit(StrategicState::HUDOpen),
        Node {
            top: Val::Vh(0.0),
            width: Val::Vw(100.0),
            height: Val::Vh(70.0),
            ..default()
        },
        Button,
        HudButton::KillHud, //Feels like a clunky way to quit the menu
    ));

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
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                children![
                    (
                        Button,
                        HudButton::BuldingTabAction,
                        Node {
                            width: percent(18),
                            height: percent(80),
                            margin: UiRect::all(percent(1)),
                            ..default()
                        },
                        Text::new("Investigate buildings"),
                        BackgroundColor(Srgba::new(0.1, 0.9, 0.1, 1.0).into()),
                    ),
                    (
                        Button,
                        HudButton::EconomyTabAction,
                        Node {
                            width: percent(18),
                            height: percent(80),
                            margin: UiRect::all(percent(1)),
                            ..default()
                        },
                        Text::new("Check wares"),
                        BackgroundColor(Srgba::new(0.1, 0.9, 0.1, 1.0).into())
                    ),
                    (
                        Button,
                        HudButton::OperationAction,
                        Node {
                            width: percent(18),
                            height: percent(80),
                            margin: UiRect::all(percent(1)),
                            ..default()
                        },
                        Text::new("Send a new caravan"),
                        BackgroundColor(Srgba::new(0.1, 0.9, 0.1, 1.0).into())
                    )
                ]
            ),
        ],
    ));
}
