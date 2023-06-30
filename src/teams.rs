use serde::{Deserialize, Serialize};
use std::fmt;
use serde_json::{Value};
use std::fs;

use crate::modifiers::Modifier;

#[derive(Deserialize, Serialize)]
pub struct Team {
    pub team_name:String,
    pub delvers:Vec<BaseDelver>,
    pub dungeon:Dungeon
}
impl Team {
    pub fn load_from_file(file:&str,index:usize) -> Team {
        let contents = fs::read_to_string(file).unwrap();
        let teams:Value = serde_json::from_str(&contents).unwrap();
        let mut teams:Vec<Value> = teams.as_array().unwrap().to_owned();
        serde_json::from_value(teams[index].take()).unwrap()
    }
}

pub struct Delver {
    pub base:BaseDelver,
    pub hp:i8,
    pub active:bool,
    pub modifiers:Vec<Box<dyn Modifier>> // Janky,
}
impl Delver {
    fn new_delver(name: String) -> Delver{
        Delver{ base: BaseDelver::new_delver(name), hp: 5, active:true, modifiers: Vec::new()}
    }
    pub fn load_delver(base: BaseDelver) -> Delver {
        Delver { base, hp: 5, active: true, modifiers:Vec::new()}
    }
    fn to_json(self) -> String {
        serde_json::to_string(&self.base).unwrap()
    }
    pub fn get_stat(&self, stat:DelverStats) -> f32 {
        match stat {
            DelverStats::Exploriness => self.base.exploriness,
            DelverStats::Fightiness => self.base.fightiness,
            DelverStats::Speediness => self.base.speediness
        }
    }
}
impl fmt::Display for Delver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.base.fmt(f)
    }
}
pub enum DelverStats {
    Exploriness,
    Fightiness,
    Speediness
}

#[derive(Deserialize, Serialize)]
pub struct BaseDelver {
    pub name: String,
    pub exploriness: f32,
    pub fightiness: f32,
    pub speediness: f32
}

impl BaseDelver {
    fn new_delver(name: String) -> BaseDelver{
        BaseDelver {name, exploriness:0.5, fightiness:0.5, speediness:0.5}
    }
}
impl fmt::Display for BaseDelver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Deserialize, Serialize)]
pub struct Dungeon {
    pub name: String,
    pub twistiness: f32,
    pub deadliness: f32,
    pub lengthiness: f32
}
impl Dungeon {
    pub fn new_dungeon(name: String) -> Dungeon{
        Dungeon {name, twistiness:0.5, deadliness:0.5, lengthiness:0.5}
    }
}