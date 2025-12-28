use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::math::usize;
use bevy::picking::hover::HoverMap;
use bevy::ui::InteractionDisabled;

use super::city_data::CityData;
use super::market::*;
use super::strategic_map::{Caravan, Order, Player, SelectedCaravan, SelectedCity, StrategicState};
use super::tooltip::Tooltips;
use crate::game::strategic_map::CityNodeMarker;
use crate::game::strategic_map::{ActivePlayer, BelongsTo};
use crate::game::tooltip::TooltipOf;
use crate::prelude::*;
use crate::GameState;

pub fn plugin(app: &mut App) {
    app.init_state::<PopupHUD>()
        .add_systems(OnEnter(StrategicState::HUDOpen), city_hud_setup)
        .add_systems(OnEnter(PopupHUD::Buildings), building_menu)
        .add_systems(OnEnter(PopupHUD::Caravan), caravan_menu)
        .add_systems(OnEnter(PopupHUD::Wares), wares_menu)
        .add_systems(
            Update,
            caravan_destination_buttons.run_if(in_state(StrategicState::DestinationPicker)),
        )
        .add_systems(OnEnter(StrategicState::DestinationPicker), on_city_scout)
        .add_systems(OnExit(StrategicState::DestinationPicker), off_city_scout)
        .add_systems(OnEnter(PopupHUD::Off), set_interaction(true))
        .add_systems(OnExit(PopupHUD::Off), set_interaction(false))
        .add_systems(
            Update,
            no_popup_button
                .run_if(in_state(GameState::Game))
                .run_if(in_state(PopupHUD::Off)),
        )
        .add_systems(OnExit(PopupHUD::Off), set_interaction(false))
        .add_systems(
            Update,
            (caravan_button, send_scroll_events).run_if(in_state(PopupHUD::Caravan)),
        )
        .add_systems(
            Update,
            update_caravan_menu.run_if(
                any_match_filter::<Changed<Caravan>>.or(resource_changed::<SelectedCaravan>),
            ),
        )
        .add_systems(Update, update_caravan_order_idx)
        .add_observer(on_scroll_handler)
        .add_systems(Update, popup_button);
}

fn set_interaction(show: bool) -> impl Fn(Commands, Query<Entity, With<Node>>) {
    move |mut commands: Commands, query: Query<Entity, With<Node>>| {
        for ent in query {
            let mut cmd = commands.entity(ent);
            if show {
                cmd.remove::<InteractionDisabled>();
            } else {
                cmd.insert(InteractionDisabled);
            }
        }
    }
}

#[derive(Component)]
pub struct PopUpItem;

#[derive(Component)]
pub struct IncomeValue(Resources);

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum PopupHUD {
    #[default]
    Off,
    Buildings,
    Caravan,
    Wares,
}

fn on_city_scout(mut interaction_query: Query<&mut Visibility, With<PopUpItem>>) {
    for mut node_vis in interaction_query.iter_mut() {
        *node_vis = Visibility::Hidden;
    }
}

fn off_city_scout(mut interaction_query: Query<&mut Visibility, With<PopUpItem>>) {
    for mut node_vis in interaction_query.iter_mut() {
        *node_vis = Visibility::Visible;
    }
}

