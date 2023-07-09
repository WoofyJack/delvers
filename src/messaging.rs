use crate::events::Entity;
use crate::locations::Coordinate;
use crate::sim::Game;
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Delving,
    Attack (Entity, Entity, u8), // Source, Target, Amount
    Heal (Entity, Entity, u8), // Source, Target, Amount
    BeginNavigation (Entity), //active_delver
    Travel (Entity, Coordinate), // active_delver, destination
    FailedNavigation (Entity),
    Custom (String),
    Death (Entity),
    None
}

impl Message {
    pub fn to_string(&self, game:&Game) -> String {
        match self {
            Message::Attack(attacker, reciever, amount) => attacker.to_string(game) + " injures " + &reciever.to_string(game),
            Message::Delving => String::from("The ") + &game.delverteam.to_string() + " are delving into the " + &game.defenderteam.to_string() + "'s dungeon, " + &game.defenderteam.dungeon.to_string(),
            Message::Heal (healer, reciever, amount) => healer.to_string(game) + " heals " + &reciever.to_string(game),
            Message::BeginNavigation(navigator) => navigator.to_string(game) + " begins trying to navigate to the next room.",
            Message::Travel(navigator, destination) => navigator.to_string(game) + " guides the delvers to " + &destination.to_string(),
            Message::FailedNavigation(navigator) => navigator.to_string(game) + " hurts themselves while navigating.",
            Message::Custom(message) => message.clone(),
            Message::Death(dier) => dier.to_string(game) + " dies.",
            Message::None => game.last_log_message.clone()
        }
    }
}