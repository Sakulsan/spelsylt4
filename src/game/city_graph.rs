use std::collections::HashMap;
use std::f32::consts::PI;

use super::city_data::CityData;
use super::market::*;
use super::strategic_map::Faction;
use crate::game::strategic_map::BuildinTable;
use crate::{prelude::*, GameState};

use petgraph::algo::astar;
use petgraph::{algo::connected_components, graph::NodeIndex, Graph, Undirected};

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::Game),
        (setup, gen_edges, remove_random_edges)
            .chain()
            .in_set(NodeGenSet),
    );
    app.add_systems(Update, gizmo_nodes.run_if(resource_exists::<CityGraph>));
}

#[derive(Component, Clone, Debug)]
pub struct Node(pub NodeIndex, pub Vec2, pub Color);

#[derive(Component, Clone, Debug)]
struct CityEdge(f32);

#[derive(Component, Clone, Debug)]
pub struct CityTypeComponent(pub CityData);

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

fn gen_rand_circle(i: i32, min: f32, max: f32, rng: &mut ResMut<GlobalRng>) -> Vec2 {
    let ang = rng.random_range(min..=max);
    let d = (i + 1) as f32 * CIRCLE_DIST;
    let jx = rng.random_range(-JITTER..JITTER);
    let jy = rng.random_range(-JITTER..JITTER);
    Vec2::from_angle(ang) * d + vec2(jx, jy)
}

pub fn get_path(graph: &Res<CityGraph>, node1: NodeIndex, node2: NodeIndex) -> (f32, Vec<Entity>) {
    let (cost, mut path) = astar(
        &graph.graph,
        node1,
        |x| x == node2,
        |e| e.weight().0,
        |_| 0.0,
    )
    .expect(format!("Graph does not connect node {0:?} and {1:?}", node1, node2).as_str());

    let path = path
        .iter_mut()
        .map(|node_idx| graph.graph[*node_idx])
        .collect::<Vec<Entity>>();
    (cost, path)
}

