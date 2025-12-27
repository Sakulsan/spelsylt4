use super::strategic_hud::PopupHUD;
use crate::prelude::*;
use bevy::audio::Volume;

use super::market::*;
use crate::GameState;
use std::collections::HashMap;

// This plugin will contain the game. In this case, it's just be a screen that will
// display the current settings for 5 seconds before returning to the menu
#[derive(Resource, Deref)]
pub struct SelectedCity(pub String);

#[derive(Resource, Deref)]
struct BuildinTable(HashMap<String, Building>);

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::Game),
        (strategic_setup, crate::kill_music),
    )
    .insert_resource(SelectedCity("Unkown".to_string()))
    .insert_resource(BuildinTable(super::market::gen_building_tables()))
    .init_state::<StrategicState>()
    .add_systems(
        Update,
        (city_interaction_system).run_if(in_state(PopupHUD::Off)),
    );
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

fn strategic_setup(
    mut commands: Commands,
    //    display_quality: Res<DisplayQuality>,
    //    volume: Res<Volume>,
    asset_server: Res<AssetServer>,
    mut sylt: Sylt,
) {
    commands.spawn((
        AudioPlayer(asset_server.load::<AudioSource>("music/Moneymoneymoney.ogg")),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            ..default()
        },
    ));

    commands.spawn((
        Node {
            width: Val::Vw(100.0),
            height: Val::Vh(100.0),
            ..default()
        },
        Transform::from_xyz(0., 0.0, -1.0),
        Sprite {
            image: sylt.get_sprite("map").image,
            ..default()
        },
        children![(
            Button,
            Transform::from_xyz(0., 0.0, 1.0),
            CityData {
                id: "Capital".to_string(),
                population: 2,
                buildings: vec![
                    "Automated Clothiers".to_string(),
                    "Mushroom Farm".to_string()
                ],
            },
            CityIcon {
                id: "Capital".to_string()
            },
            Node {
                width: px(32),
                height: px(32),
                ..default()
            } //          Sprite {
              //                color: Srgba::new(1.0, 0.1, 0.1, 1.0).into(),
              //                custom_size: Some(Vec2::new(75., 75.)),
              //                ..default()
              //            }
        )],
    ));
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
