use serde::{Deserialize, Serialize};
use std::{fmt, path::StripPrefixError};
use serde_json::{Value};
use std::fs;

use crate::{modifiers::Modifier, sim::Game};

#[derive(Deserialize, Serialize)]
pub struct BaseTeam {
    pub team_name:String,
    delvers:Vec<BaseDelver>, //This is emptied when put into GameTeam
    pub dungeon:Dungeon
}
impl BaseTeam {
    pub fn load_from_file(file:&str,index:usize) -> BaseTeam {
        let contents = fs::read_to_string(file).unwrap();
        let teams:Value = serde_json::from_str(&contents).unwrap();
        let mut teams:Vec<Value> = teams.as_array().unwrap().to_owned();
        serde_json::from_value(teams[index].take()).unwrap()
    }
}

pub struct GameTeam {
    pub delvers:Vec<Delver>,
    pub fighter:usize,
    pub rogue:usize
    // Healer, etc.
}
impl GameTeam {
    pub fn load_team(base: &BaseTeam) -> GameTeam {
        let mut delvers = Vec::new();
        delvers.push(Delver::load_delver(base.delvers[0].clone()));
        delvers.push(Delver::load_delver(base.delvers[1].clone()));
        GameTeam {delvers, fighter:0, rogue:1}
    }
    pub fn get_index(&self, delver:&Delver) -> Option<usize> {
        let mut result = Option::None;
        for d in 0..self.delvers.len() {
            if std::ptr::eq(&self.delvers[d], delver) { result = Option::Some(d); break}
        }
        result
    }
    pub fn choose_delver(&self, stat:DelverStats) -> &Delver {
        match stat {
            DelverStats::Exploriness => &self.delvers[self.rogue],
            DelverStats::Fightiness => &self.delvers[self.fighter],
            DelverStats::Speediness => &self.delvers[self.rogue]
        }
    }
}
pub struct Delver {
    pub base: BaseDelver,
    pub hp:i8,
    pub active:bool,
    pub modifiers:Vec<Box<dyn Modifier>> // Janky,
}
impl Delver {
    // fn new_delver(name: String) -> Delver{
    //     Delver{ base: &BaseDelver::new_delver(name), hp: 5, active:true, modifiers: Vec::new()}
    // }
    pub fn load_delver (base: BaseDelver) -> Delver {
        Delver {base, hp: 5, active: true, modifiers:Vec::new()}
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
#[derive(Clone, Copy)]
pub enum DelverStats {
    Exploriness,
    Fightiness,
    Speediness
}

#[derive(Deserialize, Serialize, Clone)]
pub struct BaseDelver {
    pub name: String,
    pub exploriness: f32,
    pub fightiness: f32,
    pub speediness: f32
}

impl BaseDelver {
    pub fn new_delver(name: String) -> BaseDelver{
        BaseDelver {name, exploriness:0.5, fightiness:0.5, speediness:0.5}
    }
    pub fn to_game_delver(self) -> Delver {
        Delver::load_delver(self)
    }
}
impl fmt::Display for BaseDelver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Deserialize, Serialize, Clone)]
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