use crate::{DisplayQuality, GameState, Volume};
use bevy::prelude::*;
use bevy_simple_text_input::{TextInput, TextInputValue};

// List
use accesskit::{Node as Accessible, Role};
use bevy::{
    a11y::AccessibilityNode,
    ecs::spawn::SpawnIter,
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::hover::HoverMap,
};

use crate::assets::Sylt;

//demo recs
use bevy::{
    color::palettes,
    feathers::{
        controls::{button, checkbox, ButtonProps},
        dark_theme::create_dark_theme,
        theme::{ThemeBackgroundColor, ThemedText, UiTheme},
        tokens,
    },
    input_focus::tab_navigation::TabGroup,
    ui::{Checked, InteractionDisabled},
    ui_widgets::{observe, Activate, ValueChange},
};

// This plugin will contain the game. In this case, it's just be a screen that will
// display the current settings for 5 seconds before returning to the menu
pub fn game_plugin(app: &mut App) {
    app.insert_resource(UiTheme(create_dark_theme()))
        .add_systems(OnEnter(GameState::Game), game_setup)
        .insert_resource(SelectedSprite("No ID".to_string()))
        .add_systems(
            Update,
            (
                on_file_path_update,
                send_scroll_events,
                button_system,
                refresh_sprite_menu,
            ),
        )
        .add_observer(on_scroll_handler);
}

#[derive(Component)]
pub struct FileFinder;

#[derive(Resource, Deref)]
struct SelectedSprite(String);

pub fn on_file_path_update(
    file_text: Option<Single<&TextInputValue, (With<FileFinder>, Changed<TextInputValue>)>>,
) {
    if let Some(file_text) = file_text {
        dbg!(&file_text.into_inner().0);
    }
}

// Tag component used to tag entities added on the game screen
#[derive(Component)]
struct OnGameScreen;

fn game_setup(
    mut commands: Commands,
    display_quality: Res<DisplayQuality>,
    volume: Res<Volume>,
    sylt: Sylt,
) {
    commands.spawn(demo_root(sylt));
}

#[derive(Component, Clone, Copy)]
struct DemoDisabledButton;

#[derive(Component)]
struct PreviewSprite;

fn demo_root(sylt: Sylt) -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
        TabGroup::default(),
        ThemeBackgroundColor(tokens::WINDOW_BG),
        children![(
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::Start,
                padding: UiRect::all(px(8)),
                row_gap: px(8),
                width: percent(30),
                min_width: px(200),
                ..default()
            },
            children![
                vertically_scrolling_list(sylt),
                (
                    TextInput,
                    FileFinder,
                    Node {
                        padding: UiRect::all(Val::Px(5.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor::all(Color::BLACK),
                ),
                (PreviewSprite, ImageNode { ..default() }),
                (
                    button(
                        ButtonProps::default(),
                        (),
                        Spawn((Text::new("Update path"), ThemedText))
                    ),
                    observe(
                        |_activate: On<Activate>,
                        mut sylt: Sylt,
                        selected_sprite: Res<SelectedSprite>,
                        mut query_sprite: Single<&mut ImageNode, With<PreviewSprite>>,
                        text_box: Single<&TextInputValue, With<FileFinder>>| {
                            if text_box.0 == "" {
                                sylt.update_path(&selected_sprite.0, None);
                            }
                            else {
                                sylt.update_path(&selected_sprite.0, Some(text_box.0.clone()));
                            }
                            _ = sylt.save();
                            info!("Updating path!");

                            //Update the sprite
                            let sprite_handler = sylt.get_sprite(&selected_sprite.0);
                            **query_sprite = ImageNode {
                                image: sprite_handler.image,
                                ..default()
                            };
                        }
                    )
                ),
                (
                    checkbox(Checked, Spawn((Text::new("Checkbox"), ThemedText))),
                    observe(
                        |change: On<ValueChange<bool>>,
                         query: Query<Entity, With<DemoDisabledButton>>,
                         mut commands: Commands| {
                            info!("Checkbox clicked!");
                            let mut button = commands.entity(query.single().unwrap());
                            if change.value {
                                button.insert(InteractionDisabled);
                            } else {
                                button.remove::<InteractionDisabled>();
                            }
                            let mut checkbox = commands.entity(change.source);
                            if change.value {
                                checkbox.insert(Checked);
                            } else {
                                checkbox.remove::<Checked>();
                            }
                        }
                    )
                ),
            ]
        ),],
    )
}

const LINE_HEIGHT: f32 = 21.;

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
            delta *= LINE_HEIGHT;
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

const FONT_SIZE: f32 = 20.;

#[derive(Component)]
struct SpriteMenuButton(String);
fn vertically_scrolling_list(mut sylt: Sylt) -> impl Bundle {
    //    sylt.asset_map.load().expect("Could not load");
    let map: Vec<_> = sylt.asset_map.get_all_assets().cloned().collect();
    (
        Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            width: px(200),
            ..default()
        },
        children![
            (
                // Title
                Text::new("Available assets"),
                TextFont {
                    font_size: FONT_SIZE,
                    ..default()
                },
                Label,
            ),
            (
                // Scrolling list
                Node {
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::Stretch,
                    height: px(300),
                    overflow: Overflow::scroll_y(), // n.b.
                    ..default()
                },
                BackgroundColor(Color::srgb(0.10, 0.10, 0.10)),
                Children::spawn(SpawnIter(map.into_iter().map(move |i| {
                    let name = i.name;
                    (
                        Button,
                        SpriteMenuButton(name.clone()),
                        Node {
                            min_height: px(LINE_HEIGHT),
                            max_height: px(LINE_HEIGHT),
                            ..default()
                        },
                        children![(
                            Text(format!("{}", name)),
                            Label,
                            AccessibilityNode(Accessible::new(Role::ListItem)),
                        )],
                    )
                })))
            ),
        ],
    )
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
//Copied from menu
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color) in &mut interaction_query {
        *background_color = match *interaction {
            Interaction::Pressed => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

fn refresh_sprite_menu(
    mut sylt: Sylt,
    interaction_query: Query<
        (&Interaction, &SpriteMenuButton, Entity),
        (Changed<Interaction>, With<Button>),
    >,
    mut selected_sprite: ResMut<SelectedSprite>,
    mut query_sprite: Single<&mut ImageNode, With<PreviewSprite>>,
    mut text_box: Single<&mut TextInputValue, With<FileFinder>>,
) {
    for (interaction, sprite_id, entity) in &interaction_query {
        if *interaction == Interaction::Pressed {
            *selected_sprite = SelectedSprite(sprite_id.0.clone());
            let sprite_handler = sylt.get_sprite(&sprite_id.0);
            **query_sprite = ImageNode {
                image: sprite_handler.image,
                ..default()
            };

            if let Some(asset) = sylt.get_asset(&sprite_id.0) {
                text_box.0 = asset.path.clone().unwrap_or("".to_string());
            } else {
                panic!("Could not get the id'd asset");
            }
            println!("Pressed the {} button", sprite_id.0);
        }
    }
}
