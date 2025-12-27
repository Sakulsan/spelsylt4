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

#[derive(Resource, Deref, Debug)]
pub struct BuildinTable(HashMap<String, Building>);

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
    pub goal_city_id: String,
    pub trade_order: Vec<(Resources, isize)>,
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
        let capitals = vec![
            "Great Lancastershire",
            //"Jewel of All Creation", These capitals aren't represented on the map  yet.
            //"Terez-e-Palaz",
            "Tevet Pekhep Dered",
        ];
        let mut image = if city_data.0.population < 3 {
            ImageNode::new(sylt.get_image("town_ui_icon"))
        } else {
            ImageNode::new(sylt.get_image("town_map_icon"))
        };
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
                width: px(16),
                height: px(16),
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
    pub market: HashMap<Resources, isize>,
    pub tier_up_counter: u8,
}

impl CityData {
    pub fn new(race: BuildingType, tier: u8, mut rng: &mut ResMut<GlobalRng>) -> CityData {
        let buildings_per_tier = match tier {
            1 => (1, 0, 0, 0, 0),
            2 => (1, 1, 0, 0, 0),
            3 => (2, 1, 1, 0, 0),
            4 => (2, 2, 1, 1, 0),
            5 => (3, 2, 2, 1, 1),
            _ => {
                panic!("Tried to generate a city of tier {:?}", tier)
            }
        };
        let (mut t1, mut t2, mut t3, mut t4, mut t5) = (vec![], vec![], vec![], vec![], vec![]);

        for i in 0..buildings_per_tier.0 {
            t1.push((
                (market::gen_random_building(1, &mut rng, race)),
                Faction::Neutral,
            ));
        }

        for i in 0..buildings_per_tier.1 {
            t2.push((
                (market::gen_random_building(2, &mut rng, race)),
                Faction::Neutral,
            ));
        }

        for i in 0..buildings_per_tier.2 {
            t3.push((
                (market::gen_random_building(3, &mut rng, race)),
                Faction::Neutral,
            ));
        }

        for i in 0..buildings_per_tier.3 {
            t4.push((
                (market::gen_random_building(4, &mut rng, race)),
                Faction::Neutral,
            ));
        }

        for i in 0..buildings_per_tier.4 {
            t5.push((
                (market::gen_random_building(5, &mut rng, race)),
                Faction::Neutral,
            ));
        }

        let mut market = HashMap::new();
        for res in Resources::all_resources() {
            market.insert(res, 0);
        }

        CityData {
            id: super::namelists::generate_city_name(race, &mut rng),
            race: race,
            population: tier,
            buildings_t1: t1,
            buildings_t2: t2,
            buildings_t3: t3,
            buildings_t4: t4,
            buildings_t5: t5,
            market: market,
            tier_up_counter: 0
        }
    }

    pub fn get_resource_value(&self, res: &Resources) -> f64 {
        let total = self.market.get(res).expect(format!("tried to find resource {:?} but the resource was missing", res).as_str());
        let sigmoid = 2.0/(1.0 + (std::f64::consts::E).powf(*total as f64 / 200.0)) * res.get_base_value() as f64;
        sigmoid.max(0.3)
    }

    pub fn available_commodities(&self, building_table: &Res<BuildinTable>) -> Vec<Resources> {
        let mut resources: HashMap<Resources, isize> = HashMap::new();
        macro_rules! get_outputs {
            ($list:expr) => {
                for b in &$list {
                    if b.1 != Faction::Neutral { continue; }
                    for (res, amount) in &building_table.0.get(&b.0).expect(format!("Couldn't retrieve value for {:?}", &b.0).as_str()).input {
                        resources.insert(*res, resources.get(res).or_else(|| -> Option<&isize> {Some(&0)}).expect(format!("bruh value {:?}", res).as_str()) + amount);
                    }
                    for (res, amount) in &building_table.0.get(&b.0).expect(format!("Couldn't retrieve value for {:?}", &b.0).as_str()).output {
                        resources.insert(*res, resources.get(res).or_else(|| -> Option<&isize> {Some(&0)}).expect(format!("bruh value {:?}", res).as_str()) - amount);
                    }
                }
            };
        }

        get_outputs!(self.buildings_t1);
        get_outputs!(self.buildings_t2);
        get_outputs!(self.buildings_t3);
        get_outputs!(self.buildings_t4);
        get_outputs!(self.buildings_t5);

        resources.iter().filter(|(k, v)| v >= &&0).map(|(k, v)| *k).collect::<Vec<Resources>>()
    }

