use bevy::prelude::*;

use bevy::ui_widgets::{observe, ValueChange};

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
        .add_systems(OnEnter(StrategicState::Buildings), hud_setup)
        .add_systems(Update, (city_interaction_system, kill_button));
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum StrategicState {
    #[default]
    Map,
    Buildings,
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
        ImageNode {
            image: sylt.get_sprite("map").image,
            ..default()
        },
        children![(
            Button,
            CityData {
                id: "Capital".to_string(),
                population: 2,
                districts: vec![DistrictType::Farm],
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
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                HudButton::KillHud => {
                    menu_state.set(StrategicState::Map);
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
    EconomyTabAction,
    BuldingTabAction,
}

fn hud_setup(
    mut commands: Commands,
    mut sylt: Sylt,
    city_data: Query<&CityData>,
    selected_city: Res<SelectedCity>,
) {
    for city in city_data {
        if city.id == selected_city.0 {
            commands.spawn((
                DespawnOnExit(StrategicState::Buildings),
                Node {
                    top: Val::Vh(0.0),
                    width: Val::Vw(100.0),
                    height: Val::Vh(70.0),
                    ..default()
                },
                BackgroundColor(Srgba::new(0.2, 0.2, 0.2, 0.8).into()),
                Button,
                HudButton::KillHud, //Feels like a clunky way to quit the menu
            ));
            commands.spawn((
                DespawnOnExit(StrategicState::Buildings),
                Node {
                    top: Val::Vh(70.0),
                    width: Val::Vw(100.0),
                    height: Val::Vh(30.0),
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    ..default()
                },
                BackgroundColor(Srgba::new(0.2, 0.2, 0.2, 1.0).into()),
                children![
                    ((
                        Node {
                            width: percent(100.0),
                            height: percent(20.0),
                            ..default()
                        },
                        // Title
                        Text::new(city.id.clone()),
                        TextFont { ..default() }
                    )),
                    ((
                        Node {
                            width: percent(100.0),
                            height: percent(100.0),
                            align_items: AlignItems::Start,
                            justify_content: JustifyContent::Start,
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        Children::spawn((SpawnWith(
                            |parent: &mut bevy::ecs::relationship::RelatedSpawner<ChildOf>| {
                                for _ in 0..5 {
                                    parent.spawn((
                                        Node {
                                            width: percent(20),
                                            height: percent(100),
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BorderColor::all(Color::BLACK),
                                    ));
                                }
                            }
                        ),))
                    )),
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
                menu_state.set(StrategicState::Buildings);
            }
            Interaction::Hovered => *node_color = Srgba::new(1.0, 0.1, 0.1, 1.0).into(),
            _ => {}
        }
    }
}
