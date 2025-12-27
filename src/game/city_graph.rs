use std::f32::consts::PI;

use crate::prelude::*;

use petgraph::{algo::connected_components, graph::NodeIndex, Graph, Undirected};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, (setup, gen_edges, remove_random_edges).chain());
    app.add_systems(Update, gizmo_nodes);
}

#[derive(Component, Clone, Debug)]
pub struct Node(pub NodeIndex, pub Vec2, pub Color);

#[derive(Component, Clone, Debug)]
struct CityEdge(f32);

#[derive(Resource)]
pub struct CityGraph {
    graph: CGraph,
}

const CIRCLE_DIST: f32 = 100.0;
const ANGULAR_CONSTRAINT: f32 = PI / 9.0;
const JITTER: f32 = 15.0;
const CITY_COUNTS: [usize; 9] = [3, 4, 4, 5, 8, 12, 15, 20, 15];
const MIN_CITY_DIST: f32 = 25.0;
const SCALE: f32 = 1.0;

type CGraph = Graph<Entity, CityEdge, Undirected>;

fn setup(mut rng: ResMut<GlobalRng>, mut commands: Commands) {
    let vec2 = |x, y| Vec2::new(x, y);

    let mut random_circle_pos = |i: i32, min: f32, max: f32| {
        let ang = rng.random_range(min..=max);
        let d = (i + 1) as f32 * CIRCLE_DIST;
        let jx = rng.random_range(-JITTER..JITTER);
        let jy = rng.random_range(-JITTER..JITTER);
        Vec2::from_angle(ang) * d + vec2(jx, jy)
    };

    let mut g = Graph::new_undirected();

    let positions = [
        vec2(450., -1650.),
        vec2(450., -150.),
        vec2(-30., 1460.),
        vec2(-1360., 500.),
    ];

    let mut spawn_city = |pos: Vec2, color| {
        let mut ent = commands.spawn_empty();
        let idx = g.add_node(ent.id());
        ent.insert((
            Transform::from_translation(pos.extend(0.0)),
            Node(idx, pos, color),
        ));
    };

    const M: f32 = 2000.0 - 110.0;

    let rect = |a, b, c, d| Rect {
        min: vec2(a as f32, b as f32),
        max: vec2(c as f32, d as f32),
    };

    let map_rect = Rect {
        min: -vec2(M, M),
        max: vec2(M, M),
    };

    let lake_rects = [
        rect(-430, 620, 330, 850),
        rect(-700, 340, 360, 620),
        rect(-650, 220, 160, 340),
        rect(-650, 70, -30, 220),
        rect(-390, -40, -250, 70),
        rect(-250, 270, 130, 70),
        rect(1060, 1200, 1750, 1650),
        rect(1220, 800, 1640, 1200)
    ];

    let check_boxes = |p| {
        let contains = map_rect.contains(p);
        let mut not_underwater = true;
        for rect in lake_rects {
            not_underwater = not_underwater && rect.contains(p);
        }
        contains && not_underwater
    };

    let colors = [0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0];

    let make_color = |c: f32| Color::hsv(c * 2.0, 1.0, 1.0);

    let mut other_pos = Vec::new();

    for i in 0..4 {
        let capital_pos = positions[i];

        spawn_city(capital_pos * SCALE, make_color(colors[0]));

        let (min, max) = match i {
            3 => (-(260f32.to_radians()), 0.0),
            _ => (-PI, PI),
        };

        for (c, j) in CITY_COUNTS.into_iter().enumerate() {
            other_pos.clear();

            for _ in 0..j {
                let mut city_pos = capital_pos + random_circle_pos(c as i32, min, max);
                let mut attempts = 0;
                'x: loop {
                    attempts += 1;
                    if attempts > 10 {
                        info!("reached max attempts");
                        break;
                    }
                    for v in &other_pos {
                        if city_pos.distance(*v) < MIN_CITY_DIST {
                            city_pos = capital_pos + random_circle_pos(c as i32, min, max);
                            continue 'x;
                        }
                    }
                    break;
                }
                if check_boxes(city_pos) {
                    other_pos.push(city_pos);
                    spawn_city(city_pos, make_color(colors[(1 + c) % colors.len()]));
                }
            }
        }
    }

    commands.insert_resource(CityGraph { graph: g });
}