    pub fn update_market(&mut self, building_table: &Res<BuildinTable>) {
        macro_rules! update_market_over_buildings {
            ($list:expr) => {
                for b in &$list {
                    if b.1 != Faction::Neutral { continue; }
                    for (res, amount) in &building_table.0.get(&b.0).expect(format!("Couldn't retrieve value for {:?}", &b.0).as_str()).input {
                        self.market.insert(*res, self.market[&res] - amount);
                    }
                    for (res, amount) in &building_table.0.get(&b.0).expect(format!("Couldn't retrieve value for {:?}", &b.0).as_str()).output {
                        self.market.insert(*res, self.market[&res] + amount);
                    }
                }
            };
        }

        update_market_over_buildings!(self.buildings_t1);
        update_market_over_buildings!(self.buildings_t2);
        update_market_over_buildings!(self.buildings_t3);
        update_market_over_buildings!(self.buildings_t4);
        update_market_over_buildings!(self.buildings_t5);

        let match_condition = self.population;

        let mut tier_up = |condition: bool| {
            if condition {
                if self.tier_up_counter == 5 {
                    self.tier_up_counter = 0;
                    self.population += 1;
                } else {
                    self.tier_up_counter += 1;
                }
            } else {
                self.tier_up_counter = 0;
            }
        };

        match match_condition {
            1 => {
                tier_up(self.market.insert(Resources::Food, self.market[&Resources::Food] - 5).expect("error in city market") - 5 >= 0 &&
                self.market.insert(Resources::Water, self.market[&Resources::Water] - 3).expect("error in city market") - 3 >= 0 &&
                self.market.insert(Resources::Lumber, self.market[&Resources::Lumber] - 2).expect("error in city market") - 2 >= 0);
                self.market.insert(Resources::SimpleLabour, self.market[&Resources::SimpleLabour] + 5);
            },
            2 => {
                tier_up(self.market.insert(Resources::Food, self.market[&Resources::Food] - 15).expect("error in city market") - 15 >= 0 &&
                self.market.insert(Resources::Water, self.market[&Resources::Water] - 10).expect("error in city market") - 10 >= 0 &&
                self.market.insert(Resources::Lumber, self.market[&Resources::Lumber] - 5).expect("error in city market") - 5 >= 0 &&
                self.market.insert(Resources::Stone, self.market[&Resources::Stone] - 3).expect("error in city market") - 3 >= 0 &&
                self.market.insert(Resources::Glass, self.market[&Resources::Glass] - 3).expect("error in city market") - 3 >= 0 &&
                self.market.insert(Resources::Textiles, self.market[&Resources::Textiles] - 3).expect("error in city market") - 3 >= 0);
                self.market.insert(Resources::SimpleLabour, self.market[&Resources::SimpleLabour] + 20);
                self.market.insert(Resources::ComplexLabour, self.market[&Resources::ComplexLabour] + 5);
            },
            3 => {
                tier_up(self.market.insert(Resources::Food, self.market[&Resources::Food] - 20).expect("error in city market") - 20 >= 0 &&
                self.market.insert(Resources::Water, self.market[&Resources::Water] - 15).expect("error in city market") - 15 >= 0 &&
                self.market.insert(Resources::Lumber, self.market[&Resources::Lumber] - 10).expect("error in city market") - 10 >= 0 &&
                self.market.insert(Resources::Stone, self.market[&Resources::Stone] - 6).expect("error in city market") - 6 >= 0 &&
                self.market.insert(Resources::Glass, self.market[&Resources::Glass] - 6).expect("error in city market") - 6 >= 0 &&
                self.market.insert(Resources::Textiles, self.market[&Resources::Textiles] - 6).expect("error in city market") - 6 >= 0 &&
                self.market.insert(Resources::Medicines, self.market[&Resources::Medicines] - 3).expect("error in city market") - 3 >= 0 &&
                self.market.insert(Resources::ManufacturedGoods, self.market[&Resources::ManufacturedGoods] - 3).expect("error in city market") - 3 >= 0 &&
                self.market.insert(Resources::Luxuries, self.market[&Resources::Luxuries] - 15).expect("error in city market") - 15 >= 0 &&
                self.market.insert(Resources::Transportation, self.market[&Resources::Transportation] - 15).expect("error in city market") - 15 >= 0);
                self.market.insert(Resources::Drugs, self.market[&Resources::Drugs] - 5).expect("error in city market");
                self.market.insert(Resources::Slaves, self.market[&Resources::Slaves] - 5).expect("error in city market");
                self.market.insert(Resources::SimpleLabour, self.market[&Resources::SimpleLabour] + 45);
                self.market.insert(Resources::ComplexLabour, self.market[&Resources::ComplexLabour] + 20);
            },
            4 => {
                tier_up(self.market.insert(Resources::Food, self.market[&Resources::Food] - 50).expect("error in city market") - 50 >= 0 &&
                self.market.insert(Resources::Water, self.market[&Resources::Water] - 30).expect("error in city market") - 30 >= 0 &&
                self.market.insert(Resources::Lumber, self.market[&Resources::Lumber] - 20).expect("error in city market") - 20 >= 0 &&
                self.market.insert(Resources::Stone, self.market[&Resources::Stone] - 15).expect("error in city market") - 15 >= 0 &&
                self.market.insert(Resources::Glass, self.market[&Resources::Glass] - 15).expect("error in city market") - 15 >= 0 &&
                self.market.insert(Resources::Textiles, self.market[&Resources::Textiles] - 15).expect("error in city market") - 15 >= 0 &&
                self.market.insert(Resources::Medicines, self.market[&Resources::Medicines] - 10).expect("error in city market") - 10 >= 0 &&
                self.market.insert(Resources::ManufacturedGoods, self.market[&Resources::ManufacturedGoods] - 10).expect("error in city market") - 10 >= 0 &&
                self.market.insert(Resources::Luxuries, self.market[&Resources::Luxuries] - 25).expect("error in city market") - 25 >= 0 &&
                self.market.insert(Resources::Transportation, self.market[&Resources::Transportation] - 25).expect("error in city market") - 25 >= 0 &&
                self.market.insert(Resources::Military, self.market[&Resources::Military] - 15).expect("error in city market") - 15 >= 0);
                self.market.insert(Resources::Drugs, self.market[&Resources::Drugs] - 10).expect("error in city market");
                self.market.insert(Resources::Slaves, self.market[&Resources::Slaves] - 10).expect("error in city market");
                self.market.insert(Resources::Vitae, self.market[&Resources::Vitae] - 2).expect("error in city market");
                self.market.insert(Resources::SimpleLabour, self.market[&Resources::SimpleLabour] + 80);
                self.market.insert(Resources::ComplexLabour, self.market[&Resources::ComplexLabour] + 45);
            },
            5 => {
                self.market.insert(Resources::Food, self.market[&Resources::Food] - 100);
                self.market.insert(Resources::Water, self.market[&Resources::Water] - 50);
                self.market.insert(Resources::Lumber, self.market[&Resources::Lumber] - 40);
                self.market.insert(Resources::Stone, self.market[&Resources::Stone] - 30);
                self.market.insert(Resources::Glass, self.market[&Resources::Glass] - 30);
                self.market.insert(Resources::Textiles, self.market[&Resources::Textiles] - 20);
                self.market.insert(Resources::Medicines, self.market[&Resources::Medicines] - 20);
                self.market.insert(Resources::ManufacturedGoods, self.market[&Resources::ManufacturedGoods] - 20);
                self.market.insert(Resources::Luxuries, self.market[&Resources::Luxuries] - 60);
                self.market.insert(Resources::Transportation, self.market[&Resources::Transportation] - 60);
                self.market.insert(Resources::Military, self.market[&Resources::Military] - 50);
                self.market.insert(Resources::Drugs, self.market[&Resources::Drugs] - 20).expect("error in city market");
                self.market.insert(Resources::Slaves, self.market[&Resources::Slaves] - 20).expect("error in city market");
                self.market.insert(Resources::Vitae, self.market[&Resources::Vitae] - 5).expect("error in city market");
                self.market.insert(Resources::SimpleLabour, self.market[&Resources::SimpleLabour] + 125);
                self.market.insert(Resources::ComplexLabour, self.market[&Resources::ComplexLabour] + 80);
            }
            _ => { panic!("Tried to update markets on a city of tier {:?}", self.population) }
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