fn spawn_city(
    pos: Vec2,
    color: Color,
    race: BuildingType,
    tier: u8,
    capital: bool,
    commands: &mut Commands,
    mut rng: &mut ResMut<GlobalRng>,
    g: &mut Graph<Entity, CityEdge, Undirected>,
) {
    let mut ent = commands.spawn_empty();
    let idx = g.add_node(ent.id());
    let mut data = CityData::new(race, tier, &mut rng);
    let mut empty_market: HashMap<Resources, isize> = HashMap::new();
    for res in Resources::all_resources() {
        empty_market.insert(res, 0);
    }
    if capital {
        data = match race {
            BuildingType::Dwarven => CityData {
                id: "Terez-e-Palaz".to_string(),
                race: BuildingType::Dwarven,
                population: 5,
                buildings_t1: vec![
                    ("Gem Cutters".to_string(), Faction::Neutral),
                    ("Gem Cutters".to_string(), Faction::Neutral),
                    ("Standard Mines".to_string(), Faction::Neutral),
                    ("Standard Mines".to_string(), Faction::Neutral),
                    ("Standard Mines".to_string(), Faction::Neutral),
                ],
                buildings_t2: vec![
                    ("Growth Vats".to_string(), Faction::Neutral),
                    ("Core Drill".to_string(), Faction::Neutral),
                    ("Preparatory Facilities".to_string(), Faction::Neutral),
                    ("Educated Workers".to_string(), Faction::Neutral),
                ],
                buildings_t3: vec![
                    ("Automation Components".to_string(), Faction::Neutral),
                    ("Megabreweries".to_string(), Faction::Neutral),
                    ("Megabreweries".to_string(), Faction::Neutral),
                ],
                buildings_t4: vec![
                    ("Industrial Smeltery".to_string(), Faction::Neutral),
                    ("Dwarven Assembly Lines".to_string(), Faction::Neutral),
                ],
                buildings_t5: vec![("The Great Red Forges".to_string(), Faction::Neutral)],
                market: empty_market,
                tier_up_counter: 0,
            },
            BuildingType::Elven => CityData {
                id: "Jewel of All Creation".to_string(),
                race: BuildingType::Elven,
                population: 5,
                buildings_t1: vec![
                    ("Earth Spirit Aid".to_string(), Faction::Neutral),
                    ("Ironwood Forestry".to_string(), Faction::Neutral),
                    ("Forest Foraging".to_string(), Faction::Neutral),
                    ("Standard Mines".to_string(), Faction::Neutral),
                    ("Standard Mines".to_string(), Faction::Neutral),
                ],
                buildings_t2: vec![
                    ("Amber Plantations".to_string(), Faction::Neutral),
                    ("Amber Plantations".to_string(), Faction::Neutral),
                    ("Gardens of Wonder".to_string(), Faction::Neutral),
                    ("Gardens of Wonder".to_string(), Faction::Neutral),
                ],
                buildings_t3: vec![
                    ("Integrated Farms".to_string(), Faction::Neutral),
                    ("Elemental Springs".to_string(), Faction::Neutral),
                    ("Basic Industry".to_string(), Faction::Neutral),
                ],
                buildings_t4: vec![
                    ("Gaian Meadows".to_string(), Faction::Neutral),
                    ("Self-spinning Weavers".to_string(), Faction::Neutral),
                ],
                buildings_t5: vec![(
                    "Tower of the Luminous Science".to_string(),
                    Faction::Neutral,
                )],
                market: empty_market,
                tier_up_counter: 0,
            },
            BuildingType::Goblin => CityData {
                id: "Tevet Pekhep Dered".to_string(),
                race: BuildingType::Goblin,
                population: 5,
                buildings_t1: vec![
                    ("Deep Mines".to_string(), Faction::Neutral),
                    ("Deep Mines".to_string(), Faction::Neutral),
                    ("Animated Objects".to_string(), Faction::Neutral),
                    ("Alchemical Enhancements".to_string(), Faction::Neutral),
                    ("Alchemical Enhancements".to_string(), Faction::Neutral),
                ],
                buildings_t2: vec![
                    ("Glaziery".to_string(), Faction::Neutral),
                    ("Glaziery".to_string(), Faction::Neutral),
                    ("Charcoal Kilns".to_string(), Faction::Neutral),
                    ("Hill Quarries".to_string(), Faction::Neutral),
                ],
                buildings_t3: vec![
                    ("Artisan District".to_string(), Faction::Neutral),
                    ("Trains".to_string(), Faction::Neutral),
                    ("Apothecary's Workshop".to_string(), Faction::Neutral),
                ],
                buildings_t4: vec![
                    ("Siege-Factories".to_string(), Faction::Neutral),
                    ("Golem Automatons".to_string(), Faction::Neutral),
                ],
                buildings_t5: vec![(
                    "Cauldronworks of the Four Clans".to_string(),
                    Faction::Neutral,
                )],
                market: empty_market,
                tier_up_counter: 0,
            },
            BuildingType::Human => CityData {
                id: "Great Lancastershire".to_string(),
                race: BuildingType::Human,
                population: 5,
                buildings_t1: vec![
                    ("Large Industrial District".to_string(), Faction::Neutral),
                    ("Large Industrial District".to_string(), Faction::Neutral),
                    ("Fishing Port".to_string(), Faction::Neutral),
                    ("Fishing Port".to_string(), Faction::Neutral),
                    ("Tree Plantations".to_string(), Faction::Neutral),
                ],
                buildings_t2: vec![
                    ("Water Cleaning Facilities".to_string(), Faction::Neutral),
                    ("Water Cleaning Facilities".to_string(), Faction::Neutral),
                    ("Hired Workforces".to_string(), Faction::Neutral),
                    ("Small-scale Forges".to_string(), Faction::Neutral),
                ],
                buildings_t3: vec![
                    ("Manufactories".to_string(), Faction::Neutral),
                    ("Mercenary Guild".to_string(), Faction::Neutral),
                    ("Apothecary's Workshop".to_string(), Faction::Neutral),
                ],
                buildings_t4: vec![
                    ("Teleportation Circle Network".to_string(), Faction::Neutral),
                    ("Strip Mines".to_string(), Faction::Neutral),
                ],
                buildings_t5: vec![("Sunstrider Headquarters".to_string(), Faction::Neutral)],
                market: empty_market,
                tier_up_counter: 0,
            },
            _ => {
                panic!(
                    "Attempted to spawn city for capital of race type {:?}",
                    race
                )
            }
        };
    }
    ent.insert((
        Transform::from_translation(pos.extend(0.0)),
        Node(idx, pos, color),
        Button,
        data,
    ));
}

