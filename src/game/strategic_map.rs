use super::city_data::*;
use super::strategic_hud::{LockedCities, PopupHUD};
use super::turn::TurnEndSinglePlayer;
use crate::game::city_graph::{CityGraph, Node as CityNode, get_path};
use crate::game::turn::{TurnEndClient, TurnEndHost};
use crate::network::message::NetworkMessage;
use crate::network::message::{ClientMessage, PlayerId, ServerMessage};
use crate::{NetworkState, prelude::*};

use super::market::*;
use crate::GameState;
use std::collections::{BTreeMap, HashMap};

use bevy_egui::{EguiContext, EguiPrimaryContextPass, PrimaryEguiContext, egui};
use bevy_ui_anchor::{AnchorPoint, AnchorUiConfig, AnchoredUiNodes};
use serde::{Deserialize, Serialize};

// This plugin will contain the game. In this case, it's just be a screen that will
// display the current settings for 5 seconds before returning to the menu
#[derive(Reflect, Resource, Deref, DerefMut)]
pub struct SelectedCity(pub CityData);

#[derive(Reflect, Resource, Deref, Debug)]
pub struct BuildinTable(pub HashMap<String, Building>);

#[derive(Reflect, Component, Default)]
pub struct Player {
    pub player_id: PlayerId,
    pub money: f64,
}

#[derive(Reflect, Component, Default)]
pub struct ActivePlayer;

#[derive(Reflect, Component, Debug, Clone, Copy)]
#[relationship(relationship_target = Owns)]
pub struct BelongsTo(pub Entity);

#[derive(Reflect, Component, Debug)]
#[relationship_target(relationship = BelongsTo)]
pub struct Owns(Vec<Entity>);

#[derive(Reflect, Resource, Deref, DerefMut)]
pub struct SelectedCaravan(pub Entity);

#[derive(Reflect, Component, Clone, Default, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Caravan {
    pub orders: Vec<Order>,
    pub order_idx: usize,
    pub time_travelled: usize,
    pub position_city_id: String,
    pub cargo: HashMap<Resources, usize>,
}

#[derive(Clone, Reflect, Default, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Order {
    pub goal_city_id: String,
    pub trade_order: BTreeMap<Resources, (isize, bool)>,
}