fn no_popup_button(
    mut commands: Commands,
    mut interaction_query: Query<(&Interaction, &HudButton), (Changed<Interaction>, With<Button>)>,
    mut menu_state: ResMut<NextState<StrategicState>>,
    mut tab_state: ResMut<NextState<PopupHUD>>,
    selected_city: Res<SelectedCity>,
    player: Option<Single<Entity, With<ActivePlayer>>>,
) {
    let Some(player) = player else {
        error!("No active player exists!");
        return;
    };

    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                HudButton::KillHud => {
                    menu_state.set(StrategicState::Map);
                }

                HudButton::OperationAction => {
                    info!("Spawning caravan");
                    commands.spawn((
                        Caravan {
                            position_city_id: selected_city.0.id.clone(),
                            orders: vec![Order {
                                goal_city_id: selected_city.0.id.clone(),
                                ..default()
                            }],
                            ..default()
                        },
                        BelongsTo(*player),
                    ));
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

fn popup_window(commands: &mut Commands, direction: FlexDirection) -> Entity {
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
                top: Val::Vh(5.0),
                left: Val::Vw(5.0),
                width: Val::Vw(90.0),
                height: Val::Vh(90.0),
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::FlexStart,
                display: Display::Flex,
                flex_direction: direction,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Srgba::new(0.2, 0.2, 0.2, 1.0).into()),
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

fn building_menu(mut commands: Commands, city: ResMut<SelectedCity>) {
    let window = popup_window(&mut commands, FlexDirection::ColumnReverse);
    let population = city.0.population + 1;
    for tiers in 1..population {
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
                    for building_slot in 0..population - tiers {
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
                                Button,
                                related!(
                                    Tooltips[(
                                        Text::new(building.0.clone()),
                                        TextShadow::default(),
                                        // Set the justification of the Text
                                        TextLayout::new_with_justify(Justify::Center),
                                        // Set the style of the Node itself.
                                        Node { ..default() },
                                        BackgroundColor(Srgba::new(0.05, 0.05, 0.05, 1.0).into()),
                                    ),
                                    (
                                        Text::new("I produce this"),
                                        TextShadow::default(),
                                        // Set the justification of the Text
                                        TextLayout::new_with_justify(Justify::Center),
                                        // Set the style of the Node itself.
                                        Node { ..default() }
                                    ),
                                    (
                                        Text::new("here is lore"),
                                        TextShadow::default(),
                                        // Set the justification of the Text
                                        TextLayout::new_with_justify(Justify::Center),
                                        // Set the style of the Node itself.
                                        Node { ..default() }
                                    )]
                                ),
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

#[derive(Component, Default, Clone, Debug)]
struct CaravanMenu;
#[derive(Component, Clone, Eq, PartialEq, Debug, Hash)]
enum CaravanMenuButtons {
    NewStop,
    RemoveStop(String),
    AddTradeToStop(String),
    ChangeTrade(String, Resources),
    IncTradeAmount(String, Resources),
    DecTradeAmount(String, Resources),
    KillTrade(String, Resources),
    ChangeTradeConfirm(String, Resources, Resources),
}

fn caravan_menu(mut commands: Commands) {
    let id = popup_window(&mut commands, FlexDirection::Column);
    commands.entity(id).with_children(|parent| {
        parent.spawn((
            Node {
                height: percent(100),
                width: percent(100),
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            CaravanMenu,
        ));
    });
}

//Redo the caravan menu in case of new buttons and so on
fn update_caravan_menu(
    caravan_box: Query<Entity, With<CaravanMenu>>,
    selected_caravan: ResMut<SelectedCaravan>,
    caravans: Query<&Caravan>,
    mut commands: Commands,
) {
    let Ok(selected_caravan) = caravans.get(selected_caravan.0) else {
        error!("No selected caravan to display");
        return;
    };

    for caravan_box in caravan_box.iter() {
        commands.entity(caravan_box).despawn_children();
        commands.entity(caravan_box).with_children(|parent| {
            parent.spawn((
                Node {
                    width: percent(100),
                    height: percent(10),
                    ..default()
                },
                Text::new(format!(
                    "Caravan in {}",
                    selected_caravan.position_city_id.clone()
                )),
            ));
            parent
                .spawn((Node {
                    width: percent(100),
                    height: percent(90),
                    align_items: AlignItems::FlexStart,
                    flex_direction: FlexDirection::Column,

                    ..default()
                },))
                //Actually content
                .with_children(|parent| {
                    create_route_showcase(parent, &selected_caravan.orders);
                    parent.spawn((
                        Button,
                        CaravanMenuButtons::NewStop,
                        Node {
                            width: percent(100),
                            height: px(64),
                            margin: UiRect::all(px(4)),
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        Text::new("New stop"),
                        BackgroundColor(Srgba::new(1.0, 0.1, 0.1, 1.0).into()),
                    ));
                });
        });
    }
}

#[derive(Component, Default, Clone, Debug)]
struct CaravanCityUINode(String);

fn create_route_showcase(parent: &mut ChildSpawnerCommands, orders: &Vec<Order>) {
    for stop in orders {
        let transaction_count = stop.trade_order.len();

        parent
            .spawn((
                CaravanCityUINode(stop.goal_city_id.clone()),
                Node {
                    left: percent(5),
                    width: percent(90),
                    min_height: px(72 + 48 * transaction_count),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Srgba::new(0.1, 0.1, 0.1, 1.0).into()),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Node {
                        height: px(64),
                        margin: UiRect::all(px(4)),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    children![
                        (Text::new(stop.goal_city_id.clone()),),
                        (
                            Button,
                            CaravanMenuButtons::AddTradeToStop(stop.goal_city_id.clone()),
                            Node {
                                position_type: PositionType::Absolute,
                                width: px(256),
                                height: px(60),
                                top: px(0),
                                right: px(76),
                                border: UiRect::all(px(2)),
                                ..default()
                            },
                            Text::new("New transaction"),
                            BackgroundColor(Srgba::new(0.1, 0.9, 0.1, 1.0).into()),
                        ),
                        (
                            Button,
                            CaravanMenuButtons::RemoveStop(stop.goal_city_id.clone()),
                            Node {
                                position_type: PositionType::Absolute,
                                width: px(64),
                                height: px(60),
                                top: px(0),
                                right: px(0),
                                border: UiRect::all(px(2)),
                                ..default()
                            },
                            BackgroundColor(Srgba::new(0.9, 0.1, 0.1, 1.0).into()),
                        )
                    ],
                ));

                for (resource, amount) in &stop.trade_order {
                    //A single stops hud
                    parent
                        .spawn((
                            Node {
                                width: percent(85),
                                left: percent(10),
                                height: px(48),
                                border: UiRect::all(px(4)),
                                ..default()
                            },
                            BackgroundColor(Srgba::new(0.1, 1.0, 0.1, 1.0).into()),
                            BorderColor::all(Color::BLACK),
                            Text::new("Buys something"),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Button,
                                CaravanMenuButtons::ChangeTrade(
                                    stop.goal_city_id.clone(),
                                    *resource,
                                ),
                                Node {
                                    width: px(256),
                                    height: px(44),
                                    margin: UiRect::all(px(2)),
                                    border: UiRect::all(px(2)),
                                    ..default()
                                },
                                BackgroundColor(Srgba::new(0.9, 0.2, 0.2, 1.0).into()),
                                Text::new(resource.get_name()),
                            ));

                            parent.spawn((
                                Button,
                                CaravanMenuButtons::DecTradeAmount(
                                    stop.goal_city_id.clone(),
                                    *resource,
                                ),
                                Node {
                                    width: px(44),
                                    height: px(44),
                                    margin: UiRect::all(px(2)),
                                    ..default()
                                },
                                BackgroundColor(Srgba::new(0.1, 0.2, 0.8, 1.0).into()),
                                BorderColor::all(Color::BLACK),
                                Text::new("-"),
                            ));
                            parent.spawn((
                                Node {
                                    width: px(44),
                                    height: px(44),
                                    margin: UiRect::all(px(2)),
                                    ..default()
                                },
                                BackgroundColor(Srgba::new(0.1, 0.2, 0.8, 1.0).into()),
                                Text::new(format!("{}", amount)),
                            ));
                            parent.spawn((
                                Button,
                                CaravanMenuButtons::IncTradeAmount(
                                    stop.goal_city_id.clone(),
                                    *resource,
                                ),
                                Node {
                                    width: px(44),
                                    height: px(44),
                                    margin: UiRect::all(px(2)),
                                    ..default()
                                },
                                BackgroundColor(Srgba::new(0.1, 0.2, 0.8, 1.0).into()),
                                Text::new("+"),
                            ));

                            parent.spawn((
                                IncomeValue(*resource),
                                Node {
                                    height: px(44),
                                    margin: UiRect::all(px(2)),
                                    ..default()
                                },
                                Text::new("Proft: 567$"), //TODO add next cost from  `amount` and `resource` in this town (from stop.goal_city_id)
                            ));

                            parent.spawn((
                                Button,
                                CaravanMenuButtons::KillTrade(stop.goal_city_id.clone(), *resource),
                                Node {
                                    width: px(44),
                                    height: px(44),
                                    margin: UiRect::all(px(2)),
                                    ..default()
                                },
                                BackgroundColor(Srgba::new(0.9, 0.1, 0.1, 1.0).into()),
                            ));
                        });
                }
            });
    }
}

fn caravan_button(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &CaravanMenuButtons),
        (Changed<Interaction>, With<Button>),
    >,
    mut hudNode: Query<Entity, With<CaravanCityUINode>>,
    //mut menu_state: ResMut<NextState<StrategicState>>,
    selected_caravan: Res<SelectedCaravan>,
    mut caravans: Query<&mut Caravan>,
    mut window_state: ResMut<NextState<StrategicState>>,
) {
    let Ok(mut selected_caravan) = caravans.get_mut(selected_caravan.0) else {
        return;
    };

    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                CaravanMenuButtons::NewStop => {
                    window_state.set(StrategicState::DestinationPicker);
                }
                CaravanMenuButtons::AddTradeToStop(stop_name) => {
                    let mut order = &mut selected_caravan
                        .orders
                        .iter_mut()
                        .find(|order| order.goal_city_id == *stop_name)
                        .expect(format!("Couldn't find city named {}", stop_name).as_str())
                        .trade_order;
                    for resource in Resources::all_resources() {
                        match order.entry(resource) {
                            Entry::Occupied(_) => continue,
                            Entry::Vacant(e) => {
                                e.insert(0);
                                break;
                            }
                        }
                    }
                }
                CaravanMenuButtons::RemoveStop(stop_name) => {
                    selected_caravan
                        .orders
                        .retain(|position| position.goal_city_id != *stop_name);
                }
                CaravanMenuButtons::IncTradeAmount(city_id, resource) => {
                    *selected_caravan
                        .orders
                        .iter_mut()
                        .find(|order| order.goal_city_id == *city_id)
                        .expect(format!("Couldn't find city named {}", city_id).as_str())
                        .trade_order
                        .get_mut(resource) //Should never call a undefined resource
                        .expect("Couldn't find resource, should never happen") += 1;
                }
                CaravanMenuButtons::DecTradeAmount(city_id, resource) => {
                    *selected_caravan
                        .orders
                        .iter_mut()
                        .find(|order| order.goal_city_id == *city_id)
                        .expect(format!("Couldn't find city named {}", city_id).as_str())
                        .trade_order
                        .get_mut(resource) //Should never call a undefined resource
                        .expect("Couldn't find resource, should never happen") -= 1;
                }
                CaravanMenuButtons::KillTrade(city_id, resource) => {
                    selected_caravan
                        .orders
                        .iter_mut()
                        .find(|order| order.goal_city_id == *city_id)
                        .expect(format!("Couldn't find city named {}", city_id).as_str())
                        .trade_order
                        .remove(resource);
                }
                CaravanMenuButtons::ChangeTrade(city_id, resource) => {
                    for entity in hudNode.iter() {
                        let mut order: BTreeSet<_> = selected_caravan
                            .orders
                            .iter()
                            .find(|order| order.goal_city_id == *city_id)
                            .expect(format!("Couldn't find city named {}", city_id).as_str())
                            .trade_order
                            .keys()
                            .cloned()
                            .collect();

                        let resources: BTreeSet<_> =
                            Resources::all_resources().into_iter().collect();

                        commands.entity(entity).with_children(|parent| {
                            parent
                                .spawn((
                                    // Scrolling list
                                    ZIndex(10),
                                    Node {
                                        flex_direction: FlexDirection::Column,
                                        align_self: AlignSelf::Stretch,
                                        height: px(200),
                                        width: px(400),
                                        overflow: Overflow::scroll_y(), // n.b.
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
                                ))
                                .with_children(|parent| {
                                    for &res in resources.difference(&order) {
                                        parent.spawn((
                                            Button,
                                            CaravanMenuButtons::ChangeTradeConfirm(
                                                city_id.clone(),
                                                *resource,
                                                res,
                                            ),
                                            Node {
                                                min_height: px(32),
                                                max_height: px(32),
                                                ..default()
                                            },
                                            children![(Text::new(res.get_name()))],
                                        ));
                                    }
                                });
                        });
                    }
                }
                CaravanMenuButtons::ChangeTradeConfirm(city_id, from_res, to_res) => {
                    for entity in hudNode.iter() {
                        commands.entity(entity).despawn_children();
                    }

                    if to_res != from_res {
                        let order = selected_caravan
                            .orders
                            .iter_mut()
                            .find(|order| &order.goal_city_id == city_id)
                            .expect(format!("Couldn't find city named {}", city_id).as_str());
                        if order.trade_order.contains_key(to_res) {
                            println!("You already are selling this");
                        } else {
                            let value = order.trade_order.remove(from_res).unwrap();
                            order.trade_order.insert(*to_res, value);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn update_caravan_order_idx(
    mut caravan_query: Query<(&mut Caravan), (Changed<Caravan>)>
) {
    for mut caravan in caravan_query {
        if caravan.orders[caravan.order_idx].goal_city_id == caravan.position_city_id && caravan.orders.len() > 1 {
            caravan.order_idx = (caravan.order_idx + 1) % caravan.orders.len();
        }
    }
}

fn caravan_destination_buttons(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &CityNodeMarker),
        (Changed<Interaction>, With<Button>),
    >,
    city_data_query: Query<&CityData>,
    selected_caravan: Res<SelectedCaravan>,
    mut caravans: Query<&mut Caravan>,
    mut window_state: ResMut<NextState<StrategicState>>,
) {
    let Ok(mut selected_caravan) = caravans.get_mut(selected_caravan.0) else {
        error!("Selected caravan doesn't exist");
        return;
    };

    for (interaction, city_entity) in &interaction_query {
        if let Ok(city) = city_data_query.get(city_entity.0) {
            println!("Found a city that was touched");
            if *interaction == Interaction::Pressed {
                println!("Found a city that was clicked");
                selected_caravan.orders.push(Order {
                    goal_city_id: city.id.clone(),
                    ..default()
                });
                window_state.set(StrategicState::HUDOpen);
            }
        } else {
            error!("Clicked a non existing city");
            return;
        };
    }
}

fn wares_menu(mut commands: Commands, mut sylt: Sylt, town: Res<SelectedCity>) {
    let window = popup_window(&mut commands, FlexDirection::Row);

    //Basic and exotic mats
    commands.entity(window).with_children(|parent| {
        //Basic and exotic mats
        let city_data = &town.0;
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
                            height: percent(70),
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
                            vec![
                                Resources::Food,
                                Resources::Plants,
                                Resources::CommonOre,
                                Resources::RareOre,
                                Resources::Lumber,
                                Resources::Stone,
                                Resources::Water,
                                Resources::Glass,
                                Resources::Coal,
                            ],
                            "Basic materials".to_string(),
                            &city_data,
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
                            vec![Resources::Drugs, Resources::Slaves, Resources::Vitae],
                            "Illegal materials".to_string(),
                            &city_data,
                            &mut sylt,
                        );
                    });

                parent
                    .spawn((
                        Node {
                            width: percent(100),
                            height: percent(55),
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
                            vec![
                                Resources::RefinedValuables,
                                Resources::CommonAlloys,
                                Resources::Textiles,
                                Resources::ManufacturedGoods,
                                Resources::Medicines,
                                Resources::Reagents,
                                Resources::Machinery,
                            ],
                            "Advanced materials".to_string(),
                            &city_data,
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
                            vec![
                                Resources::SimpleLabour,
                                Resources::Military,
                                Resources::Transportation,
                                Resources::Luxuries,
                                Resources::ComplexLabour,
                            ],
                            "Services".to_string(),
                            &city_data,
                            &mut sylt,
                        );
                    });

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
                            vec![
                                Resources::ExoticAlloys,
                                Resources::Spellwork,
                                Resources::Artifacts,
                            ],
                            "Exotic materials".to_string(),
                            &city_data,
                            &mut sylt,
                        );
                    });
            });
    });
}

