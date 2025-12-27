use std::collections::HashMap;

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
    Souls,
    SimpleLabour,
    Military,
    Transportation,
    Luxuries,
    ComplexLabour,
    ExoticAlloys,
    Spellwork,
    Artifacts,
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
            Self::Souls,
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
}

pub fn gen_building_tables() -> HashMap<String, Building> {
    let mut all_buildings = HashMap::new();

    all_buildings.insert(
        "Automated Clothiers".to_string(),
        Building {
            input: quick_hash(vec![(Resources::Machinery, 5), (Resources::Plants, 20)]),
            output: quick_hash(vec![(Resources::Textiles, 15)]),
            image_sylt_id: Some("automated_clothiers".to_string()),
            tier: 1,
        },
    );

    all_buildings.insert(
        "Gem Cutters".to_string(),
        Building {
            input: quick_hash(vec![(Resources::Water, 10), (Resources::RareOre, 10)]),
            output: quick_hash(vec![(Resources::RefinedValuables, 10)]),
            image_sylt_id: Some("gem_cutters".to_string()),
            tier: 1,
        },
    );

    all_buildings.insert(
        "Preparatory Facilities".to_string(),
        Building {
            input: quick_hash(vec![
                (Resources::RefinedValuables, 5),
                (Resources::Machinery, 5),
                (Resources::Plants, 5),
            ]),
            output: quick_hash(vec![(Resources::Reagents, 20)]),
            image_sylt_id: Some("preparatory_facilities".to_string()),
            tier: 1,
        },
    );
    /*
        all_buildings.insert(
            "Tunnelers".to_string(),
            Building {
                input: quick_hash(vec![
                    (Resources::Machinery, 10),
                    (Resources::CommonAlloys, 10),
                    (Resources::Water, 10),
                    (Resources::Coal, 5),
                ]),
                output: quick_hash(vec![(Resources::Transportation, 100)]),
                image_sylt_id: Some("tunnelers".to_string()),
            },
        );

        all_buildings.insert(
            "Growth Vats".to_string(),
            Building {
                input: quick_hash(vec![(Resources::Machinery, 5), (Resources::Plants, 20)]),
                output: quick_hash(vec![(Resources::Medicines, 20)]),
                image_sylt_id: Some("growth_vats".to_string()),
            },
        );

        all_buildings.insert(
            "Automation Components".to_string(),
            Building {
                input: quick_hash(vec![
                    (Resources::Machinery, 10),
                    (Resources::ManufacturedGoods, 10),
                    (Resources::Coal, 10),
                ]),
                output: quick_hash(vec![(Resources::SimpleLabour, 120)]),
                image_sylt_id: Some("automation_components".to_string()),
            },
        );

        all_buildings.insert(
            "Megabreweries".to_string(),
            Building {
                input: quick_hash(vec![
                    (Resources::Reagents, 5),
                    (Resources::Machinery, 5),
                    (Resources::Food, 70),
                ]),
                output: quick_hash(vec![(Resources::Luxuries, 150)]),
                image_sylt_id: Some("megabreweries".to_string()),
            },
        );

        all_buildings.insert(
            "Industrial Smeltery".to_string(),
            Building {
                input: quick_hash(vec![
                    (Resources::CommonOre, 110),
                    (Resources::Coal, 45),
                    (Resources::Machinery, 20),
                ]),
                output: quick_hash(vec![(Resources::CommonAlloys, 100)]),
                image_sylt_id: Some("industrial_smeltery".to_string()),
            },
        );

        all_buildings.insert(
            "Dwarven Assembly Lines".to_string(),
            Building {
                input: quick_hash(vec![
                    (Resources::CommonAlloys, 40),
                    (Resources::Water, 10),
                    (Resources::Coal, 10),
                    (Resources::Stone, 15),
                ]),
                output: quick_hash(vec![(Resources::Machinery, 80)]),
                image_sylt_id: Some("dwarven_assembly_lines".to_string()),
            },
        );

        all_buildings.insert(
            "Adamantium Smeltery".to_string(),
            Building {
                input: quick_hash(vec![
                    (Resources::CommonAlloys, 20),
                    (Resources::RefinedValuables, 40),
                    (Resources::Reagents, 20),
                    (Resources::Machinery, 20),
                    (Resources::RareOre, 30),
                    (Resources::CommonOre, 30),
                    (Resources::Coal, 10),
                ]),
                output: quick_hash(vec![(Resources::ExoticAlloys, 100)]),
                image_sylt_id: Some("adamantium_smeltery".to_string()),
            },
        );
    */
    all_buildings
}

fn quick_hash(array: Vec<(Resources, isize)>) -> HashMap<Resources, isize> {
    array
        .into_iter()
        .map(|value_pair| (value_pair.0, value_pair.1))
        .collect()
}
