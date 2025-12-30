use super::strategic_map::*;
use crate::prelude::*;
use crate::{game::market, network::message::PlayerId};

use super::market::*;
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Reflect, Component, Default, Clone, Debug, Serialize, Deserialize)]
pub struct CityData {
    pub id: String,
    pub race: BuildingType,
    pub population: u8,
    pub buildings_t1: Vec<(String, Faction, (bool, bool))>,
    pub buildings_t2: Vec<(String, Faction, (bool, bool))>,
    pub buildings_t3: Vec<(String, Faction, (bool, bool))>,
    pub buildings_t4: Vec<(String, Faction, (bool, bool))>,
    pub buildings_t5: Vec<(String, Faction, (bool, bool))>,
    pub market: HashMap<Resources, isize>,
    pub warehouses: HashMap<PlayerId, HashMap<Resources, isize>>,
    pub tier_up_counter: u8,
}

impl CityData {
    pub fn overwrite(&mut self, other: &CityData) {
        self.id = other.id.clone();
        self.race = other.race;
        self.population = other.population;
        self.buildings_t1 = other.buildings_t1.clone();
        self.buildings_t2 = other.buildings_t2.clone();
        self.buildings_t3 = other.buildings_t3.clone();
        self.buildings_t4 = other.buildings_t4.clone();
        self.buildings_t5 = other.buildings_t5.clone();
        self.market = other.market.clone();
        self.warehouses = other.warehouses.clone();
        self.tier_up_counter = other.tier_up_counter;
    }

    pub fn new(
        name: String,
        race: BuildingType,
        tier: u8,
        mut rng: &mut ResMut<GlobalRng>,
        players: Query<&Player>,
    ) -> CityData {
        let buildings_per_tier = match tier {
            1 => (1, 0, 0, 0, 0),
            2 => (1, 1, 0, 0, 0),
            3 => (2, 1, 1, 0, 0),
            4 => (2, 2, 1, 1, 0),
            5 => (3, 2, 2, 1, 1),
            _ => {
                panic!("Tried to generate a city of tier {:?}", tier)
            }
        };
        let (mut t1, mut t2, mut t3, mut t4, mut t5) = (vec![], vec![], vec![], vec![], vec![]);

        for _i in 0..buildings_per_tier.0 {
            t1.push((
                (market::gen_random_building(1, &mut rng, race)),
                Faction::Neutral,
                (false, false),
            ));
        }

        for _i in 0..buildings_per_tier.1 {
            t2.push((
                (market::gen_random_building(2, &mut rng, race)),
                Faction::Neutral,
                (false, false),
            ));
        }

        for _i in 0..buildings_per_tier.2 {
            t3.push((
                (market::gen_random_building(3, &mut rng, race)),
                Faction::Neutral,
                (false, false),
            ));
        }

        for _i in 0..buildings_per_tier.3 {
            t4.push((
                (market::gen_random_building(4, &mut rng, race)),
                Faction::Neutral,
                (false, false),
            ));
        }

        for _i in 0..buildings_per_tier.4 {
            t5.push((
                (market::gen_random_building(5, &mut rng, race)),
                Faction::Neutral,
                (false, false),
            ));
        }

        let mut market = HashMap::new();
        for res in Resources::all_resources() {
            market.insert(res, 0);
        }

        let mut warehouses = HashMap::new();

        //println!("players: {0:?}", players);
        for player in players {
            warehouses.insert(player.player_id, market.clone());
        }

        CityData {
            id: name,
            race: race,
            population: tier,
            buildings_t1: t1,
            buildings_t2: t2,
            buildings_t3: t3,
            buildings_t4: t4,
            buildings_t5: t5,
            market: market,
            warehouses: warehouses,
            tier_up_counter: 0,
        }
    }

    pub fn get_resource_value_modifier(&self, res: &Resources) -> f64 {
        let Some(total) = self.market.get(res) else {
            panic!("tried to find resource {res:?} but the resource was missing")
        };

        let sigmoid =
            2.0 / (1.0 + (std::f64::consts::E).powf((*total) as f64 * 1.0 / 200.0)) as f64;
        sigmoid.max(0.3)
    }

    fn get_theoretical_resource_value_modifier(&self, _res: &Resources, amount: isize) -> f64 {
        let sigmoid = 2.0 / (1.0 + (std::f64::consts::E).powf(amount as f64 * 1.0 / 200.0));
        sigmoid.max(0.3)
    }

    pub fn get_resource_value(&self, res: &Resources) -> f64 {
        self.get_resource_value_modifier(res) * res.get_base_value() as f64
    }

    fn get_theoretical_resource_value(&self, res: &Resources, amount: isize) -> f64 {
        self.get_theoretical_resource_value_modifier(res, amount) * res.get_base_value() as f64
    }