#[derive(Component)]
enum HudButton {
    KillHud,
    EconomyTabAction,
    BuldingTabAction,
    OperationAction,
}

#[derive(Component)]
enum PopupButton {
    KillHud,
    BuldingTabAction,
}

fn create_resource_list(
    parent: &mut ChildSpawnerCommands,
    resources: Vec<Resources>,
    box_name: String,
    town: &CityData,
    mut sylt: &mut Sylt,
) {
    //out.push((resource, data[resource], CALCULATE(data[resource])));
    parent.spawn((
        Node {
            width: percent(100),
            height: px(40),
            ..default()
        },
        Text::new(box_name.clone()),
    ));

    for resource in resources {
        create_resource_icon(
            parent,
            resource,
            town.get_resource_value(&resource),
            &mut sylt,
        );
    }
}

fn create_resource_icon(
    parent: &mut ChildSpawnerCommands,
    resource: Resources,
    cost: f64,
    //    amount: usize,
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
            (Text::new(resource.get_name())),
            (Text::new(format!("{:.2}$", cost)),)
        ],
    ));
}

#[derive(Clone, Default, Eq, PartialEq, Hash, Component)]
pub struct BottomBar;

pub fn city_hud_setup(mut commands: Commands, selected_city: ResMut<SelectedCity>) {
    let city = selected_city.0.clone();
    //Map quit upon click
    commands.spawn((
        BottomBar,
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
        BottomBar,
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

/// Injects scroll events into the UI hierarchy.
fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= 32.;
        }

        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }

        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(Scroll { entity, delta });
            }
        }
    }
}

/// UI scrolling event.
#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
struct Scroll {
    entity: Entity,
    /// Scroll delta in logical coordinates.
    delta: Vec2,
}

fn on_scroll_handler(
    mut scroll: On<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    let delta = &mut scroll.delta;
    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.x > 0. {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.
        };

        if !max {
            scroll_position.x += delta.x;
            // Consume the X portion of the scroll delta.
            delta.x = 0.;
        }
    }

    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.y > 0. {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.
        };

        if !max {
            scroll_position.y += delta.y;
            // Consume the Y portion of the scroll delta.
            delta.y = 0.;
        }
    }

    // Stop propagating when the delta is fully consumed.
    if *delta == Vec2::ZERO {
        scroll.propagate(false);
    }
}
