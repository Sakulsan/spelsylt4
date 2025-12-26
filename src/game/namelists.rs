use std::default;

use bevy::prelude::*;
use rand::seq::IndexedRandom;
use rand::Rng;

use crate::assets::Sylt;
use crate::GameState;
use crate::GlobalRng;

#[derive(Debug, Clone, Copy)]
enum CityType {
    Human,
    Elven,
    Goblin,
    Dwarven,
}

pub fn generate_city_names(
    amount: (usize, usize, usize, usize),
    mut rng: &mut ResMut<GlobalRng>,
) -> Vec<Vec<String>> {
    let mut names = vec![vec![], vec![], vec![], vec![]];
    let mut city_iter = |citytype: CityType, amount: usize, idx: usize| {
        for i in 0..amount {
            names[idx].push(generate_city_name(citytype, &mut rng));
        }
    };
    city_iter(CityType::Dwarven, amount.0, 0);
    city_iter(CityType::Elven, amount.1, 1);
    city_iter(CityType::Goblin, amount.2, 2);
    city_iter(CityType::Human, amount.3, 3);

    names
}

pub fn generate_city_name(city_type: CityType, mut rng: &mut ResMut<GlobalRng>) -> String {
    match city_type {
        CityType::Dwarven => get_dwarven_name(&mut rng),
        CityType::Elven => get_elven_name(&mut rng),
        CityType::Goblin => get_goblin_name(&mut rng),
        CityType::Human => get_human_name(&mut rng),
        default => panic!("You forgot to implement a namelist for {:?}", city_type),
    }
}

pub fn get_dwarven_name(mut rng: &mut ResMut<GlobalRng>) -> String {
    let dwarven_initial_particles = vec![
        "Ka", "Kal", "Bo", "Bol", "To", "Te", "Tal", "De", "Do", "Don", "Be", "Ge", "Get",
    ];
    let dwarven_latter_particle = vec!["rak", "raz", "bek", "bak", "rek", "dek", "dak", "bok"];
    let dwarven_connector = vec!["a", "e", "i", "o", "y", "yz", "az", "em", "et", "od", "an"];

    let name = format!(
        "{0}{1}-{2}-{3}{4}",
        dwarven_initial_particles
            .choose(&mut rng)
            .expect("error in initial particle dwarven name generator"),
        dwarven_latter_particle
            .choose(&mut rng)
            .expect("error in latter particle dwarven name generator"),
        dwarven_connector
            .choose(&mut rng)
            .expect("error in connector dwarven name generator"),
        dwarven_initial_particles
            .choose(&mut rng)
            .expect("error in initiale particle dwarven name generator"),
        dwarven_latter_particle
            .choose(&mut rng)
            .expect("error in latter particle dwarven name generator")
    );

    name
}

pub fn get_elven_name(mut rng: &mut ResMut<GlobalRng>) -> String {
    let elven_initial_particle = vec![
        "Dawn", "Sun", "Gem", "Ice", "Frost", "Heart", "Sky", "Heaven", "Winter", "Lore", "Fire",
        "World", "Moon", "Forge", "Flame", "Star", "Mage", "Silver", "Storm", "Amber", "Ash",
        "Brass", "Gold", "Diamond", "Emerald", "Earth", "Jewel"
    ];
    let elven_latter_particle = vec![
        "light", "spire", "tower", "haven", "reach", "star", "hearth", "home", "land", "peak", "fire",
        "fall", "rise", "spring", "reign", "garden", "sun", "edge", "crown"
    ];
    let elven_placement_particle = vec![
        "'s Edge", "'s Crown", "'s Peak", "'s Radiance", "'s Heart", "'s Eye", ""
    ];
    let elven_placement_particle =
        vec!["'s Edge", "'s Crown", "'s Peak", "'s Radiance", "'s Heart"];

    let mut name = "".to_string();

    match rng.random_range(0..10) {
        1 => {
            name = format!(
                "{0}{1} at {2}{3}",
                elven_initial_particle
                    .choose(&mut rng)
                    .expect("error in initial particle elven name generator"),
                elven_latter_particle
                    .choose(&mut rng)
                    .expect("error in latter particle elven name generator"),
                elven_initial_particle
                    .choose(&mut rng)
                    .expect("error in latter particle elven name generator"),
                elven_placement_particle
                    .choose(&mut rng)
                    .expect("error in placement particle elven name generator"),
            );
        }
        2 => {
            name = format!(
                "{0}{1}",
                elven_initial_particle
                    .choose(&mut rng)
                    .expect("error in initial particle elven name generator"),
                elven_placement_particle
                    .choose(&mut rng)
                    .expect("error in latter particle elven name generator")
            );
        }
        _ => {
            name = format!(
                "{0}{1}",
                elven_initial_particle
                    .choose(&mut rng)
                    .expect("error in initial particle elven name generator"),
                elven_latter_particle
                    .choose(&mut rng)
                    .expect("error in latter particle elven name generator")
            );
        }
    }

    name
}

