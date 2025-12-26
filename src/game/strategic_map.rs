use bevy::prelude::*;

use crate::assets::Sylt;
use crate::GameState;

// This plugin will contain the game. In this case, it's just be a screen that will
// display the current settings for 5 seconds before returning to the menu
pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Game), strategic_setup)
        .add_systems(Update, (city_interaction_system));
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
    ));
}

#[derive(Component)]
struct CityIcon;

fn city_interaction_system(
    mut interaction_query: Query<(&Interaction, &CityIcon), Changed<Interaction>>,
) {
    for (interaction, city) in &mut interaction_query {
        //        *background_color = match (*interaction, _) {
        //            Interaction::Pressed => PRESSED_BUTTON.into(),
        //            Interaction::Hovered => HOVERED_PRESSED_BUTTON.into(),
        //        }
        println!("Pressed a button");
    }
}
