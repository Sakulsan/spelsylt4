use super::strategic_hud::PopupHUD;
use super::city_data::*;
use crate::game::city_graph::{CityGraph, get_path, Node as CityNode};
use crate::game::market;
use crate::prelude::*;

use super::market::*;
use crate::GameState;
use std::collections::HashMap;

use bevy_ui_anchor::{AnchorPoint, AnchorUiConfig, AnchoredUiNodes};

// This plugin will contain the game. In this case, it's just be a screen that will
// display the current settings for 5 seconds before returning to the menu
#[derive(Resource, Deref)]
pub struct SelectedCity(pub CityData);

#[derive(Resource, Deref, Debug)]
pub struct BuildinTable(pub HashMap<String, Building>);

#[derive(Resource, Default)]
pub struct PlayerStats {
    pub caravans: Vec<Caravan>,
    pub money: f64,
}

#[derive(Resource)]
pub struct SelectedCaravan(pub Caravan);

#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct Caravan {
    pub orders: Vec<Order>,
    pub order_idx: usize,
    pub time_travelled: usize,
    pub position_city_id: String,
    pub cargo: Vec<(Resources, usize)>,
}

#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct Order {
    pub goal_city_id: String,
    pub trade_order: HashMap<Resources, isize>,
}

impl Caravan {
    pub fn update_orders(&mut self, city: &Res<CityGraph>, mut nodes: &mut Query<(&CityNode, &mut CityData)>, building_table: &Res<BuildinTable>, player_money: &mut f64) {
        if self.orders.len() == 0 { return }
        let current_node = nodes
                                                .iter()
                                                .filter(|(_, y)| y.id == self.position_city_id)
                                                .next()
                                                .expect(format!("Caravan located in city {:?} that doesn't exist", self.position_city_id).as_str());
        let next_node = nodes
                                                .iter()
                                                .filter(|(_, y)| y.id == self.orders[self.order_idx].goal_city_id)
                                                .next()
                                                .expect(format!("Caravan trying to get to city {:?} that doesn't exist", self.orders[self.order_idx].goal_city_id).as_str());
        let (_, path) = get_path(city, current_node.0.0, next_node.0.0);
        if path.len() > 1 {
            let next_city = nodes.get(path[1]).expect("Caravan travelling to a city that doesnt exist");
            self.position_city_id = next_city.1.id.to_string();
        }
        let mut current_city = nodes.get_mut(path[0]).expect("failed to get a path in order updater????");
        if self.orders[self.order_idx].goal_city_id == current_city.1.id {
            let available_commodies = current_city.1.available_commodities(&building_table);
            for (trade, amount) in self.orders[self.order_idx].trade_order.clone() {
                if amount > 0 && available_commodies.contains(&trade) {
                    let amount_available = current_city.1.market[&trade];
                    let amount_bought = amount.abs().min(amount_available);
                    let price = current_city.1.get_bulk_buy_price(&trade, amount_bought as usize);
                    *player_money -= price;
                    current_city.1.market.insert(trade, amount_available - amount_bought);
                }
                if amount > 0 {
                    let amount_available = current_city.1.market[&trade];
                    let amount_sold = amount.abs();
                    let price = current_city.1.get_bulk_sell_price(&trade, amount_sold as usize);
                    *player_money += price;
                    current_city.1.market.insert(trade, amount_available + amount_sold);
                }
            }

            self.order_idx = (self.order_idx + 1) % self.orders.len();
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::Game),
        (crate::kill_music, spawn_map_sprite, spawn_city_ui_nodes),
    )
    .insert_resource(SelectedCity(CityData {
        id: "Placeholder".to_string(),
        ..default()
    }))
    .insert_resource(PlayerStats {
        money: 5000.0,
        ..default()
    })
    .insert_resource(SelectedCaravan(Caravan { ..default() }))
    .insert_resource(BuildinTable(super::market::gen_building_tables()))
    .init_state::<StrategicState>()
    .add_systems(
        Update,
        (update_caravan_hud).run_if(resource_changed::<PlayerStats>),
    )
    .add_systems(
        Update,
        (
            city_interaction_system,
            check_turn_button,
            check_outline_button,
        )
            .run_if(in_state(PopupHUD::Off)),
    )
    .add_systems(Update, update_ui_nodes.run_if(in_state(GameState::Game)));
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum StrategicState {
    #[default]
    Map,
    HUDOpen,
    DestinationPicker,
}

fn spawn_map_sprite(mut commands: Commands, mut sylt: Sylt) {
    commands.spawn((
        Sprite {
            image: sylt.get_sprite("map").image,
            ..default()
        },
        DespawnOnExit(GameState::Game),
    ));

    //Next turn button
    commands.spawn((
        Button,
        TurnButton {},
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            right: px(0),
            width: px(128),
            height: px(64),
            border: UiRect::all(Val::Px(2.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BorderColor::all(Color::BLACK),
        DespawnOnExit(GameState::Game),
        BackgroundColor(Srgba::new(0.2, 0.8, 0.2, 1.0).into()),
        children![(Text::new("Next turn"))],
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

fn update_caravan_hud(
    caravan_box: Query<Entity, With<CaravanHudEntity>>,
    stats: Res<PlayerStats>,
    mut commands: Commands,
) {
    for caravan_box in caravan_box.iter() {
        commands.entity(caravan_box).despawn_children();
        commands.entity(caravan_box).with_children(|parent| {
            for caravan in stats.caravans.iter() {
                parent.spawn((
                    Button,
                    CaravanHudItem(caravan.clone()),
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
fn spawn_city_ui_nodes(
    mut commands: Commands,
    graph_nodes: Query<(Entity, &CityNode, &super::city_graph::CityTypeComponent)>,
    mut sylt: Sylt,
    mut rng: ResMut<GlobalRng>,
) {
    for (ent, node, city_data) in graph_nodes {
        let capitals = vec![
            "Great Lancastershire",
            //"Jewel of All Creation", These capitals aren't represented on the map  yet.
            //"Terez-e-Palaz",
            "Tevet Pekhep Dered",
        ];
        let mut image = ImageNode::new(sylt.get_image("town_ui_icon"));
        let mut background = BackgroundColor(Srgba::new(1.0, 0.1, 0.1, 0.3).into());
        let city_descriptor = match city_data.0.population {
            0..3 => format!("{:?} town", city_data.0.race),
            3..6 => format!("{:?} city", city_data.0.race),
            _ => format!(
                "GREAT AREA OF {:?} (error in tooltip code btw)",
                city_data.0.race
            ),
        };
        if capitals.contains(&city_data.0.id.as_str()) {
            image.color.set_alpha(0.0);
            background.0.set_alpha(0.0);
        }
        commands.entity(ent).insert(AnchoredUiNodes::spawn_one((
            AnchorUiConfig {
                anchorpoint: AnchorPoint::middle(),
                ..default()
            },
            Button,
            city_data.0.clone(),
            Transform::from_xyz(0., 0.0, 1.0),
            Node {
                width: px(32),
                height: px(32),
                ..default()
            },
            image,
            background,
            related!(
                Tooltips[(
                    Text::new(city_data.0.id.clone()),
                    TextShadow::default(),
                    // Set the justification of the Text
                    TextLayout::new_with_justify(Justify::Center),
                    // Set the style of the Node itself.
                    Node { ..default() },
                    BackgroundColor(Srgba::new(0.05, 0.05, 0.05, 1.0).into()),
                ),
                (
                    Text::new(format!("Tier: {}", city_data.0.population)),
                    TextShadow::default(),
                    // Set the justification of the Text
                    TextLayout::new_with_justify(Justify::Center),
                    // Set the style of the Node itself.
                    Node { ..default() }
                ),
                (
                    Text::new(city_descriptor),
                    TextShadow::default(),
                    // Set the justification of the Text
                    TextLayout::new_with_justify(Justify::Center),
                    // Set the style of the Node itself.
                    Node { ..default() }
                )]
            ),
        )));
    }
}

fn update_ui_nodes(
    nodes: Query<(&mut UiTransform, &CityNode)>,
    camera: Option<Single<(&GlobalTransform, &Camera), With<Camera2d>>>,
) {
    return;

    let Some((cam_trans, cam)) = camera.map(|c| c.into_inner()) else {
        error!("Missing camera!");
        return;
    };

    for (mut transform, node) in nodes {
        let Some(ndc_pos) = cam.world_to_ndc(cam_trans, node.1.extend(0.0)) else {
            continue;
        };
        let ndc_pos = ndc_pos / 2.0 + Vec3::splat(0.5);
        let ndc_pos = Vec2::new(ndc_pos.x, 1.0 - ndc_pos.y) * 100.0;

        transform.translation.x = Val::Vw(ndc_pos.x);
        transform.translation.y = Val::Vh(ndc_pos.y);
    }
}

//#[derive(Component)]
//struct Demographic<T>();

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum Faction {
    #[default]
    Neutral,
    Player(usize),
}

#[derive(Component, Default, Clone, Debug)]
struct TurnButton {}

#[derive(Component, Default, Clone, Debug)]
struct CaravanHudEntity {}
#[derive(Component, Default, Clone, Debug)]
struct CaravanHudItem(Caravan);

//#[derive(Component)]
//struct Market {
//    population: u8,
//    districts: Vec<DistrictType>,
//}

fn city_interaction_system(
    mut interaction_query: Query<(&Interaction, &CityData), Changed<Interaction>>,
    //ui_entities: Query<Entity, With<super::strategic_hud::PopUpItem>>,
    mut menu_state: ResMut<NextState<StrategicState>>,
    mut selected_city: ResMut<SelectedCity>,
    mut popupp_state: ResMut<NextState<PopupHUD>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (interaction, city) in &mut interaction_query {
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
            //Interaction::Hovered => *node_color = Srgba::new(1.0, 0.1, 0.1, 1.0).into(),
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
) {
    for (interaction, mut node_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                println!("New turn");
                commands.trigger(super::turn::TurnEnd);
            }
            Interaction::Hovered => {
                *node_color = BackgroundColor(Srgba::new(1.0, 0.1, 0.1, 1.0).into())
            }
            _ => *node_color = BackgroundColor(Srgba::new(0.2, 0.8, 0.2, 1.0).into()),
        }
    }
}

//Next turn button
fn check_outline_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut CaravanHudItem),
        Changed<Interaction>,
    >,

    mut tab_state: ResMut<NextState<PopupHUD>>,
    mut next_caravan: ResMut<SelectedCaravan>,
    mut commands: Commands,
) {
    for (interaction, mut node_color, caravan_data) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *next_caravan = SelectedCaravan(caravan_data.0.clone());
                tab_state.set(PopupHUD::Caravan);
            }
            Interaction::Hovered => {
                *node_color = BackgroundColor(Srgba::new(1.0, 0.1, 0.1, 1.0).into())
            }
            _ => *node_color = BackgroundColor(Srgba::new(0.8, 0.1, 0.1, 1.0).into()),
        }
    }
}
