use super::strategic_hud::PopupHUD;
use crate::prelude::*;
use bevy::audio::Volume;

use super::market::*;
use crate::GameState;
use std::collections::HashMap;

use bevy_ui_anchor::{AnchorPoint, AnchorUiConfig, AnchoredUiNodes};

// This plugin will contain the game. In this case, it's just be a screen that will
// display the current settings for 5 seconds before returning to the menu
#[derive(Resource, Deref)]
pub struct SelectedCity(pub String);

#[derive(Resource, Deref)]
struct BuildinTable(HashMap<String, Building>);

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::Game),
        (crate::kill_music, spawn_map_sprite, spawn_city_ui_nodes),
    )
    .insert_resource(SelectedCity("Unkown".to_string()))
    .insert_resource(BuildinTable(super::market::gen_building_tables()))
    .init_state::<StrategicState>()
    .add_systems(
        Update,
        (city_interaction_system).run_if(in_state(PopupHUD::Off)),
    )
    .add_systems(Update, update_ui_nodes.run_if(in_state(GameState::Game)));
}

/*#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum StrategicState<T: Send + Sync + Eq + std::fmt::Debug + std::hash::Hash + Clone + 'static> {
    #[default]
    Map,
    HUDOpen(HUDPosition, T),
}*/

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
}

use super::city_graph::Node as CityNode;
fn spawn_city_ui_nodes(
    mut commands: Commands,
    graph_nodes: Query<(Entity, &CityNode)>,
    mut sylt: Sylt,
) {
    for (ent, node) in graph_nodes {
        commands.entity(ent).insert(AnchoredUiNodes::spawn_one((
            AnchorUiConfig {
                anchorpoint: AnchorPoint::middle(),
                ..default()
            },
            Button,
            Transform::from_xyz(0., 0.0, 1.0),
            CityData {
                id: "Capital".to_string(),
                population: 2,
                buildings: vec![
                    "Automated Clothiers".to_string(),
                    "Mushroom Farm".to_string(),
                ],
            },
            CityIcon {
                id: "Capital".to_string(),
            },
            Node {
                width: px(32),
                height: px(32),
                ..default()
            },
            ImageNode::new(sylt.get_image("town_ui_icon")),
            BackgroundColor(Srgba::new(1.0, 0.1, 0.1, 1.0).into()),
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
pub struct CityData {
    pub id: String,
    pub population: u8,
    pub buildings: Vec<String>,
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
    mut popupp_state: ResMut<NextState<PopupHUD>>,
) {
    for (interaction, mut node_color, city) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                println!("Pressed the city {}", city.id);
                selected_city.0 = city.id.clone();
                menu_state.set(StrategicState::HUDOpen);
                popupp_state.set(PopupHUD::Off);
            }
            Interaction::Hovered => *node_color = Srgba::new(1.0, 0.1, 0.1, 1.0).into(),
            _ => {}
        }
    }
}
