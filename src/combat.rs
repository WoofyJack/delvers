use serde::{Serialize, Deserialize};
use crate::base_entities::{BaseDefender};
use crate::entities::{Defender, Stats};
impl Defender {
    pub fn create_monster(name:String, difficulty:f32, hp:i8, maxhp:i8) -> Defender{
        let mut monster_base = BaseDefender::new_delver(name);
        monster_base.fightiness = difficulty;
        let mut defender = monster_base.to_game_defender();
        defender.hp = hp;
        defender.maxhp = maxhp;
        defender
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Monster {
    pub name:String,
    pub difficulty:f32,
    pub hp:i8,
    pub maxhp:i8
}
impl Monster {
    pub fn to_game_defender(self) -> Defender {
        Defender::create_monster(self.name, self.difficulty, self.hp, self.maxhp)
    }
}