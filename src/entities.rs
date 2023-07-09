use serde::{Serialize, Deserialize};
use std::{fmt};
use rand::Rng;
use colored::{Colorize, ColoredString};
use crate::{room_types::Coordinate, sim::Game, room_types::RoomType, combat::Monster,
    base_entities::{BaseDefender,BaseDelver,BaseTeam},
    modifiers::BaseModifier};

#[derive(Clone, Copy,Debug, Deserialize, Serialize)]
pub enum Stats {
    Exploriness,
    Fightiness,
    Magiciness,
    Supportiveness
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Entity { // Losely defined as "Something that can have modifiers and be targetted by stuff"
    Delver {index:usize},
    Defender {index:usize},
    Room {index:Coordinate},
    Dungeon,
    DelverTeam,
    DefenderTeam,
    None
}
impl Entity {
    pub fn to_string(&self, game:&Game) -> String{
        match self {
            Entity::Delver{index} => {game.delverteam.delvers[*index].to_string()},
            Entity::Defender {index} => {game.defenderteam.active_defenders[*index].to_string()},
            _ => panic!("Invalid to_string")
        }
    }
    pub fn get_delver_index(self) -> usize {
        match self {
            Entity::Delver {index} => index,
            _ => panic!("Expected delver")
        }
    }
    pub fn get_stat(&self,game:&Game, stat:Stats) -> f32 {
        match self {
            Entity::Delver {index} => game.delverteam.delvers[*index].get_stat(stat),
            Entity::Defender{index} => game.defenderteam.active_defenders[*index].get_stat(stat),
            _ => panic!("Expected delver or defender")
        }
    }
    pub fn collect_stats(&self, game:&Game, stat:Stats) -> f32 {
        match self {
            Entity::Delver {index} => Delver::collect_stats(self, &game.delverteam.delvers, stat),
            Entity::Defender{index} =>  Defender::collect_stats(self, &game.defenderteam.active_defenders, stat),
            _ => panic!("Expected delver or defender")
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Delver {
    base: BaseDelver,
    pub hp:i8,
    pub maxhp:i8,
    pub active:bool,
    pub modifiers:Vec<BaseModifier>
}

impl Delver {
    pub fn load_delver (base: BaseDelver) -> Delver {
        let modifiers = base.perm_mods.clone();
        Delver {base, hp: 5, maxhp:5, active: true, modifiers}
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.base).unwrap()
    }
    pub fn collect_stats(active_delver:&Entity, all_delvers:&Vec<Delver>, stat:Stats) -> f32 {
        let active_delver = match active_delver {
            Entity::Delver { index } => *index,
            _ => panic!("Invalid stats collected")
        };
        let active_delver = &all_delvers[active_delver];
        let mut total = 0.0;
        total += active_delver.get_stat(stat) * 0.75; // Active_delver should be in party, so 0.25 will also get added.
        for d in all_delvers {
            total += d.get_stat(stat) * 0.25;
        }
        total
    }
    pub fn get_stat(&self, stat:Stats) -> f32 {
        let mut statvalue =  match stat {
            Stats::Exploriness => self.base.exploriness,
            Stats::Fightiness => self.base.fightiness,
            Stats::Magiciness => self.base.magiciness,
            Stats::Supportiveness => self.base.supportiveness
        };
        for m in &self.modifiers {
            statvalue = m.get_stat(stat, statvalue)
        };
        statvalue
    }
}

impl fmt::Display for Delver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut name = self.base.name.truecolor(252, 36, 0);
        if !self.active {
            name = name.truecolor(100, 100, 100);
        };

        write!(f, "{}", name)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Defender {
    base: BaseDefender,
    pub hp:i8,
    pub maxhp:i8,
    pub active:bool,
    pub modifiers:Vec<BaseModifier>
}
impl Defender {
    pub fn load_defender (base: BaseDefender) -> Defender {
        let modifiers = base.perm_mods.clone();
        Defender {base, hp: 5, maxhp:5, active: true, modifiers}
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.base).unwrap()
    }
    pub fn collect_stats(active_defender:&Entity, all_defenders:&Vec<Defender>, stat:Stats) -> f32 {
        let active_defender = match active_defender {
            Entity::Defender { index } => *index,
            _ => panic!("Invalid stats collected")
        };
        let active_defender = &all_defenders[active_defender];
        let mut total: f32 = 0.0;
        total += active_defender.get_stat(stat) * 0.75; // Active_delver should be in party, so 0.25 will also get added.
        for d in all_defenders {
            total += d.get_stat(stat) * 0.25;
        }
        total
    }
    pub fn get_stat(&self, stat:Stats) -> f32 {
        let mut statvalue =  match stat {
            Stats::Exploriness => self.base.exploriness,
            Stats::Fightiness => self.base.fightiness,
            Stats::Magiciness => self.base.magiciness,
            Stats::Supportiveness => self.base.supportiveness
        };
        for m in &self.modifiers {
            statvalue = m.get_stat(stat, statvalue)
        };
        statvalue
    }
}
impl fmt::Display for Defender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut name = self.base.name.normal();
        name = if self.active {
            name.truecolor(38,6,215)
        }
        else {
            name.truecolor(100, 100, 100)
        };

