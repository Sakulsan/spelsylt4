use crate::prelude::*;

use petgraph::{graph::NodeIndex, Graph, Undirected};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, (setup, gen_edges).chain());
    app.add_systems(Update, gizmo_nodes);
}

#[derive(Component)]
struct Node(NodeIndex);

#[derive(Component)]
struct CityEdge(f32);

#[derive(Resource)]
struct CityGraph {
    graph: Graph<(), CityEdge, Undirected>,
}

fn setup(mut commands: Commands) {
    let mut rng = rand::rng();
    let mut random_pos = || {
        Vec3::new(
            rng.random_range(0.0..1920.0),
            rng.random_range(0.0..1080.0),
            0.0,
        )
    };

    let mut g = Graph::new_undirected();

    for i in 0..100 {
        let idx = g.add_node(());
        commands.spawn((Node(idx), Transform::from_translation(random_pos())));
    }

    commands.insert_resource(CityGraph { graph: g });
}

fn gizmo_nodes(mut gizmos: Gizmos, nodes: Query<(&Transform, &Node)>, g: Res<CityGraph>) {
    for (t, n) in nodes {
        gizmos.circle_2d(t.translation.xy(), 5.0, Color::linear_rgb(1.0, 0.0, 0.0));
    }
}

fn gen_edges() {}
