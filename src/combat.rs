use crate::teams::{BaseDefender, Defender, Stats};
impl Defender {
    pub fn create_monster(name:String, difficulty:f32) -> Defender{
        let mut monster_base = BaseDefender::new_delver(name);
        monster_base.fightiness = difficulty;
        monster_base.to_game_defender()
    }
}