        write!(f, "{}", name)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Room {
    pub complete:bool,
    pub room_type:RoomType
}
impl Room {
    pub fn new_room(rng: &mut impl Rng) -> Room {
        let room_type = match rng.gen_range(0..3) {
            0..=1 => RoomType::Arcane,
            2..=4 => RoomType::Trapped,
            5 => {
                let zombie = Monster{name: String::from("Zombie Pirate"), difficulty: 0.2, hp:3, maxhp:3};
                RoomType::Fight { monsters: vec![zombie.clone(),zombie.clone(),zombie], partyname:String::from("a horde of zombie pirates!")}
            }
            _ => panic!("Fix the rng range"),
        };

        Room {complete: false, room_type}
    }
}


// -------------------------------- Delver Team -------------------------
#[derive(Serialize, Deserialize)]
pub struct DelverTeam {
    name:String,
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
        DelverTeam {name:base.team_name.clone(), delvers, fighter:0, nimble:1, magic:2, support:3}
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
    pub fn choose_delver(&self, stat:Stats) -> Entity {
        let delver = match stat {
            Stats::Exploriness => self.nimble,
            Stats::Fightiness => self.fighter,
            Stats::Magiciness => self.magic,
            Stats::Supportiveness => self.support
        };
        if self.delvers[delver].active {return Entity::Delver { index: delver}}
        
        for d in &self.delvers {
            if d.active {
                let index = self.get_index(d).unwrap();
                return Entity::Delver { index}
            }
        }
        panic!("All delvers dead.")
    }
}
impl fmt::Display for DelverTeam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name.truecolor(252, 36, 0))
    }
}



// -------------------------- Defender Team -------------

#[derive(Serialize, Deserialize)]
pub struct DefenderTeam {
    name:String,
    pub defender:BaseDefender,
    pub active_defenders:Vec<Defender>,
    pub dungeon:Dungeon
}
impl DefenderTeam {
    pub fn load_team(base: &BaseTeam) -> DefenderTeam {
        // let defender = Defender::load_defender(base.defenders[0].clone());
        
        DefenderTeam {name:base.team_name.clone(), defender:base.defenders[0].clone(), dungeon:base.dungeon.clone(), active_defenders:Vec::new()}
    }
    pub fn get_index(&self, delver:&Defender) -> Option<usize> {
        let mut result = Option::None;
        for d in 0..self.active_defenders.len() {
            if std::ptr::eq(&self.active_defenders[d], delver) { result = Option::Some(d); break}
        }
        result
    }
    pub fn choose_defender(&self, stat:Stats) -> Entity {
        let mut max = 0.0;        
        let mut best = None;
        for d in &self.active_defenders {
            if d.active {
                if max < d.get_stat(stat) {max = d.get_stat(stat); best = Some(d);}
            }
        }
        let index = self.get_index(best.unwrap()).unwrap();
        return Entity::Defender { index };
        // panic!("All delvers dead.")
    }
}
impl fmt::Display for DefenderTeam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name.truecolor(38,6,215))
    }
}

// -------------------------- Dungeon ---------------------------

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
impl fmt::Display for Dungeon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name.truecolor(38,6,215))
    }
}