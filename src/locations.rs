use serde::{Serialize, Deserialize};
use core::panic;
use std::ops::Add;
use std::fmt;
use rand::Rng;

use crate::{sim::Game,
    events::{EventType,EventQueue, Scene, Entity}, 
    teams::Stats};

#[derive(Serialize, Deserialize)]
pub struct Room {
    pub complete:bool,
    pub room_type:RoomType
}
impl Room {
    pub fn new_room(rng: &mut impl Rng) -> Room {
        let room_type = match rng.gen_range(0..2) {
            0 => RoomType::Arcane,
            1 => RoomType::Trapped,
            _ => panic!("Fix the rng range"),
        };

        Room {complete: false, room_type}
    }
}
#[derive(Serialize, Deserialize)]
pub enum RoomType {
    Trapped,
    Arcane,
    BossFight
}
impl RoomType {
    fn on_enter(&self, _game:&Game,  room:Entity, _queue:&mut EventQueue) {}
    pub fn attempt_clear(&self, game:&Game,  room:Entity, delver:Entity, queue:&mut EventQueue) {
        // queue.events.push(EventType::ClearRoom.target(delver,room));
        match self {
            RoomType::Arcane => {arcane_ward::attempt_clear(game, room, delver, queue)}
            RoomType::Trapped => {trapped::attempt_clear(game, room, delver, queue)}
            RoomType::BossFight => {bossfight::attempt_clear(game, room, delver, queue)}
        }
    }
    fn on_exit(&self, _game:&Game,  room:Entity, _queue:&mut EventQueue) {}
    pub fn base_stat(&self) -> Stats {
        match self {
            RoomType::Arcane => arcane_ward::base_stat(),
            RoomType::Trapped => trapped::base_stat(),
            RoomType::BossFight => bossfight::base_stat()
        }
    }
}

mod trapped {
    use crate::locations::*;
    pub fn attempt_clear(game:&Game,  room:Entity, delver:Entity, queue:&mut EventQueue) {
        let trigger_index = game.rand_target;
        let trigger_delver = Entity::Delver { index: trigger_index};


        let begin = delver.to_string(game) + " attempts to disarm a trap.";
        let success = (EventType::ClearRoom.target(delver,room),
                                        delver.to_string(game) + " disarms the traps");
        let fail = (EventType::Damage {amount: 1 }.target(room,trigger_delver),
            trigger_delver.to_string(game) + " triggers a trap room, hurting themselves");
        let scene = Box::new(Scene {begin, difficulty:0.8, stat:base_stat(), success, fail});

        queue.events.push(EventType::Scene { scene }.no_target(room));
    }
    pub fn base_stat() -> Stats {
        Stats::Fightiness
    }
}

mod bossfight{
    use crate::locations::*;
    pub fn attempt_clear(_game:&Game,  room:Entity, delver:Entity, queue:&mut EventQueue) {
        let event = EventType::StartBossFight.no_target(room);
        queue.events.push(event);
        let event = EventType::ClearRoom.target(delver, room);
        queue.events.push(event);
    }
    pub fn base_stat() -> Stats {
        Stats::Fightiness
    }
}
mod arcane_ward {
    use crate::locations::*;
    pub fn attempt_clear(game:&Game,  room:Entity, delver:Entity, queue:&mut EventQueue) {
        let trigger_index = game.rand_target;
        let trigger_delver = Entity::Delver { index: trigger_index};


        let begin = delver.to_string(game) + " attempts to clear an arcane ward.";
        let success = (EventType::ClearRoom.target(delver, room)
                                                        ,delver.to_string(game) + " clears the arcane ward");
        let fail = (EventType::Damage {amount: 2 }.target(room, trigger_delver),
                                                        trigger_delver.to_string(game) + " is exploded by a magical wrad.");
        let scene = Box::new(Scene {begin, difficulty:0.6, stat:base_stat(), success, fail});

        queue.events.push(EventType::Scene { scene }.no_target(room));
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