use std::collections::HashMap;

use bevy::pbr::generate;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Resources {
    Food,
    Plants,
    CommonOre,
    RareOre,
    Lumber,
    Stone,
    Water,
    Glass,
    Coal,
    RefinedValuables,
    CommonAlloys,
    Textiles,
    ManufacturedGoods,
    Medicines,
    Reagents,
    Machinery,
    Drugs,
    Slaves,
    Vitae,
    SimpleLabour,
    Military,
    Transportation,
    Luxuries,
    ComplexLabour,
    ExoticAlloys,
    Spellwork,
    Artifacts,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum BuildingType {
    Human,
    Elven,
    Goblin,
    Dwarven,
    #[default]
    Generic,
    Illegal
}

impl Resources {
    pub fn all_resources() -> [Self; 27] {
        [
            Self::Food,
            Self::Plants,
            Self::CommonOre,
            Self::RareOre,
            Self::Lumber,
            Self::Stone,
            Self::Water,
            Self::Glass,
            Self::Coal,
            Self::RefinedValuables,
            Self::CommonAlloys,
            Self::Textiles,
            Self::ManufacturedGoods,
            Self::Medicines,
            Self::Reagents,
            Self::Machinery,
            Self::Drugs,
            Self::Slaves,
            Self::Vitae,
            Self::SimpleLabour,
            Self::Military,
            Self::Transportation,
            Self::Luxuries,
            Self::ComplexLabour,
            Self::ExoticAlloys,
            Self::Spellwork,
            Self::Artifacts,
        ]
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Building {
    input: HashMap<Resources, isize>,
    output: HashMap<Resources, isize>,
    tier: usize,
    image_sylt_id: Option<String>,
    build_type: BuildingType
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
    generate_building!("Megabreweries", Reagents x 5, Machinery x 5, Food x 70; Luxuries x 150; 3);
    generate_building!("Industrial Smeltery", CommonOre x 100, Coal x 40, Machinery x 20, SimpleLabour x 15; CommonAlloys x 100; 4);
    generate_building!("Dwarven Assembly Lines", CommonAlloys x 40, Water x 10, Coal x 10, Stone x 10, SimpleLabour x 5; Machinery x 80; 4);
    generate_building!("Adamantium Smeltery", CommonAlloys x 20, RefinedValuables x 40, Reagents x 20, Machinery x 20, RareOre x 30, CommonOre x 20, Coal x 10, ComplexLabour x 10; ExoticAlloys x 100; 5);

    //Elven buildings
    current_type = BuildingType::Elven;
    generate_building!("Earth Spirit Aid", Spellwork x 1; SimpleLabour x 15; 1);
    generate_building!("Ironwood Forestry", Spellwork x 1, SimpleLabour x 15; CommonAlloys x 10; 1);
    generate_building!("Forest Foraging", Spellwork x 1, SimpleLabour x 15; Lumber x 30; 1);
    generate_building!("Domesticated Orchards", Spellwork x 5, Water x 10, SimpleLabour x 20, ComplexLabour x 70; ManufacturedGoods x 50; 2);
    generate_building!("Amber Plnatation", Spellwork x 5, Water x 5, SimpleLabour x 5; RefinedValuables x 20; 2);
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
    generate_building!("Golem Automatons", Artifacts x 10, ExoticAlloys x 10, Reagents x 10, Stone x 5; ComplexLabour x 220; 4);
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

    all_buildings
}

fn quick_hash(array: Vec<(Resources, isize)>) -> HashMap<Resources, isize> {
    array
        .into_iter()
        .map(|value_pair| (value_pair.0, value_pair.1))
        .collect()
}