    pub fn get_bulk_buy_price(&self, res: &Resources, amount: usize) -> f64 {
        let mut amount_available = *self.market.get(res).expect(
            format!(
                "tried to find resource {:?} but the resource was missing in internal market",
                res
            )
            .as_str(),
        );
        let mut total_cost = 0.0;
        for _i in 0..amount {
            let price = self.get_theoretical_resource_value(res, amount_available);
            total_cost += price;
            amount_available -= 1;
        }
        total_cost
    }

    pub fn get_bulk_sell_price(&self, res: &Resources, amount: usize) -> f64 {
        let mut amount_available = *self.market.get(res).expect(
            format!(
                "tried to find resource {:?} but the resource was missing in internal market",
                res
            )
            .as_str(),
        );
        let mut total_profit = 0.0;
        for _i in 0..amount {
            let price = self.get_theoretical_resource_value(res, amount_available);
            total_profit += price;
            amount_available += 1;
        }
        total_profit
    }

    pub fn available_commodities(&self, building_table: &Res<BuildinTable>) -> Vec<Resources> {
        let mut resources: HashMap<Resources, isize> = HashMap::new();
        macro_rules! get_outputs {
            ($list:expr) => {
                for b in &$list {
                    if b.1 != Faction::Neutral {
                        continue;
                    }
                    for (res, amount) in &building_table
                        .0
                        .get(&b.0)
                        .expect(format!("Couldn't retrieve value for {:?}", &b.0).as_str())
                        .input
                    {
                        resources.insert(
                            *res,
                            resources
                                .get(res)
                                .or_else(|| -> Option<&isize> { Some(&0) })
                                .expect(format!("bruh value {:?}", res).as_str())
                                - amount,
                        );
                    }
                    for (res, amount) in &building_table
                        .0
                        .get(&b.0)
                        .expect(format!("Couldn't retrieve value for {:?}", &b.0).as_str())
                        .output
                    {
                        resources.insert(
                            *res,
                            resources
                                .get(res)
                                .or_else(|| -> Option<&isize> { Some(&0) })
                                .expect(format!("bruh value {:?}", res).as_str())
                                + amount,
                        );
                    }
                }
            };
        }

        get_outputs!(self.buildings_t1);
        get_outputs!(self.buildings_t2);
        get_outputs!(self.buildings_t3);
        get_outputs!(self.buildings_t4);
        get_outputs!(self.buildings_t5);

        let mut hash = resources
            .iter()
            .filter(|(_k, v)| v >= &&0)
            .map(|(k, _v)| *k)
            .collect::<HashSet<Resources>>();

        for (res, amount) in self.market.clone() {
            if amount > 0 {
                hash.insert(res);
            }
        }

        hash.iter().map(|x| *x).collect::<Vec<_>>()
    }

