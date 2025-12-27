use super::strategic_hud::PopupHUD;
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

#[derive(Resource, Deref)]
struct BuildinTable(HashMap<String, Building>);

#[derive(Resource, Default)]
pub struct PlayerStats {
    pub caravans: Vec<Caravan>,
    pub money: isize,
}

#[derive(Resource)]
pub struct SelectedCaravan(pub Caravan);

#[derive(Clone, Default, Eq, PartialEq, Debug, Hash)]
pub struct Caravan {
    pub orders: Vec<Order>,
    pub position_city_id: String,
    pub cargo: Vec<(Resources, usize)>,
}

#[derive(Clone, Default, Eq, PartialEq, Debug, Hash)]
pub struct Order {
    goal_city_id: String,
    trade_order: Vec<(Resources, isize)>,
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
        money: 5000,
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

use super::city_graph::Node as CityNode;
use super::tooltip::Tooltips;
fn spawn_city_ui_nodes(
    mut commands: Commands,
    graph_nodes: Query<(Entity, &CityNode, &super::city_graph::CityTypeComponent)>,
    mut sylt: Sylt,
    mut rng: ResMut<GlobalRng>,
) {
    for (ent, node, city_data) in graph_nodes {
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
            ImageNode::new(sylt.get_image("town_ui_icon")),
            BackgroundColor(Srgba::new(1.0, 0.1, 0.1, 0.3).into()),
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
                    Text::new("hello\nbevy!"),
                    TextShadow::default(),
                    // Set the justification of the Text
                    TextLayout::new_with_justify(Justify::Center),
                    // Set the style of the Node itself.
                    Node { ..default() }
                ),
                (
                    Text::new("hello\nbevy!"),
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

#[derive(Component, Default, Clone, Debug)]
pub struct CityData {
    pub id: String,
    pub race: BuildingType,
    pub population: u8,
    pub buildings_t1: Vec<(String, Faction)>,
    pub buildings_t2: Vec<(String, Faction)>,
    pub buildings_t3: Vec<(String, Faction)>,
    pub buildings_t4: Vec<(String, Faction)>,
    pub buildings_t5: Vec<(String, Faction)>,
}

impl CityData {
    pub fn new(race: BuildingType, tier: u8, mut rng: &mut ResMut<GlobalRng>) -> CityData {
        let buildings_per_tier = match tier {
            1 => { (1, 0, 0, 0, 0) },
            2 => { (1, 1, 0, 0, 0) },
            3 => { (2, 1, 1, 0, 0) },
            4 => { (2, 2, 1, 1, 0) },
            5 => { (3, 2, 2, 1, 1) },
            _ => { panic!("Tried to generate a city of tier {:?}", tier) }
        };
        let (mut t1, mut t2, mut t3, mut t4, mut t5) = (vec!(), vec!(), vec!(), vec!(), vec!());

        for i in 0..buildings_per_tier.0 {
            t1.push(((market::gen_random_building(1, &mut rng, race)), Faction::Neutral));
        }

        for i in 0..buildings_per_tier.1 {
            t2.push(((market::gen_random_building(2, &mut rng, race)), Faction::Neutral));
        }

        for i in 0..buildings_per_tier.2 {
            t3.push(((market::gen_random_building(3, &mut rng, race)), Faction::Neutral));
        }

        for i in 0..buildings_per_tier.3 {
            t4.push(((market::gen_random_building(4, &mut rng, race)), Faction::Neutral));
        }

        for i in 0..buildings_per_tier.4 {
            t5.push(((market::gen_random_building(5, &mut rng, race)), Faction::Neutral));
        }

        CityData {
            id: super::namelists::generate_city_name(race, &mut rng),
            race: race,
            population: tier,
            buildings_t1: t1,
            buildings_t2: t2,
            buildings_t3: t3,
            buildings_t4: t4,
            buildings_t5: t5
        }
    }
}

//#[derive(Component)]
//struct Market {
//    population: u8,
//    districts: Vec<DistrictType>,
//}

fn city_interaction_system(
    mut interaction_query: Query<(&Interaction, &CityData), Changed<Interaction>>,
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
