use crate::prelude::*;
use bevy::ui::InteractionDisabled;

#[derive(Component, Debug)]
#[relationship(relationship_target = Tooltips)]
pub struct TooltipOf {
    #[relationship]
    pub target: Entity,
}

#[derive(Component, Debug)]
#[require(Interaction)]
#[relationship_target(relationship = TooltipOf, linked_spawn)]
pub struct Tooltips(Vec<Entity>);

#[derive(Component, Debug)]
pub struct TooltipContainerTarget {
    id: Entity,
}

#[derive(Component, Debug)]
pub struct TooltipContainer;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (spawn_tooltip_container, reparent_tooltips).chain())
        .add_systems(Update, (reveal_tooltips, fix_positions_of_tooltips));
}

pub fn spawn_tooltip_container(
    mut commands: Commands,
    query: Query<Entity, (Added<Tooltips>, Without<TooltipContainerTarget>)>,
) {
    for ent in query {
        let id = commands
            .spawn((
                TooltipContainer,
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                Visibility::Hidden,
            ))
            .id();

        commands.entity(ent).insert(TooltipContainerTarget { id });
    }
}

pub fn fix_positions_of_tooltips(
    windows: Query<&Window>,
    query: Query<&mut UiTransform, With<TooltipContainer>>,
) {
    let Some(Vec2 { x, y }) = windows.iter().find_map(|w| w.cursor_position()) else {
        return;
    };

    for mut transform in query {
        transform.translation = Val2::px(x, y);
    }
}

pub fn reparent_tooltips(
    mut commands: Commands,
    query: Query<(Entity, &TooltipOf), Added<TooltipOf>>,
    containers: Query<&TooltipContainerTarget>,
) {
    for (ent, tooltip) in query {
        let Ok(container) = containers.get(tooltip.target) else {
            error!("Couldn't find parent of {tooltip:?}");
            continue;
        };
        commands.entity(ent).insert(ChildOf(container.id));
    }
}

pub fn reveal_tooltips(
    query: Query<
        (&Interaction, &TooltipContainerTarget),
        (Changed<Interaction>, Without<InteractionDisabled>),
    >,
    mut vis_query: Query<&mut Visibility>,
) {
    for (interaction, container) in query {
        let Ok(mut vis) = vis_query.get_mut(container.id) else {
            error!("Container had no visibility");
            continue;
        };
        match interaction {
            Interaction::Pressed | Interaction::Hovered => {
                *vis = Visibility::Visible;
            }
            Interaction::None => {
                *vis = Visibility::Hidden;
            }
        }
    }
}