impl Caravan {
    pub fn update_orders(
        _: On<TurnEndSinglePlayer>,
        players: Query<(&mut Player, &Owns)>,
        mut caravans: Query<&mut Caravan>,
        city: Res<CityGraph>,
        mut nodes: Query<(&CityNode, &mut CityData)>,
        building_table: Res<BuildinTable>,
    ) {
        for (mut player, owned_entities) in players {
            for ent in owned_entities.collection() {
                let Ok(mut caravan) = caravans.get_mut(*ent) else {
                    continue;
                };
                if caravan.orders.len() == 0 {
                    return;
                }
                let city_by_id = |id| {
                    nodes
                        .iter()
                        .find(|(_, city)| &city.id == id)
                        .unwrap_or_else(|| panic!("Attempted to get nonexistent city {id}"))
                };
                let current_node = city_by_id(&caravan.position_city_id);
                let next_node = city_by_id(&caravan.orders[caravan.order_idx].goal_city_id);
                let (_, path) = get_path(&city, current_node.0.0, next_node.0.0);
                info!(
                    "Nodes next to current node: {0:?}",
                    city.graph
                        .neighbors(current_node.0.0)
                        .map(|n| nodes.get(city.graph[n]).unwrap())
                        .map(|(_, data)| data.id.clone())
                        .collect::<Vec<String>>()
                );
                info!(
                    "Nodes next to next node: {0:?}",
                    city.graph
                        .neighbors(next_node.0.0)
                        .map(|n| nodes.get(city.graph[n]).unwrap())
                        .map(|(_, data)| data.id.clone())
                        .collect::<Vec<String>>()
                );

                let paths_mapped: Vec<String> = path
                    .iter()
                    .map(|n| nodes.get(*n).unwrap())
                    .map(|(_, data)| data.id.clone())
                    .collect();

                //info!("astar path: {:?}", paths_mapped);

                let mut current_city = nodes
                    .get_mut(path[0])
                    .expect("failed to get a path in order updater????");
                if path.len() > 1 {
                    current_city = nodes
                        .get_mut(path[1])
                        .expect("Caravan travelling to a city that doesnt exist");
                    caravan.position_city_id = current_city.1.id.to_string();
                    //info!("Caravan travels to {0:?}", current_city.1.id.to_string());
                }
                //info!(  "Caravan wants to get to {0:?}",caravan.orders[caravan.order_idx].goal_city_id);
                if caravan.orders[caravan.order_idx].goal_city_id == current_city.1.id {
                    let available_commodies = current_city.1.available_commodities(&building_table);
                    let cargo_access = caravan.cargo.clone();
                    info!("Caravan currently has {:?} stored", cargo_access);
                    for (trade, (amount, interacts_with_warehouse)) in
                        caravan.orders[caravan.order_idx].trade_order.clone()
                    {
                        println!("open market is set to: {interacts_with_warehouse}");
                        //Buy from market
                        if amount > 0 && interacts_with_warehouse {
                            if available_commodies.contains(&trade) {
                                let amount_available = current_city.1.market[&trade];
                                let amount_bought = amount.abs().min(amount_available);
                                let price = current_city
                                    .1
                                    .get_bulk_buy_price(&trade, amount_bought as usize);
                                info!("Caravan paid {0} for {2} {1}", price, trade.get_name(), amount_bought);
                                player.money -= price;
                                caravan.cargo.insert(
                                    trade,
                                    cargo_access.get(&trade).unwrap_or(&0) + amount_bought as usize,
                                );
                                info!("Caravan now has {0} {1}", caravan.cargo.get(&trade).unwrap_or(&0), &trade.get_name());
                                current_city
                                    .1
                                    .market
                                    .insert(trade, amount_available - amount_bought);
                            } else {
                                error!("Could not buy commodety");
                                continue;
                            }
                        // Take from warehouse
                        } else if amount > 0 {
                            let city_id = current_city.1.id.clone();
                            let Some(mut warehouse) =
                                current_city.1.warehouses.get_mut(&player.player_id)
                            else {
                                error!(
                                    "no warehouse of playerid {0} in {1}",
                                    player.player_id, current_city.1.id
                                );
                                continue;
                            };
                            let amount_available = warehouse
                                .get(&trade)
                                .expect(&format!("malformed warehouse in {0}", city_id))
                                .clone();

                            let amount_taken = amount.min(amount_available);

                            caravan.cargo.insert(
                                trade,
                                cargo_access.get(&trade).unwrap_or(&0) + amount_taken as usize,
                            );
                            warehouse.insert(trade, amount_available - amount_taken);
                        }
                        //Sell to market
                        if amount < 0 && interacts_with_warehouse {
                            let amount_available = current_city.1.market[&trade];
                            let amount_sold = amount
                                .abs()
                                .min(*cargo_access.get(&trade).unwrap_or(&0) as isize);
                            let price = current_city
                                .1
                                .get_bulk_sell_price(&trade, amount_sold as usize);
                            player.money += price;
                            caravan.cargo.insert(
                                trade,
                                cargo_access.get(&trade).unwrap_or(&0) - amount_sold as usize,
                            );
                            info!("Caravan sold {1} for {0}", price, trade.get_name());
                            current_city
                                .1
                                .market
                                .insert(trade, amount_available + amount_sold);
                        }
                        //Put into warehouse
                        else if amount < 0 {
                            let city_id = current_city.1.id.clone();
                            let Some(mut warehouse) =
                                current_city.1.warehouses.get_mut(&player.player_id)
                            else {
                                error!(
                                    "no warehouse of playerid {0} in {1}",
                                    player.player_id, current_city.1.id
                                );
                                continue;
                            };
                            let amount_available = warehouse
                                .get(&trade)
                                .expect(&format!("malformed warehouse in {0}", city_id))
                                .clone();

                            let amount_deposited = amount
                                .abs()
                                .min(*cargo_access.get(&trade).unwrap_or(&0) as isize);

                            caravan.cargo.insert(
                                trade,
                                cargo_access.get(&trade).unwrap_or(&0) - amount_deposited as usize,
                            );
                            warehouse.insert(trade, amount_available + amount_deposited);
                        }
                    }

                    info!(
                        "Caravan finished trading, new goal is {0:?}",
                        caravan.orders[(caravan.order_idx + 1) % caravan.orders.len()].goal_city_id
                    );
                    caravan.order_idx = (caravan.order_idx + 1) % caravan.orders.len();
                }
            }
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::Game),
        (
            crate::kill_music,
            spawn_map_sprite,
            spawn_city_ui_nodes,
            spawn_game_ost,
        ))
        .insert_resource(SelectedCity(CityData {
            id: "Placeholder".to_string(),
            ..default()
        }))
        .insert_resource(SelectedCaravan(Entity::PLACEHOLDER))
        .insert_resource(BuildinTable(super::market::gen_building_tables()))
        .init_state::<StrategicState>()
        .add_systems(
            Update,
            update_caravan_outliner.run_if(
                any_match_filter::<Changed<Caravan>>.or(resource_changed::<SelectedCaravan>),
            ),
        )
        .add_systems(
            Update,
            (
                city_interaction_system,
                check_turn_button,
                check_outline_button,
            )
                .run_if(in_state(GameState::Game))
                .run_if(in_state(PopupHUD::Off)),
        )
        .add_systems(
            Update,
            (make_caravan_ids, update_miku_cat, open_miku_cat).run_if(in_state(GameState::Game)),
        )
        .add_observer(Caravan::update_orders)
        .add_observer(on_city_updated);
}

