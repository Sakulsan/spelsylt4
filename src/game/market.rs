use std::collections::HashMap;

use crate::prelude::*;
use bevy::pbr::generate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Resources {
    Artifacts,
    Coal,
    CommonAlloys,
    CommonOre,
    ComplexLabour,
    Drugs,
    ExoticAlloys,
    Food,
    Glass,
    Lumber,
    Luxuries,
    Machinery,
    ManufacturedGoods,
    Medicines,
    Military,
    Plants,
    RareOre,
    Reagents,
    RefinedValuables,
    SimpleLabour,
    Slaves,
    Spellwork,
    Stone,
    Textiles,
    Transportation,
    Vitae,
    Water,
}

pub const BASIC_RESOURCES: [Resources; 9] = [
    Resources::Food,
    Resources::Plants,
    Resources::CommonOre,
    Resources::RareOre,
    Resources::Lumber,
    Resources::Stone,
    Resources::Water,
    Resources::Glass,
    Resources::Coal,
];

pub const ADVANCED_RESOURCES: [Resources; 7] = [
    Resources::RefinedValuables,
    Resources::CommonAlloys,
    Resources::Textiles,
    Resources::ManufacturedGoods,
    Resources::Medicines,
    Resources::Reagents,
    Resources::Machinery,
];

pub const SERVICE_RESOURCES: [Resources; 5] = [
    Resources::SimpleLabour,
    Resources::Military,
    Resources::Transportation,
    Resources::Luxuries,
    Resources::ComplexLabour,
];

pub const EXOTIC_RESOURCES: [Resources; 3] = [
    Resources::ExoticAlloys,
    Resources::Spellwork,
    Resources::Artifacts,
];

pub const ILLEGAL_RESOURCES: [Resources; 3] =
    [Resources::Drugs, Resources::Slaves, Resources::Vitae];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Serialize, Deserialize)]
pub enum BuildingType {
    Human,
    Elven,
    Goblin,
    Dwarven,
    #[default]
    Generic,
    Illegal,
    Unique,
}

impl Resources {
    pub fn all_resources() -> [Self; 27] {
        [
            Self::Artifacts,
            Self::Coal,
            Self::CommonAlloys,
            Self::CommonOre,
            Self::ComplexLabour,
            Self::Drugs,
            Self::ExoticAlloys,
            Self::Food,
            Self::Glass,
            Self::Lumber,
            Self::Luxuries,
            Self::Machinery,
            Self::ManufacturedGoods,
            Self::Medicines,
            Self::Military,
            Self::Plants,
            Self::RareOre,
            Self::Reagents,
            Self::RefinedValuables,
            Self::SimpleLabour,
            Self::Slaves,
            Self::Spellwork,
            Self::Stone,
            Self::Textiles,
            Self::Transportation,
            Self::Vitae,
            Self::Water,
        ]
    }

    pub fn get_base_value(&self) -> isize {
        match &self {
            Self::Food => 1,
            Self::Plants => 1,
            Self::CommonOre => 1,
            Self::RareOre => 1,
            Self::Lumber => 1,
            Self::Stone => 1,
            Self::Water => 1,
            Self::Glass => 1,
            Self::Coal => 1,
            Self::RefinedValuables => 3,
            Self::CommonAlloys => 3,
            Self::Textiles => 3,
            Self::ManufacturedGoods => 3,
            Self::Medicines => 3,
            Self::Reagents => 3,
            Self::Machinery => 3,
            Self::Drugs => 3,
            Self::Slaves => 3,
            Self::Vitae => 5,
            Self::SimpleLabour => 1,
            Self::Military => 1,
            Self::Transportation => 1,
            Self::Luxuries => 1,
            Self::ComplexLabour => 1,
            Self::ExoticAlloys => 5,
            Self::Spellwork => 5,
            Self::Artifacts => 5,
        }
    }