fn gizmo_nodes(mut gizmos: Gizmos, nodes: Query<&Node>, g: Res<CityGraph>) {
    for n in &nodes {
        gizmos.circle_2d(n.1, 5.0, n.2);
    }

    let g = &g.graph;
    for (n1) in &nodes {
        for neighbor in g.neighbors(n1.0) {
            let (n2) = nodes.get(g[neighbor]).expect("lol");

            gizmos.line_2d(n1.1, n2.1, Color::linear_rgb(1.0, 0.0, 0.0));
        }
    }
}

fn is_crossing((a, b): (Vec2, Vec2), nodes: &Query<&Node>, g: &CGraph) -> bool {
    for edge in g.edge_indices() {
        let Some((n1, n2)) = g.edge_endpoints(edge) else {
            panic!("what")
        };
        let Ok([n1, n2]) = nodes.get_many([g[n1], g[n2]]) else {
            panic!("what")
        };

        if intersect([n1.1, n2.1, a, b]) {
            return true;
        }
    }
    false
}

fn ccw(a: Vec2, b: Vec2, c: Vec2) -> bool {
    (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
}

fn intersect([a, b, c, d]: [Vec2; 4]) -> bool {
    fn shorten(a: Vec2, b: Vec2) -> [Vec2; 2] {
        let ab = (b - a).normalize();
        let ba = (a - b).normalize();

        let a_1 = a + ab * 0.0001;
        let b_1 = b + ba * 0.0001;

        [a_1, b_1]
    }

    let [a, b] = shorten(a, b);
    let [c, d] = shorten(c, d);

    ccw(a, c, d) != ccw(b, c, d) && ccw(a, b, c) != ccw(a, b, d)
}

fn gen_edges(nodes: Query<&Node>, mut g: ResMut<CityGraph>) {
    let g = &mut g.graph;

    let mut all_nodes: Vec<_> = nodes.iter().collect();
    let mut scratch = Vec::new();

    for n in &nodes {
        scratch.clear();
        for neighbor in g.neighbors(n.0) {
            break;
            let Ok(&Node(_, pos, _)) = nodes.get(g[neighbor]) else {
                error!("Couldn't find entity in query");
                continue;
            };
            scratch.push(pos - n.1);
        }

        all_nodes.sort_by_key(|&Node(_, t2, _)| (n.1.distance(*t2) * 1_000_000.0) as u64);

        'outer: for other in all_nodes.iter().skip(1).take(10) {
            if scratch.len() > 3 {
                break;
            }
            for x in &scratch {
                let y = other.1 - n.1;

                if y.angle_to(*x).abs() < ANGULAR_CONSTRAINT {
                    continue 'outer;
                }
            }

            if is_crossing((n.1, other.1), &nodes, &*g) {
                continue;
            } else {
                scratch.push(other.1 - n.1);
                g.add_edge(n.0, other.0, CityEdge(n.1.distance(other.1)));
            }
        }
    }
}

fn remove_random_edges(mut rng: ResMut<GlobalRng>, mut g: ResMut<CityGraph>) {
    const REMOVAL_FACTOR: f64 = 0.25;

    let g = &mut g.graph;
    let starting_components = connected_components(&*g);

    let mut g_test = CGraph::new_undirected();
    g_test.clone_from(g);

    let mut edges: Vec<_> = g.edge_indices().collect();
    edges.sort();
    let mut removed_edges = 0;
    let required = (REMOVAL_FACTOR * edges.len() as f64) as usize;

    while removed_edges < required {
        // try removing an edge
        let Some(&random_edge) = edges.choose(&mut rng) else {
            error!("This fucking blows dude");
            return;
        };
        let Some((n1, n2)) = g.edge_endpoints(random_edge) else {
            continue;
        };
        g_test.remove_edge(random_edge);
        // where performance goes to die
        if connected_components(&g_test) == starting_components {
            g.remove_edge(random_edge);
            edges.pop();
            removed_edges += 1;
        } else {
            info!("Failed at removing edge, retrying");
            g_test.clone_from(g);
        }
    }
}