#[derive(Resource, Deref, DerefMut, Serialize, Deserialize)]
pub struct CaravanIdTracker(pub u64);
impl CaravanIdTracker {
    fn next_id(&mut self) -> u64 {
        let id = self.0;
        self.0 += 1;
        id
    }
}

#[derive(Component, Clone, Copy, Deref, DerefMut, Serialize, Deserialize, Debug)]
pub struct CaravanId(pub u64);

fn make_caravan_ids(
    mut commands: Commands,
    query: Query<Entity, (Added<Caravan>, Without<CaravanId>)>,
    mut ids: ResMut<CaravanIdTracker>,
) {
    for entity in query {
        let next_id = ids.next_id();
        commands.entity(entity).insert(CaravanId(next_id));
    }
}

#[derive(Reflect, Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum StrategicState {
    #[default]
    Map,
    HUDOpen,
    DestinationPicker,
}

#[derive(Reflect, Component)]
struct MikuCaravanSlot(String);

fn open_miku_cat(
    interactions: Query<(Entity, &Interaction, &Caravan), (With<Caravan>, Changed<Interaction>)>,
    mut sel: ResMut<SelectedCaravan>,
    mut state: ResMut<NextState<PopupHUD>>,
) {
    for (ent, interaction, caravan) in interactions {
        if *interaction == Interaction::Pressed {
            **sel = ent;
            info!("Selecting {:?}", ent);
            info!("caravan: {:?}", caravan);
            state.set(PopupHUD::Caravan);
        }
    }
}

fn update_miku_cat(
    mut commands: Commands,
    caravans: Query<(Entity, &Caravan), Changed<Caravan>>,
    cities: Query<(Entity, &MikuCaravanSlot)>,
    mut sylt: Sylt,
) {
    let image = sylt.get_image("carrige_icon");

    for (c_ent, caravan) in caravans {
        let Some((city, _)) = cities
            .iter()
            .find(|(_, data)| data.0 == caravan.position_city_id)
        else {
            continue;
        };

        commands.entity(c_ent).insert((
            Button,
            ChildOf(city),
            Node {
                height: px(60.0),
                width: px(60.0),
                ..default()
            },
            ImageNode::new(image.clone()),
        ));
    }
}

