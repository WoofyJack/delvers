use serde::{Serialize, Deserialize};
use core::panic;
use std::ops::Add;
use std::fmt;
use rand::Rng;

use crate::{sim::Game, events::{Event, EventType,EventQueue, Outcomes}, messaging::Message, combat::Monster};
use crate::entities::{Stats, Defender, Entity};


#[derive(Serialize, Deserialize)]
pub enum RoomType {
    Trapped,
    Arcane,
    BossFight,
    Fight {monsters:Vec<Monster>, partyname:String}
}
impl RoomType {
    fn on_enter(&self, _game:&Game,  room:Entity, _queue:&mut EventQueue) {}
    pub fn attempt_clear(&self, game:&Game,  room:Entity, delver:Entity, queue:&mut EventQueue) {
        // queue.events.push(EventType::ClearRoom.target(delver,room));
        match self {
            RoomType::Arcane => {arcane_ward::attempt_clear(game, room, delver, queue)}
            RoomType::Trapped => {trapped::attempt_clear(game, room, delver, queue)}
            RoomType::BossFight => {bossfight::attempt_clear(game, room, delver, queue)}
            RoomType::Fight {monsters, partyname} => {fight::attempt_clear(game, room, delver, queue, monsters, partyname.clone())}
        }
    }
    fn on_exit(&self, _game:&Game,  room:Entity, _queue:&mut EventQueue) {}
    pub fn base_stat(&self) -> Stats {
        match self {
            RoomType::Arcane => arcane_ward::base_stat(),
            RoomType::Trapped => trapped::base_stat(),
            RoomType::BossFight => bossfight::base_stat(),
            RoomType::Fight { .. } => fight::base_stat()
        }
    }
}

mod trapped {
    use crate::room_types::*;
    pub fn attempt_clear(game:&Game,  room:Entity, delver:Entity, queue:&mut EventQueue) {
        let trigger_index = game.rand_target;
        let trigger_delver = Entity::Delver { index: trigger_index};


        let message = Message::Custom(delver.to_string(game) + " disarms the traps");
        let success = vec![Event{ event_type:EventType::ClearRoom, source:delver, target:room, message}];

        let message = Message::Custom(trigger_delver.to_string(game) + " triggers a trap room, hurting themselves");
        let fail = vec![Event{event_type:EventType::Damage (1), source:room, target:trigger_delver, message}];
        
        let outcomes = Outcomes {success, fail};

        let message = Message::Custom(delver.to_string(game) + " attempts to disarm a trap.");
        let event = Event::type_and_message(EventType::Roll { difficulty: 0.8, stat: base_stat(), outcomes}, message);

        queue.events.push(event);
    }
    pub fn base_stat() -> Stats {
        Stats::Fightiness
    }
}

mod bossfight{
    use crate::room_types::*;
    pub fn attempt_clear(_game:&Game,  room:Entity, delver:Entity, queue:&mut EventQueue) {
        let event = Event{event_type:EventType::StartBossFight, source:room, target:Entity::None, message:Message::None};
        queue.events.push(event);
        let event = Event{event_type:EventType::ClearRoom, source:delver, target: room, message:Message::None};
        queue.events.push(event);
    }
    pub fn base_stat() -> Stats {
        Stats::Fightiness
    }
}
mod fight {
    use crate::room_types::*;
    pub fn attempt_clear(game:&Game,  room:Entity, delver:Entity, queue:&mut EventQueue, monsters:&Vec<Monster>, partyname:String) {
        let mut monsters = (*monsters).clone();
        
        let event = Event{event_type:EventType::ClearRoom, source:delver, target: room, message:Message::None};
        queue.events.push(event);
        let final_monster = monsters.remove(0);
        let defender = final_monster.to_game_defender();
        let message = Message::Encounters(partyname);
        let event = Event{event_type:EventType::SpawnDefender(defender), source:room, target: Entity::None, message};
        queue.events.push(event);
        for m in monsters {
            let defender = m.to_game_defender();
            let event = Event{event_type:EventType::SpawnDefender(defender), source:room, target: Entity::None, message:Message::None};
            queue.events.push(event);
        }

    }
    pub fn base_stat() -> Stats {
        Stats::Fightiness
    }
}
mod arcane_ward {
    use crate::room_types::*;
    pub fn attempt_clear(game:&Game,  room:Entity, delver:Entity, queue:&mut EventQueue) {
        let trigger_index = game.rand_target;
        let trigger_delver = Entity::Delver { index: trigger_index};

        let message = Message::Custom(delver.to_string(game) + " clears the arcane ward");
        let success = vec![Event{ event_type:EventType::ClearRoom, source:delver, target:room, message}];

        let message = Message::Custom(trigger_delver.to_string(game) + " is exploded by a magical wrad.");
        let fail = vec![Event{event_type:EventType::Damage (2), source:room, target:trigger_delver, message}];
        
        let outcomes = Outcomes {success, fail};

        let message = Message::Custom(delver.to_string(game) + " attempts to clear an arcane ward.");
        let event = Event::type_and_message(EventType::Roll { difficulty: 0.8, stat: base_stat(), outcomes}, message);

        queue.events.push(event);
    }
    pub fn base_stat() -> Stats {
        Stats::Magiciness
    }
}

#[derive(Debug,Copy,Clone,Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Coordinate(pub i8, pub i8);
impl Add for Coordinate {
    type Output = Self;
    fn add(self, other:Self) -> Self{
        Coordinate(self.0+other.0, self.1+other.1)
    }
}
impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{},{}]", self.0, self.1)
    }
}