use serde::{Deserialize, Serialize};
use std::{fmt};
use serde_json::{Value};
use std::fs;

use crate::{modifiers::{Modifier, PermanentModifiers}};

#[derive(Deserialize, Serialize)]
pub struct BaseTeam {
    pub team_name:String,
    delvers:Vec<BaseDelver>, //This is emptied when put into GameTeam
    pub dungeon:Dungeon,
    defenders:Vec<BaseDefender>
}
impl BaseTeam {
    pub fn load_from_file(file:&str,index:usize) -> BaseTeam {
        let contents = fs::read_to_string(file).unwrap();
        let teams:Value = serde_json::from_str(&contents).unwrap();
        let mut teams:Vec<Value> = teams.as_array().unwrap().to_owned();
        serde_json::from_value(teams[index].take()).unwrap()
    }
}

pub struct DefenderTeam {
    pub defender:Defender,
    pub dungeon:Dungeon
}
impl DefenderTeam {
    pub fn load_team(base: &BaseTeam) -> DefenderTeam {
        let defender = Defender::load_defender(base.defenders[0].clone());
        DefenderTeam { defender, dungeon:base.dungeon.clone() }
    }
}
pub struct DelverTeam {
    pub delvers:Vec<Delver>,
    pub fighter:usize,
    pub nimble:usize,
    pub magic:usize,
    pub support:usize
    // Healer, etc.
}
impl DelverTeam {
    pub fn load_team(base: &BaseTeam) -> DelverTeam {
        let mut delvers = Vec::new();
        delvers.push(Delver::load_delver(base.delvers[0].clone()));
        delvers.push(Delver::load_delver(base.delvers[1].clone()));
        delvers.push(Delver::load_delver(base.delvers[2].clone()));
        delvers.push(Delver::load_delver(base.delvers[3].clone()));
        DelverTeam {delvers, fighter:0, nimble:1, magic:2, support:3}
    }
    pub fn get_index(&self, delver:&Delver) -> Option<usize> {
        let mut result = Option::None;
        for d in 0..self.delvers.len() {
            if std::ptr::eq(&self.delvers[d], delver) { result = Option::Some(d); break}
        }
        result
    }
    pub fn active_delvers(&self) -> Vec<usize> {
        let mut results = Vec::new();
        if self.delvers[self.fighter].active {results.push(self.fighter)}
        if self.delvers[self.magic].active {results.push(self.magic)}
        if self.delvers[self.nimble].active {results.push(self.nimble)}
        if self.delvers[self.support].active {results.push(self.support)}
        results
    }
    pub fn choose_delver(&self, stat:DelverStats) -> &Delver {
        let delver = match stat {
            DelverStats::Exploriness => &self.delvers[self.nimble],
            DelverStats::Fightiness => &self.delvers[self.fighter],
            DelverStats::Speediness => &self.delvers[self.nimble]
        };
        if delver.active {return delver}
        
        for d in &self.delvers {
            if d.active {
                return d
            }
        }
        panic!("All delvers dead.")
    }
}
pub struct Delver {
    pub base: BaseDelver,
    pub hp:i8,
    pub active:bool,
    pub modifiers:Vec<Box<dyn Modifier>>
}
impl Delver {
    // fn new_delver(name: String) -> Delver{
    //     Delver{ base: &BaseDelver::new_delver(name), hp: 5, active:true, modifiers: Vec::new()}
    // }
    pub fn load_delver (base: BaseDelver) -> Delver {
        let mut modifiers = Vec::new();
        for i in &base.perm_mods {
            modifiers.push(PermanentModifiers::to_game_mod(i));
        }
        Delver {base, hp: 5, active: true, modifiers}
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.base).unwrap()
    }
    pub fn collect_stats(active_delver:&Delver, all_delvers:&Vec<Delver>, stat:DelverStats) -> f32 {
        let mut total = 0.0;
        total += active_delver.get_stat(stat) * 0.8; // Active_delver should be in party, so 0.1 will also get added.
        for d in all_delvers {
            total += d.get_stat(stat) * 0.1;
        }
        total
    }
    pub fn get_stat(&self, stat:DelverStats) -> f32 {
        let mut statvalue =  match stat {
            DelverStats::Exploriness => self.base.exploriness,
            DelverStats::Fightiness => self.base.fightiness,
            DelverStats::Speediness => self.base.speediness
        };
        for m in &self.modifiers {
            statvalue = m.get_delver_stat(stat, statvalue)
        };
        statvalue
    }
}

impl fmt::Display for Delver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.base.fmt(f)
    }
}
#[derive(Clone, Copy,Debug)]
pub enum DelverStats {
    Exploriness,
    Fightiness,
    Speediness
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BaseDelver {
    pub name: String,
    pub exploriness: f32,
    pub fightiness: f32,
    pub speediness: f32,
    pub perm_mods: Vec<PermanentModifiers>
}

impl BaseDelver {
    pub fn new_delver(name: String) -> BaseDelver{
        BaseDelver {name, exploriness:0.5, fightiness:0.5, speediness:0.5, perm_mods:Vec::new()}
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

#[derive(Clone, Copy)]
pub enum DefenderStats {
    Fightiness
}

pub struct Defender {
    pub base: BaseDefender,
    pub hp:i8,
    pub active:bool,
    pub modifiers:Vec<Box<dyn Modifier>>
}
impl Defender {
    pub fn load_defender (base: BaseDefender) -> Defender {
        let mut modifiers = Vec::new();
        for i in &base.perm_mods {
            modifiers.push(PermanentModifiers::to_game_mod(i));
        }
        Defender {base, hp: 5, active: true, modifiers}
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.base).unwrap()
    }
    pub fn get_stat(&self, stat:DefenderStats) -> f32 {
        let mut statvalue =  match stat {
            DefenderStats::Fightiness => self.base.fightiness
        };
        for m in &self.modifiers {
            statvalue = m.get_defender_stat(stat, statvalue)
        };
        statvalue
    }
}
impl fmt::Display for Defender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.base.fmt(f)
    }
}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BaseDefender {
    pub name: String,
    pub fightiness: f32,
    pub perm_mods: Vec<PermanentModifiers>
}
impl BaseDefender {
    pub fn new_delver(name: String) -> BaseDefender{
        BaseDefender {name, fightiness:0.5, perm_mods:Vec::new()}
    }
    pub fn to_game_defender(self) -> Defender {
        Defender::load_defender(self)
    }
}
impl fmt::Display for BaseDefender {
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