pub fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player {
            player_id: 0,
            money: 5000.0,
        },
        ActivePlayer,
    ));
}

fn spawn_game_ost(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("music/Moneymoneymoney.ogg")),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            ..default()
        },
    ));
}

fn spawn_map_sprite(
    mut commands: Commands,
    mut sylt: Sylt,
    network_state: Res<State<NetworkState>>,
) {
    commands.spawn((
        Sprite {
            image: sylt.get_sprite("map").image,
            ..default()
        },
        DespawnOnExit(GameState::Game),
    ));

    //Next turn button
    commands.spawn((
        ZIndex(2),
        Button,
        TurnButton,
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            right: px(0),
            width: vw(20),
            height: px(64),
            border: UiRect::all(Val::Px(2.0)),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            ..default()
        },
        BorderColor::all(Color::BLACK),
        DespawnOnExit(GameState::Game),
        BackgroundColor(Srgba::new(0.1, 0.6, 0.1, 1.0).into()),
        children![(
            Node {
                width: percent(100),
                ..default()
            },
            TextLayout::new_with_justify(Justify::Center),
            Text::new(match network_state.get() {
                NetworkState::SinglePlayer => {
                    "Next turn"
                }
                NetworkState::Client => {
                    "End your turn"
                }
                NetworkState::Host => {
                    "End everyones turn"
                }
            })
        )],
    ));

    //Outliner menu
    commands.spawn((
        ZIndex(1),
        Node {
            position_type: PositionType::Absolute,
            top: vw(10),
            right: px(0),
            width: vw(20),
            height: vh(50),
            border: UiRect::all(Val::Px(2.0)),
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BorderColor::all(Color::BLACK),
        DespawnOnExit(GameState::Game),
        BackgroundColor(Srgba::new(0.2, 0.2, 0.2, 1.0).into()),
        children![
            (Text::new("Caravans")),
            (
                CaravanHudEntity {},
                Node {
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::FlexStart,
                    flex_direction: FlexDirection::Column,
                    ..default()
                }
            )
        ],
    ));
}

fn update_caravan_outliner(
    caravan_box: Query<Entity, With<CaravanHudEntity>>,
    player: Option<Single<Entity, With<ActivePlayer>>>,
    caravans: Query<(Entity, &Caravan, &BelongsTo)>,
    mut commands: Commands,
) {
    let Some(player) = player.map(|x| x.into_inner()) else {
        error!("No active player");
        return;
    };
    for caravan_box in caravan_box.iter() {
        commands.entity(caravan_box).despawn_children();
        commands.entity(caravan_box).with_children(|parent| {
            for (ent, caravan, owner) in caravans {
                if owner.0 != player {
                    continue;
                }
                parent.spawn((
                    Button,
                    CaravanHudItem(ent),
                    Node {
                        width: vw(20),
                        height: px(32),
                        margin: UiRect {
                            left: px(0),
                            right: px(0),
                            top: px(0),
                            bottom: px(4),
                        },
                        ..default()
                    },
                    BackgroundColor(Srgba::new(0.8, 0.1, 0.1, 1.0).into()),
                    Text::new(caravan.position_city_id.clone()),
                ));
            }
        });
    }
}

use super::tooltip::Tooltips;

#[derive(Reflect, Component)]
pub struct CityNodeMarker(pub(crate) Entity);
#[derive(Reflect, Component)]
pub struct CityImageMarker;