fn setup(mut rng: ResMut<GlobalRng>, mut commands: Commands) {
    let vec2 = |x, y| Vec2::new(x, y);

    let mut g = Graph::new_undirected();

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
        rect(-430, 620, 330, 950),
        rect(-700, 340, 360, 620),
        rect(-650, 220, 160, 340),
        rect(-650, 70, -30, 220),
        rect(-390, -40, -250, 70),
        rect(-250, -270, 130, 70),
        rect(1060, 1200, 1750, 1650),
        rect(1220, 800, 1640, 1200),
    ];

    let check_boxes = |p| {
        let contains = map_rect.contains(p);
        let mut not_underwater = true;
        for rect in lake_rects {
            not_underwater = not_underwater && !rect.contains(p);
        }
        contains && not_underwater
    };

    let colors = [0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0];

    let make_color = |c: f32| Color::hsv(c * 2.0, 1.0, 1.0);

    let mut other_pos = Vec::new();

    for (race, capital_pos) in [
        (BuildingType::Goblin, vec2(635., -1460.)),
        (BuildingType::Human, vec2(20., -150.)),
        (BuildingType::Elven, vec2(-30., 1460.)),
        (BuildingType::Dwarven, vec2(-1495., 1100.)),
    ] {
        spawn_city(
            capital_pos * SCALE,
            make_color(colors[0]),
            race,
            5,
            true,
            &mut commands,
            &mut rng,
            &mut g,
        );

        let (min, max) = match race {
            BuildingType::Dwarven => (-(260f32.to_radians()), 0.0),
            _ => (-PI, PI),
        };

        for (c, j) in CITY_COUNTS.into_iter().enumerate() {
            other_pos.clear();

            for _ in 0..j {
                let mut city_pos = capital_pos + gen_rand_circle(c as i32, min, max, &mut rng);
                let mut attempts = 0;
                'x: loop {
                    attempts += 1;
                    if attempts > 10 {
                        info!("reached max attempts");
                        break;
                    }
                    for v in &other_pos {
                        if city_pos.distance(*v) < MIN_CITY_DIST {
                            city_pos = capital_pos + gen_rand_circle(c as i32, min, max, &mut rng);
                            continue 'x;
                        }
                    }
                    break;
                }
                if check_boxes(city_pos) {
                    let tier = match c {
                        0 => 4,
                        1..3 => 3,
                        3..5 => 2,
                        _ => 1,
                    };
                    other_pos.push(city_pos);
                    println!("Missing a city spawn");
                    spawn_city(
                        city_pos,
                        make_color(colors[(1 + c) % colors.len()]),
                        race,
                        tier,
                        false,
                        &mut commands,
                        &mut rng,
                        &mut g,
                    );
                }
            }
        }
    }

    commands.insert_resource(CityGraph { graph: g });
}

fn gizmo_nodes(mut gizmos: Gizmos, nodes: Query<&Node>, g: Res<CityGraph>) {
    /*    for n in &nodes {
        gizmos.circle_2d(n.1, 5.0, n.2);
    }*/

    let g = &g.graph;
    for n1 in &nodes {
        for neighbor in g.neighbors(n1.0) {
            let n2 = nodes.get(g[neighbor]).expect("lol");

            gizmos.line_2d(n1.1, n2.1, Color::linear_rgb(0.55, 0.27, 0.075));
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
