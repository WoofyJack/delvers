// Entities as they are stored between games. Dungeons aren't right now. They'll probably be split into base and active dungeons later.

use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::{fmt, fs};

use crate::modifiers::BaseModifier;
use crate::entities::{Delver, Defender, Dungeon};
use colored::{Colorize, ColoredString};

#[derive(Deserialize, Serialize)]
pub struct BaseTeam {
    pub team_name:String,
    pub delvers:Vec<BaseDelver>, //This is emptied when put into GameTeam
    pub dungeon:Dungeon,
    pub defenders:Vec<BaseDefender>,
    color:[u8;3]
}
impl BaseTeam {
    pub fn load_from_file(file:&str,index:usize) -> BaseTeam {
        let contents = fs::read_to_string(file).unwrap();
        let teams:Value = serde_json::from_str(&contents).unwrap();
        let mut teams:Vec<Value> = teams.as_array().unwrap().to_owned();
        serde_json::from_value(teams[index].take()).unwrap()
    }
}
impl fmt::Display for BaseTeam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.team_name.truecolor(self.color[0], self.color[1], self.color[2]))
    }
}






// ------------------- Base Characters -------------------

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BaseDefender {
    pub name: String,
    pub exploriness: f32,
    pub fightiness: f32,
    pub magiciness: f32,
    pub supportiveness: f32,
    pub perm_mods: Vec<BaseModifier>
}
impl BaseDefender {
    pub fn new_delver(name: String) -> BaseDefender{
        BaseDefender {name, fightiness:0.5, magiciness:0.5, exploriness:0.5, supportiveness:0.5, perm_mods:Vec::new()}
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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BaseDelver {
    pub name: String,
    pub exploriness: f32,
    pub fightiness: f32,
    pub magiciness: f32,
    pub supportiveness: f32,
    pub perm_mods: Vec<BaseModifier>
}
impl BaseDelver {
    pub fn new_delver(name: String) -> BaseDelver{
        BaseDelver {name, exploriness:0.5, fightiness:0.5, supportiveness:0.5,magiciness:0.5, perm_mods:Vec::new()}
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