fn spawn_city_ui_nodes(
    mut commands: Commands,
    graph_nodes: Query<(Entity, &CityNode, &CityData)>,
    mut sylt: Sylt,
    _rng: ResMut<GlobalRng>,
) {
    for (ent, _node, city_data) in graph_nodes {
        let capitals = vec![
            "Great Lancastershire",
            "Jewel of All Creation",
            "Terez-e-Palaz",
            "Tevet Pekhep Dered",
        ];
        let mut image = ImageNode::new(sylt.get_image("town_ui_icon"));
        let mut background = BackgroundColor(Srgba::new(1.0, 0.1, 0.1, 0.3).into());
        let city_descriptor = match city_data.population {
            0..3 => format!("{:?} town", city_data.race),
            3..6 => format!("{:?} city", city_data.race),
            _ => format!(
                "GREAT AREA OF {:?} (error in tooltip code btw)",
                city_data.race
            ),
        };
        if capitals.contains(&city_data.id.as_str()) {
            image.color.set_alpha(0.0);
            background.0.set_alpha(0.0);
        }

        let text_node = |text: String| {
            (
                Text::new(text),
                TextLayout::new_with_justify(Justify::Center),
                Node { ..default() },
                BackgroundColor(Srgba::new(0.05, 0.05, 0.05, 1.0).into()),
            )
        };

        let city_ui_node = (
            Button,
            Node {
                width: px(32),
                height: px(32),
                ..default()
            },
            AnchorUiConfig {
                anchorpoint: AnchorPoint::middle(),
                ..default()
            },
            CityImageMarker,
            image,
            //background,
        );

        let miku_slot = (
            AnchorUiConfig {
                anchorpoint: AnchorPoint::bottomleft(),
                offset: Some(Vec3::new(10.0, 10.0, 0.0)),
                ..default()
            },
            Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::NoWrap,
                height: px(30.0),
                width: px(400.0),
                column_gap: px(10.0),
                ..default()
            },
            MikuCaravanSlot(city_data.id.clone()),
        );

        let clickable_node = (
            AnchorUiConfig {
                anchorpoint: AnchorPoint::middle(),
                ..default()
            },
            Button,
            CityNodeMarker(ent),
            BorderRadius::MAX,
            Node {
                width: px(32),
                height: px(32),
                ..default()
            },
            BackgroundColor(Srgba::new(0.2, 0.2, 0.2, 0.5).into()),
            related!(
                Tooltips[
                    text_node(city_data.id.clone()),
                    text_node(format!("Tier: {}", city_data.population)),
                    text_node(city_descriptor),
                ]
            ),
        );

        commands
            .entity(ent)
            .insert(related!(AnchoredUiNodes[miku_slot, city_ui_node, clickable_node]));
    }
}

#[derive(
    Reflect, Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States, Serialize, Deserialize,
)]
pub enum Faction {
    #[default]
    Neutral,
    Player(PlayerId),
}

#[derive(Component, Default, Clone, Debug)]
struct TurnButton;

#[derive(Component, Default, Clone, Debug)]
struct CaravanHudEntity {}
#[derive(Component, Clone, Debug)]
struct CaravanHudItem(Entity);

//#[derive(Component)]
//struct Market {
//    population: u8,
//    districts: Vec<DistrictType>,
//}