    #[rustfmt::skip]
    pub fn update_market(&mut self, building_table: &Res<BuildinTable>, mut players: &mut Query<&mut Player>) {
        macro_rules! update_market_over_buildings {
            ($list:expr) => {
                for b in &$list {
                    if b.1 != Faction::Neutral {
                        continue;
                    }
                    for (res, amount) in &building_table
                        .0
                        .get(&b.0)
                        .expect(format!("Couldn't retrieve value for {:?}", &b.0).as_str())
                        .input
                    {
                        self.market.insert(*res, self.market[&res] - amount);
                    }
                    for (res, amount) in &building_table
                        .0
                        .get(&b.0)
                        .expect(format!("Couldn't retrieve value for {:?}", &b.0).as_str())
                        .output
                    {
                        self.market.insert(*res, self.market[&res] + amount);
                    }
                }
            };
        }

        macro_rules! update_player_buildings {
            ($list:expr) => {
                for b in &$list {
                    let Faction::Player(player_id) = b.1 else { continue; };
                    let building = &building_table.0
                                                .get(&b.0)
                                                .expect(format!("Couldn't retrieve value for {:?}", &b.0).as_str());
                    let mut demands_met = false;
                    if b.2.0 {
                        let mut warehouse_meets_demands = true;
                        for (res, amount) in &building.input {
                            warehouse_meets_demands = warehouse_meets_demands
                                && amount <= self.warehouses
                                                                            .get(&(player_id as u64))
                                                                            .expect(format!("PlayerId {:?} doesn't exist", player_id).as_str())
                                                                            .get(&res)
                                                                            .expect(format!("Warehouse for player {:?} improperly initialized", player_id).as_str());
                        }
                        if warehouse_meets_demands {
                            demands_met = true;
                            for (res, amount) in &building.input {
                                let cur_amount = self.warehouses.get_mut(&(player_id as u64))
                                                                .expect(&format!("PlayerId {:?} doesn't exist", player_id))
                                                                .entry(*res)
                                                                .or_insert(0);
                                *cur_amount -= amount;
                            }
                        }
                    } else {
                        let mut market_meets_demands = true;
                        for (res, _) in &building.input {
                            market_meets_demands = market_meets_demands
                                && self.available_commodities(&building_table).contains(&res);
                        }
                        if market_meets_demands {
                            for (res, amount) in &building.input {
                                let price = self.get_bulk_buy_price(&res, *amount as usize);
                                players.iter_mut().find(|x| x.player_id == player_id as u64).expect("building belongs to player {player_id} but no such player exists").money -= price;
                                let market_amount = self.market.entry(*res).or_insert(0);
                                *market_amount -= amount;
                            }
                            demands_met = true;
                        }
                    }

                    if demands_met {
                        if b.2.1 {
                            for (res, amount) in &building.output {
                                let cur_amount = self.warehouses.get_mut(&(player_id as u64))
                                                                .expect(&format!("PlayerId {player_id} doesn't exist"))
                                                                .entry(*res)
                                                                .or_insert(0);

                                *cur_amount += amount;
                            }
                        } else {
                            for (res, amount) in &building.output {
                                let price = self.get_bulk_sell_price(&res, *amount as usize);
                                players.iter_mut().find(|x| x.player_id == player_id as u64)
                                                    .expect("building belongs to player {player_id} but no such player exists")
                                                    .money += price;

                                let market_amount = self.market.entry(*res).or_insert(0);
                                *market_amount += amount;
                            }
                        }
                    }
                }
            }
        }

        update_market_over_buildings!(self.buildings_t1);
        update_market_over_buildings!(self.buildings_t2);
        update_market_over_buildings!(self.buildings_t3);
        update_market_over_buildings!(self.buildings_t4);
        update_market_over_buildings!(self.buildings_t5);

        update_player_buildings!(self.buildings_t1);
        update_player_buildings!(self.buildings_t2);
        update_player_buildings!(self.buildings_t3);
        update_player_buildings!(self.buildings_t4);
        update_player_buildings!(self.buildings_t5);

        let match_condition = self.population;

        let mut tier_up = |condition: bool| {
            if condition {
                if self.tier_up_counter == 5 {
                    self.tier_up_counter = 0;
                    self.population += 1;
                } else {
                    self.tier_up_counter += 1;
                }
            } else {
                self.tier_up_counter = 0;
            }
        };

        match match_condition {
            1 => {
                tier_up(self.market.insert(Resources::Food, self.market[&Resources::Food] - 5).expect("error in city market") - 5 >= 0 &&
                        self.market.insert(Resources::Water, self.market[&Resources::Water] - 3).expect("error in city market") - 3 >= 0 &&
                        self.market.insert(Resources::Lumber, self.market[&Resources::Lumber] - 2).expect("error in city market") - 2 >= 0);
                self.market.insert(Resources::SimpleLabour, self.market[&Resources::SimpleLabour] + 5);
            },
            2 => {
                tier_up(self.market.insert(Resources::Food, self.market[&Resources::Food] - 15).expect("error in city market") - 15 >= 0 &&
                        self.market.insert(Resources::Water, self.market[&Resources::Water] - 10).expect("error in city market") - 10 >= 0 &&
                        self.market.insert(Resources::Lumber, self.market[&Resources::Lumber] - 5).expect("error in city market") - 5 >= 0 &&
                        self.market.insert(Resources::Stone, self.market[&Resources::Stone] - 3).expect("error in city market") - 3 >= 0 &&
                        self.market.insert(Resources::Glass, self.market[&Resources::Glass] - 3).expect("error in city market") - 3 >= 0 &&
                        self.market.insert(Resources::Textiles, self.market[&Resources::Textiles] - 3).expect("error in city market") - 3 >= 0);
                self.market.insert(Resources::SimpleLabour, self.market[&Resources::SimpleLabour] + 20);
                self.market.insert(Resources::ComplexLabour, self.market[&Resources::ComplexLabour] + 5);
            },
            3 => {
                tier_up(self.market.insert(Resources::Food, self.market[&Resources::Food] - 20).expect("error in city market") - 20 >= 0 &&
                        self.market.insert(Resources::Water, self.market[&Resources::Water] - 15).expect("error in city market") - 15 >= 0 &&
                        self.market.insert(Resources::Lumber, self.market[&Resources::Lumber] - 10).expect("error in city market") - 10 >= 0 &&
                        self.market.insert(Resources::Stone, self.market[&Resources::Stone] - 6).expect("error in city market") - 6 >= 0 &&
                        self.market.insert(Resources::Glass, self.market[&Resources::Glass] - 6).expect("error in city market") - 6 >= 0 &&
                        self.market.insert(Resources::Textiles, self.market[&Resources::Textiles] - 6).expect("error in city market") - 6 >= 0 &&
                        self.market.insert(Resources::Medicines, self.market[&Resources::Medicines] - 3).expect("error in city market") - 3 >= 0 &&
                        self.market.insert(Resources::ManufacturedGoods, self.market[&Resources::ManufacturedGoods] - 3).expect("error in city market") - 3 >= 0 &&
                        self.market.insert(Resources::Luxuries, self.market[&Resources::Luxuries] - 15).expect("error in city market") - 15 >= 0 &&
                        self.market.insert(Resources::Transportation, self.market[&Resources::Transportation] - 15).expect("error in city market") - 15 >= 0);
                self.market.insert(Resources::Drugs, self.market[&Resources::Drugs] - 5).expect("error in city market");
                self.market.insert(Resources::Slaves, self.market[&Resources::Slaves] - 5).expect("error in city market");
                self.market.insert(Resources::SimpleLabour, self.market[&Resources::SimpleLabour] + 45);
                self.market.insert(Resources::ComplexLabour, self.market[&Resources::ComplexLabour] + 20);
            },
            4 => {
                tier_up(self.market.insert(Resources::Food, self.market[&Resources::Food] - 50).expect("error in city market") - 50 >= 0 &&
                        self.market.insert(Resources::Water, self.market[&Resources::Water] - 30).expect("error in city market") - 30 >= 0 &&
                        self.market.insert(Resources::Lumber, self.market[&Resources::Lumber] - 20).expect("error in city market") - 20 >= 0 &&
                        self.market.insert(Resources::Stone, self.market[&Resources::Stone] - 15).expect("error in city market") - 15 >= 0 &&
                        self.market.insert(Resources::Glass, self.market[&Resources::Glass] - 15).expect("error in city market") - 15 >= 0 &&
                        self.market.insert(Resources::Textiles, self.market[&Resources::Textiles] - 15).expect("error in city market") - 15 >= 0 &&
                        self.market.insert(Resources::Medicines, self.market[&Resources::Medicines] - 10).expect("error in city market") - 10 >= 0 &&
                        self.market.insert(Resources::ManufacturedGoods, self.market[&Resources::ManufacturedGoods] - 10).expect("error in city market") - 10 >= 0 &&
                        self.market.insert(Resources::Luxuries, self.market[&Resources::Luxuries] - 25).expect("error in city market") - 25 >= 0 &&
                        self.market.insert(Resources::Transportation, self.market[&Resources::Transportation] - 25).expect("error in city market") - 25 >= 0 &&
                        self.market.insert(Resources::Military, self.market[&Resources::Military] - 15).expect("error in city market") - 15 >= 0);
                self.market.insert(Resources::Drugs, self.market[&Resources::Drugs] - 10).expect("error in city market");
                self.market.insert(Resources::Slaves, self.market[&Resources::Slaves] - 10).expect("error in city market");
                self.market.insert(Resources::Vitae, self.market[&Resources::Vitae] - 2).expect("error in city market");
                self.market.insert(Resources::SimpleLabour, self.market[&Resources::SimpleLabour] + 80);
                self.market.insert(Resources::ComplexLabour, self.market[&Resources::ComplexLabour] + 45);
            },
            5 => {
                self.market.insert(Resources::Food, self.market[&Resources::Food] - 100);
                self.market.insert(Resources::Water, self.market[&Resources::Water] - 50);
                self.market.insert(Resources::Lumber, self.market[&Resources::Lumber] - 40);
                self.market.insert(Resources::Stone, self.market[&Resources::Stone] - 30);
                self.market.insert(Resources::Glass, self.market[&Resources::Glass] - 30);
                self.market.insert(Resources::Textiles, self.market[&Resources::Textiles] - 20);
                self.market.insert(Resources::Medicines, self.market[&Resources::Medicines] - 20);
                self.market.insert(Resources::ManufacturedGoods, self.market[&Resources::ManufacturedGoods] - 20);
                self.market.insert(Resources::Luxuries, self.market[&Resources::Luxuries] - 60);
                self.market.insert(Resources::Transportation, self.market[&Resources::Transportation] - 60);
                self.market.insert(Resources::Military, self.market[&Resources::Military] - 50);
                self.market.insert(Resources::Drugs, self.market[&Resources::Drugs] - 20).expect("error in city market");
                self.market.insert(Resources::Slaves, self.market[&Resources::Slaves] - 20).expect("error in city market");
                self.market.insert(Resources::Vitae, self.market[&Resources::Vitae] - 5).expect("error in city market");
                self.market.insert(Resources::SimpleLabour, self.market[&Resources::SimpleLabour] + 125);
                self.market.insert(Resources::ComplexLabour, self.market[&Resources::ComplexLabour] + 80);
            }
            _ => { panic!("Tried to update markets on a city of tier {:?}", self.population) }
        }
    }
}