    pub fn get_name(&self) -> &str {
        match &self {
            Self::Food => "Food",
            Self::Plants => "Plants",
            Self::CommonOre => "Common Ore",
            Self::RareOre => "Rare Ore",
            Self::Lumber => "Lumber",
            Self::Stone => "Stone",
            Self::Water => "Water",
            Self::Glass => "Glass",
            Self::Coal => "Coal",
            Self::RefinedValuables => "Refined Valuables",
            Self::CommonAlloys => "Common Alloys",
            Self::Textiles => "Textiles",
            Self::ManufacturedGoods => "Manufactured Goods",
            Self::Medicines => "Medicines",
            Self::Reagents => "Reagents",
            Self::Machinery => "Machinery",
            Self::Drugs => "Drugs",
            Self::Slaves => "Slaves",
            Self::Vitae => "Vitae",
            Self::SimpleLabour => "Simple Labour",
            Self::Military => "Military",
            Self::Transportation => "Transportation",
            Self::Luxuries => "Luxuries",
            Self::ComplexLabour => "Complex Labour",
            Self::ExoticAlloys => "Exotic Alloys",
            Self::Spellwork => "Spellwork",
            Self::Artifacts => "Artifacts",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Building {
    pub input: HashMap<Resources, isize>,
    pub output: HashMap<Resources, isize>,
    pub tier: usize,
    pub image_sylt_id: Option<String>,
    pub build_type: BuildingType,
}

pub fn gen_building_tables() -> HashMap<String, Building> {
    let mut all_buildings = HashMap::new();
    let mut current_type = BuildingType::Generic;

    macro_rules! generate_building {
        ($name:literal, $($inputname:ident x $inputamount:literal),*; $($outputname:ident x $outputamount:literal),*; $tier:literal) => {
            all_buildings.insert(
                $name.to_string(),
                Building {
                    input: quick_hash(vec![$((Resources::$inputname, $inputamount)),*]),
                    output: quick_hash(vec![$((Resources::$outputname, $outputamount)),*]),
                    image_sylt_id: Some($name.to_lowercase()),
                    tier: $tier,
                    build_type: current_type
                },
            );
        };
    }

    //Generic buildings
    generate_building!("Standard Farms", Water x 15, SimpleLabour x 10; Food x 15, Plants x 15; 1);
    generate_building!("Standard Mines", ManufacturedGoods x 10, SimpleLabour x 10; CommonOre x 15, RareOre x 15, Coal x 15; 1);
    generate_building!("Quarry", ManufacturedGoods x 5, SimpleLabour x 5; Stone x 25; 1);
    generate_building!("Forestry Site", ManufacturedGoods x 5, SimpleLabour x 5; Lumber x 25; 1);
    generate_building!("Workers", Food x 10; SimpleLabour x 15; 1);
    generate_building!("Educated Workers", Food x 15, ManufacturedGoods x 5; ComplexLabour x 40; 2);
    generate_building!("Well", Stone x 5, SimpleLabour x 15; Water x 30; 2);
    generate_building!("Glassworks", Stone x 20, CommonOre x 10, SimpleLabour x 10; Glass x 50; 2);
    generate_building!("Wagons", Lumber x 15, CommonAlloys x 5, ComplexLabour x 10; Transportation x 50; 2);
    generate_building!("Cloth Mills", Plants x 15, ComplexLabour x 5; Textiles x 10; 2);
    generate_building!("Apothecary's Workshop", Plants x 25, ComplexLabour x 10; Medicines x 20; 3);
    generate_building!("Basic Industry", Stone x 20, Lumber x 20, CommonAlloys x 5, ComplexLabour x 10; ManufacturedGoods x 20, Luxuries x 30; 3);
    generate_building!("Hired Mercenaries", ManufacturedGoods x 5, Medicines x 5, RefinedValuables x 5, SimpleLabour x 5; Military x 75; 3);
    generate_building!("Modern Artificers", ExoticAlloys x 5, Lumber x 5, ComplexLabour x 10; ManufacturedGoods x 15, Machinery x 15; 4);
    generate_building!("Modern Soldiers", Artifacts x 5, Medicines x 5, ComplexLabour x 10; Military x 125; 4);
    generate_building!("Modern Comforts", Spellwork x 5, Artifacts x 10, ExoticAlloys x 10; Luxuries x 100, Medicines x 20, Transportation x 50; 5);

    //Dwarven buildings
    current_type = BuildingType::Dwarven;
    generate_building!("Mushroom Farm", Water x 10, SimpleLabour x 5; Food x 25; 1);
    generate_building!("Automated Clothier", Machinery x 5, Plants x 20; Textiles x 15; 1);
    generate_building!("Gem Cutters", Water x 15, RareOre x 10, ComplexLabour x 10; RefinedValuables x 15; 1);
    generate_building!("Preparatory Facilities", RefinedValuables x 5, Machinery x 5, Plants x 5, ComplexLabour x 15; Reagents x 25; 2);
    generate_building!("Core Drill", Machinery x 10, CommonAlloys x 10, Water x 10, Coal x 5; Stone x 100; 2);
    generate_building!("Growth Vats", Machinery x 5, Plants x 20, SimpleLabour x 15; Medicines x 25; 2);
    generate_building!("Automation Components", Machinery x 10, ManufacturedGoods x 10, Coal x 10; SimpleLabour x 120; 3);
    generate_building!("Megabreweries", Reagents x 5, Machinery x 5, Food x 55, SimpleLabour x 15; Luxuries x 150; 3);
    generate_building!("Industrial Smeltery", CommonOre x 100, Coal x 40, Machinery x 20, SimpleLabour x 15; CommonAlloys x 100; 4);
    generate_building!("Dwarven Assembly Lines", CommonAlloys x 40, Water x 10, Coal x 10, Stone x 10, SimpleLabour x 5; Machinery x 80; 4);
    generate_building!("Adamantium Smeltery", CommonAlloys x 20, RefinedValuables x 40, Reagents x 20, Machinery x 20, RareOre x 30, CommonOre x 20, Coal x 10, ComplexLabour x 10; ExoticAlloys x 100; 5);

    //Elven buildings
    current_type = BuildingType::Elven;
    generate_building!("Earth Spirit Aid", Spellwork x 1; SimpleLabour x 15; 1);
    generate_building!("Ironwood Forestry", Spellwork x 1, SimpleLabour x 15; CommonAlloys x 10; 1);
    generate_building!("Forest Foraging", Spellwork x 1, SimpleLabour x 15; Lumber x 30; 1);
    generate_building!("Domesticated Orchards", Spellwork x 5, Water x 10, SimpleLabour x 20, ComplexLabour x 70; ManufacturedGoods x 50; 2);
    generate_building!("Amber Plantations", Spellwork x 5, Water x 5, SimpleLabour x 5; RefinedValuables x 20; 2);
    generate_building!("Gardens of Wonder", Spellwork x 5, Water x 5, SimpleLabour x 5; Reagents x 20; 2);
    generate_building!("Elemental Springs", Spellwork x 15, Reagents x 10, SimpleLabour x 10; Water x 160; 3);
    generate_building!("Integrated Farms", Spellwork x 10, Reagents x 10, Water x 10, SimpleLabour x 10; Food x 75, Plants x 75; 3);
    generate_building!("Gaian Meadows", Spellwork x 10, Reagents x 15, Plants x 50, SimpleLabour x 10; Medicines x 80; 4);
    generate_building!("Self-spinning Weavers", Spellwork x 10, Plants x 100, Glass x 5; Textiles x 80; 4);
    generate_building!("Archmage's Tower", ComplexLabour x 50, Reagents x 50, RefinedValuables x 50, SimpleLabour x 20; Spellwork x 100; 5);

    //Goblin buildings
    current_type = BuildingType::Goblin;
    generate_building!("Deep Mines", Artifacts x 5, ComplexLabour x 5; CommonOre x 20, RareOre x 20; 1);
    generate_building!("Animated Objects", Artifacts x 5, Spellwork x 5; ComplexLabour x 60; 1);
    generate_building!("Alchemical Enhancements", Artifacts x 5, SimpleLabour x 15; SimpleLabour x 50; 1);
    generate_building!("Glaziery", Stone x 10, CommonOre x 5, SimpleLabour x 10; Glass x 30, Luxuries x 20; 2);
    generate_building!("Charcoal Kilns", Lumber x 30, ComplexLabour x 5; Coal x 60; 2);
    generate_building!("Hill Quarries", ManufacturedGoods x 5, SimpleLabour x 5; Stone x 60; 2);
    generate_building!("Artisan District", RareOre x 20, Glass x 10, ComplexLabour x 10; RefinedValuables x 90; 3);
    generate_building!("Trains", Lumber x 20, Coal x 20, Machinery x 15, ComplexLabour x 15; Transportation x 150; 3);
    generate_building!("Siege-Factories", Artifacts x 10, ComplexLabour x 15; Military x 150; 4);
    generate_building!("Golem Automatons", Artifacts x 10, ExoticAlloys x 10, Reagents x 10, Stone x 5; ComplexLabour x 100, SimpleLabour x 100, Military x 100; 4);
    generate_building!("Alchemic Factories", ExoticAlloys x 10, RefinedValuables x 10, Reagents x 10, Glass x 20, RareOre x 10, ComplexLabour x 30; Artifacts x 60; 5);

    //Human buildings
    current_type = BuildingType::Human;
    generate_building!("Large Industrial District", CommonOre x 45, RareOre x 10, ComplexLabour x 5, SimpleLabour x 15; CommonAlloys x 5, ManufacturedGoods x 5, Textiles x 5, Machinery x 5, Luxuries x 15; 1);
    generate_building!("Fishing Port", SimpleLabour x 15, Textiles x 5; Food x 40; 1);
    generate_building!("Tree Plantations", Water x 15, SimpleLabour x 10; Plants x 20, Lumber x 20; 1);
    generate_building!("Water Cleaning Facilities", Spellwork x 1, Artifacts x 1, SimpleLabour x 10; Water x 50; 2);
    generate_building!("Hired Workforces", RefinedValuables x 5; SimpleLabour x 40; 2);
    generate_building!("Small-scale Forges", CommonOre x 50, ComplexLabour x 15; CommonAlloys x 20, ManufacturedGoods x 10; 2);
    generate_building!("Manufactories", Lumber x 20, Stone x 10, Glass x 10, CommonAlloys x 10, ComplexLabour x 20, SimpleLabour x 40; ManufacturedGoods x 60; 3);
    generate_building!("Mercenary Guild", CommonAlloys x 30, Medicines x 20; Military x 200; 3);
    generate_building!("Teleportation Circle Network", Spellwork x 25, ComplexLabour x 40; Transportation x 250; 4);
    generate_building!("Strip Mines", ManufacturedGoods x 60, SimpleLabour x 35; CommonOre x 125, RareOre x 125, Coal x 50; 4);
    generate_building!("Relic Hunters", Military x 320, Medicines x 50; ExoticAlloys x 30, Spellwork x 30, Artifacts x 30, RefinedValuables x 50; 5);

    //Illegal buildings
    current_type = BuildingType::Illegal;
    generate_building!("Opium Plantation", Water x 10, SimpleLabour x 10; Drugs x 10; 1);
    generate_building!("Hired Banditry", Military x 20, Medicines x 5; Slaves x 20; 2);
    generate_building!("Joy Distillery", Reagents x 5, Medicines x 5, ComplexLabour x 10; Drugs x 30; 3);
    generate_building!("Lawless Enforcement", Military x 100, Luxuries x 25, RefinedValuables x 10; Slaves x 80; 4);
    generate_building!("Life Extractors", Slaves x 20, Spellwork x 10, ComplexLabour x 10; Vitae x 50; 5);

    //Capital buildings
    current_type = BuildingType::Unique;
    generate_building!("Tower of the Luminous Science", ComplexLabour x 70, Reagents x 75, RefinedValuables x 75; Spellwork x 150; 5);
    generate_building!("The Great Red Forges", CommonAlloys x 25, RefinedValuables x 45, Reagents x 25, Machinery x 25, RareOre x 35, CommonOre x 25, Coal x 15, ComplexLabour x 35; ExoticAlloys x 150; 5);
    generate_building!("Cauldronworks of the Four Clans",  ExoticAlloys x 15, RefinedValuables x 15, Reagents x 15, Glass x 40, RareOre x 25, ComplexLabour x 50; Artifacts x 120; 5);
    generate_building!("Sunstrider Headquarters", RefinedValuables x 100, Medicines x 50; ExoticAlloys x 40, Spellwork x 40, Artifacts x 40, Military x 70; 5);

    all_buildings
}

pub fn gen_random_building(
    tier: u8,
    mut rng: &mut ResMut<GlobalRng>,
    mut race: BuildingType,
) -> String {
    if race == BuildingType::Illegal || race == BuildingType::Generic {
        panic!("generated a random building of race {:?}", race)
    }

    let random_choice: u32 = rng.0.random();
    if rng.0.random_range(0..3) == 2 {
        race = BuildingType::Generic;
    }

    let result = match race {
        BuildingType::Dwarven => match tier {
            1 => match random_choice % 3 {
                0 => "Mushroom Farm",
                1 => "Automated Clothier",
                2 => "Gem Cutters",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            2 => match random_choice % 3 {
                0 => "Preparatory Facilities",
                1 => "Core Drill",
                2 => "Growth Vats",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            3 => match random_choice % 2 {
                0 => "Automation Components",
                1 => "Megabreweries",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            4 => match random_choice % 2 {
                0 => "Industrial Smeltery",
                1 => "Dwarven Assembly Lines",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            5 => "Adamantium Smeltery",
            _ => panic!(
                "gen_random_building tried to generate a building of tier {:?}",
                tier
            ),
        },
        BuildingType::Elven => match tier {
            1 => match random_choice % 3 {
                0 => "Earth Spirit Aid",
                1 => "Ironwood Forestry",
                2 => "Forest Foraging",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            2 => match random_choice % 3 {
                0 => "Domesticated Orchards",
                1 => "Amber Plantations",
                2 => "Gardens of Wonder",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            3 => match random_choice % 2 {
                0 => "Elemental Springs",
                1 => "Integrated Farms",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            4 => match random_choice % 2 {
                0 => "Gaian Meadows",
                1 => "Self-spinning Weavers",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            5 => "Archmage's Tower",
            _ => panic!(
                "gen_random_building tried to generate a building of tier {:?}",
                tier
            ),
        },
        BuildingType::Goblin => match tier {
            1 => match random_choice % 3 {
                0 => "Deep Mines",
                1 => "Animated Objects",
                2 => "Alchemical Enhancements",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            2 => match random_choice % 3 {
                0 => "Glaziery",
                1 => "Charcoal Kilns",
                2 => "Hill Quarries",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            3 => match random_choice % 2 {
                0 => "Hill Quarries",
                1 => "Artisan District",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            4 => match random_choice % 2 {
                0 => "Siege-Factories",
                1 => "Golem Automatons",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            5 => "Alchemic Factories",
            _ => panic!(
                "gen_random_building tried to generate a building of tier {:?}",
                tier
            ),
        },
        BuildingType::Human => match tier {
            1 => match random_choice % 3 {
                0 => "Large Industrial District",
                1 => "Fishing Port",
                2 => "Tree Plantations",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            2 => match random_choice % 3 {
                0 => "Water Cleaning Facilities",
                1 => "Hired Workforces",
                2 => "Small-scale Forges",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            3 => match random_choice % 2 {
                0 => "Manufactories",
                1 => "Mercenary Guild",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            4 => match random_choice % 2 {
                0 => "Teleportation Circle Network",
                1 => "Strip Mines",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            5 => "Relic Hunters",
            _ => panic!(
                "gen_random_building tried to generate a building of tier {:?}",
                tier
            ),
        },
        BuildingType::Generic => match tier {
            1 => match random_choice % 5 {
                0 => "Standard Farms",
                1 => "Standard Mines",
                2 => "Quarry",
                3 => "Forestry Site",
                4 => "Workers",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            2 => match random_choice % 5 {
                0 => "Educated Workers",
                1 => "Well",
                2 => "Glassworks",
                3 => "Wagons",
                4 => "Cloth Mills",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            3 => match random_choice % 3 {
                0 => "Apothecary's Workshop",
                1 => "Basic Industry",
                2 => "Hired Mercenaries",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            4 => match random_choice % 2 {
                0 => "Modern Artificers",
                1 => "Modern Soldiers",
                _ => panic!("Modulo stopped working in gen_random_building"),
            },
            5 => "Modern Comforts",
            _ => panic!(
                "gen_random_building tried to generate a building of tier {:?}",
                tier
            ),
        },
        _ => {
            panic!("fucky wucky code in gen_random_building.")
        }
    };

    result.to_string()
}

//I dont like to borrow string but its ass
pub fn get_construction_list(race: BuildingType, tier: usize) -> Vec<&'static str> {
    if race == BuildingType::Illegal || race == BuildingType::Generic {
        panic!("generated a random building of race {:?}", race)
    }

    let mut race_result = match race {
        BuildingType::Dwarven => match tier {
            1 => vec!["Mushroom Farm", "Automated Clothier", "Gem Cutters"],
            2 => vec!["Preparatory Facilities", "Core Drill", "Growth Vats"],
            3 => vec!["Automation Components", "Megabreweries"],
            4 => vec!["Industrial Smeltery", "Dwarven Assembly Lines"],
            5 => vec!["Adamantium Smeltery"],
            _ => panic!(
                "gen_random_building tried to generate a building of tier {:?}",
                tier
            ),
        },
        BuildingType::Elven => match tier {
            1 => vec!["Earth Spirit Aid", "Ironwood Forestry", "Forest Foraging"],
            2 => vec![
                "Domesticated Orchards",
                "Amber Plantations",
                "Gardens of Wonder",
            ],
            3 => vec!["Elemental Springs", "Integrated Farms"],
            4 => vec!["Gaian Meadows", "Self-spinning Weavers"],
            5 => vec!["Archmage's Tower"],
            _ => panic!(
                "gen_random_building tried to generate a building of tier {:?}",
                tier
            ),
        },
        BuildingType::Goblin => match tier {
            1 => vec!["Deep Mines", "Animated Objects", "Alchemical Enhancements"],
            2 => vec!["Glaziery", "Charcoal Kilns", "Hill Quarries"],
            3 => vec!["Hill Quarries", "Artisan District"],
            4 => vec!["Siege-Factories", "Golem Automatons"],
            5 => vec!["Alchemic Factories"],
            _ => panic!(
                "gen_random_building tried to generate a building of tier {:?}",
                tier
            ),
        },
        BuildingType::Human => match tier {
            1 => vec![
                "Large Industrial District",
                "Fishing Port",
                "Tree Plantations",
            ],
            2 => vec![
                "Water Cleaning Facilities",
                "Hired Workforces",
                "Small-scale Forges",
            ],
            3 => vec!["Manufactories", "Mercenary Guild"],
            4 => vec!["Teleportation Circle Network", "Strip Mines"],
            5 => vec!["Relic Hunters"],
            _ => panic!(
                "gen_random_building tried to generate a building of tier {:?}",
                tier
            ),
        },
        _ => {
            panic!("got a unkown race");
        }
    };
    race_result.extend(match tier {
        1 => vec![
            "Standard Farms",
            "Standard Mines",
            "Quarry",
            "Forestry Site",
            "Workers",
        ],
        2 => vec![
            "Educated Workers",
            "Well",
            "Glassworks",
            "Wagons",
            "Cloth Mills",
        ],
        3 => vec![
            "Apothecary's Workshop",
            "Basic Industry",
            "Hired Mercenaries",
        ],
        4 => vec!["Modern Artificers", "Modern Soldiers"],
        5 => vec!["Modern Comforts"],
        _ => panic!(
            "gen_random_building tried to generate a building of tier {:?}",
            tier
        ),
    });
    race_result
    //result.to_string()
}

fn quick_hash(array: Vec<(Resources, isize)>) -> HashMap<Resources, isize> {
    array
        .into_iter()
        .map(|value_pair| (value_pair.0, value_pair.1))
        .collect()
}