fn city_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &CityNodeMarker, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    city_data: Query<&CityData>,
    //ui_entities: Query<Entity, With<super::strategic_hud::PopUpItem>>,
    mut menu_state: ResMut<NextState<StrategicState>>,
    mut selected_city: ResMut<SelectedCity>,
    mut popupp_state: ResMut<NextState<PopupHUD>>,
    locked_cities: Res<LockedCities>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (interaction, city_id, mut node_color) in &mut interaction_query {
        let Ok(city) = city_data.get(city_id.0) else {
            continue;
        };

        match *interaction {
            Interaction::Pressed => {
                println!("Pressed the city {}", city.id);
                println!("The city is tier {}", city.population);

                /*                //Kills the old overlay hud
                                dbg!(ui_entities);
                                for ui_entity in ui_entities.iter() {
                                    println!("Confussion");
                                    commands.entity(ui_entity).despawn_children();
                                    //commands.entity(ui_entity).remove();
                                }
                */
                selected_city.0 = (*city).clone();
                menu_state.set(StrategicState::HUDOpen);
                popupp_state.set(PopupHUD::Off);

                commands.spawn((
                    AudioPlayer::new(asset_server.load(match selected_city.0.race {
                        BuildingType::Dwarven => "music/Dwarftowneffect.ogg",
                        BuildingType::Goblin => "music/Gnometowneffect.ogg",
                        BuildingType::Human => "music/Humanstowneffect.ogg",
                        BuildingType::Elven => "music/Elvestowneffect.ogg",
                        _ => {
                            panic!(
                                "Attempted to play city sound effect for non-existent city type."
                            )
                        }
                    })),
                    PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Once,
                        ..default()
                    },
                ));
            }
            Interaction::Hovered => {
                if locked_cities.any_locked(&city.id) {
                    *node_color = Srgba::new(1.0, 0.2, 0.2, 0.5).into()
                } else {
                    *node_color = Srgba::new(0.2, 1.0, 0.2, 0.5).into()
                }
            }
            Interaction::None => *node_color = Srgba::new(0.2, 0.2, 0.2, 0.5).into(),
            _ => {}
        }
    }
}

//Next turn button
fn check_turn_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<TurnButton>),
    >,
    mut commands: Commands,
    network_state: Res<State<NetworkState>>,
) {
    for (interaction, mut node_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => match network_state.get() {
                NetworkState::SinglePlayer => {
                    println!("Singleplayer: New turn");
                    commands.trigger(TurnEndSinglePlayer);
                }
                NetworkState::Client => {
                    println!("Client: sending orders");
                    commands.trigger(TurnEndClient);
                }
                NetworkState::Host => {
                    println!("Host: New turn");
                    commands.trigger(TurnEndHost);
                }
            },
            Interaction::Hovered => {
                *node_color = BackgroundColor(Srgba::new(0.6, 0.1, 0.1, 1.0).into())
            }
            _ => *node_color = BackgroundColor(Srgba::new(0.1, 0.6, 0.1, 1.0).into()),
        }
    }
}

//Next turn button
fn check_outline_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut CaravanHudItem),
        Changed<Interaction>,
    >,
    mut caravans: Query<&mut Node, With<Caravan>>,
    mut tab_state: ResMut<NextState<PopupHUD>>,
    mut next_caravan: ResMut<SelectedCaravan>,
) {
    for (interaction, mut node_color, caravan_data) in interaction_query.iter_mut() {
        let mut caravan_node = caravans.get_mut(caravan_data.0).unwrap();

        match *interaction {
            Interaction::Pressed => {
                next_caravan.0 = caravan_data.0;
                tab_state.set(PopupHUD::Caravan);
            }
            Interaction::Hovered => {
                caravan_node.width = px(120.0);
                caravan_node.height = px(120.0);
                *node_color = BackgroundColor(Srgba::new(1.0, 0.1, 0.1, 1.0).into())
            }
            Interaction::None => {
                caravan_node.width = px(60.0);
                caravan_node.height = px(60.0);
                *node_color = BackgroundColor(Srgba::new(0.8, 0.1, 0.1, 1.0).into())
            }
        }
    }
}

#[derive(Event, Deref, DerefMut)]
pub struct UpdatedCity(pub CityData);

fn on_city_updated(
    updated: On<UpdatedCity>,
    mut writer_server: MessageWriter<ServerMessage>,
    mut writer_client: MessageWriter<ClientMessage>,
    network_state: Res<State<NetworkState>>,
) {
    let updated_city = updated.clone();

    if *network_state == NetworkState::Client {
        writer_client.write(ClientMessage(NetworkMessage::CityUpdated { updated_city }));
    } else {
        writer_server.write(ServerMessage(NetworkMessage::CityUpdated { updated_city }));
    }
}