pub fn get_goblin_name(mut rng: &mut ResMut<GlobalRng>) -> String {
    let goblin_initial_particle = vec![
        "Ke", "Te", "Tre", "Kre", "Ge", "Ze", "Zhe", "Phe", "Pe", "Se",
    ];
    let goblin_connector = vec!["t", "z", "r", "g", "p", "kh", "sh", "w", "b", "v"];

    let chosen_particle = goblin_initial_particle
        .choose(&mut rng)
        .expect("error in initial particle goblin name generator");
    let name = format!(
        "{0}{1}{2} {3}{4}",
        chosen_particle,
        goblin_connector
            .choose(&mut rng)
            .expect("error in connector goblin name generator"),
        chosen_particle
            .chars()
            .rev()
            .collect::<String>()
            .to_lowercase(),
        goblin_initial_particle
            .choose(&mut rng)
            .expect("error in initial particle goblin name generator"),
        goblin_connector
            .choose(&mut rng)
            .expect("error in connector goblin name generator")
    );

    name
}

pub fn get_human_name(mut rng: &mut ResMut<GlobalRng>) -> String {
    let human_initial_particle = vec![
        "Coven", "Lon", "Wake", "Shef", "Man", "Brad", "Notting", "Birming", "Stoke", "Trent",
        "Chelm", "York", "New", "Canter", "Don", "Bright", "Wolver", "Ply", "Der", "South",
        "North", "Prest", "Chi", "Inver", "Lin", "Wor", "Lan", "Dun",
    ];
    let human_latter_particle = vec![
        "try",
        "don",
        "field",
        "sea",
        "bury",
        "ham",
        "port",
        "ford",
        "mouth",
        "deen",
        "land",
        "fast",
        "pool",
        "burg",
        "diff",
        "bridge",
        "hampton",
        "by",
        "cast",
        "cester",
        "shire",
        "cestershire",
        "wich",
        "chester",
    ];
    let mut human_extending_particle = "".to_string();
    match rng.random_range(0..10) {
        7 => {
            human_extending_particle = "-on-Sea".to_string();
        }
        8 => {
            human_extending_particle = format!(
                "-on-{0}{1}",
                human_initial_particle
                    .choose(&mut rng)
                    .expect("error in extending particle human name generator"),
                human_latter_particle
                    .choose(&mut rng)
                    .expect("error in extending particle human name generator")
            );
        }
        9 => {
            human_extending_particle = format!(
                " upon {0}{1}",
                human_initial_particle
                    .choose(&mut rng)
                    .expect("error in extending particle human name generator"),
                human_latter_particle
                    .choose(&mut rng)
                    .expect("error in extending particle human name generator")
            );
        }
        _ => {}
    };

    let name = format!(
        "{0}{1}{2}",
        human_initial_particle
            .choose(&mut rng)
            .expect("error in initial particle human name generator"),
        human_latter_particle
            .choose(&mut rng)
            .expect("error in latter particle human name generator"),
        human_extending_particle
    );

    name
}
