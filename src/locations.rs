use serde::Serialize;
use core::panic;
use std::ops::Add;
use std::fmt;
use rand::Rng;

use crate::{sim::Game,
    events::{EventType,EventQueue, Scene, Entity}, 
    teams::Stats};

pub struct Room {
    pub complete:bool,
    pub room_type:Box<dyn RoomType>
}
impl Room {
    pub fn new_room(rng: &mut impl Rng) -> Room {
        let room_type:Box<dyn RoomType> = match rng.gen_range(0..2) {
            0 => Box::new(Trapped),
            1 => Box::new(ArcaneWard),
            _ => panic!("Fix the rng range"),
        };

        Room {complete: false, room_type}
    }
}
pub trait RoomType {
    fn on_enter(&self, _game:&Game,  room:Entity, _queue:&mut EventQueue) {}
    fn attempt_clear(&self, _game:&Game,  room:Entity, delver:Entity, queue:&mut EventQueue) {
        queue.events.push(EventType::ClearRoom.target(delver,room));
    }
    fn on_exit(&self, _game:&Game,  room:Entity, _queue:&mut EventQueue) {}
    fn base_stat(&self) -> Stats;
}

pub struct Trapped;
impl RoomType for Trapped {
    fn attempt_clear(&self, game:&Game,  room:Entity, delver:Entity, queue:&mut EventQueue) {
        let trigger_index = game.rand_target;
        let trigger_delver = Entity::Delver { index: trigger_index};


        let begin = delver.to_string(game) + " attempts to disarm a trap.";
        let success = (EventType::ClearRoom.target(delver,room),
                                        delver.to_string(game) + " disarms the traps");
        let fail = (EventType::Damage {amount: 1 }.target(room,trigger_delver),
            trigger_delver.to_string(game) + " triggers a trap room, hurting themselves");
        let scene = Box::new(Scene {begin, difficulty:0.8, stat:self.base_stat(), success, fail});

        queue.events.push(EventType::Scene { scene }.no_target(room));
    }
    fn base_stat(&self) -> Stats {
        Stats::Fightiness
    }
}
pub struct BossFight;
impl RoomType for BossFight {
    fn attempt_clear(&self, _game:&Game,  room:Entity, delver:Entity, queue:&mut EventQueue) {
        let event = EventType::StartBossFight.no_target(room);
        queue.events.push(event);
        let event = EventType::ClearRoom.target(delver, room);
        queue.events.push(event);
    }
    fn base_stat(&self) -> Stats {
        Stats::Fightiness
    }
}
pub struct ArcaneWard;
impl RoomType for ArcaneWard {
    fn attempt_clear(&self, game:&Game,  room:Entity, delver:Entity, queue:&mut EventQueue) {
        let trigger_index = game.rand_target;
        let trigger_delver = Entity::Delver { index: trigger_index};


        let begin = delver.to_string(game) + " attempts to clear an arcane ward.";
        let success = (EventType::ClearRoom.target(delver, room)
                                                        ,delver.to_string(game) + " clears the arcane ward");
        let fail = (EventType::Damage {amount: 2 }.target(room, trigger_delver),
                                                        trigger_delver.to_string(game) + " is exploded by a magical wrad.");
        let scene = Box::new(Scene {begin, difficulty:0.6, stat:self.base_stat(), success, fail});

        queue.events.push(EventType::Scene { scene }.no_target(room));
    }
    fn base_stat(&self) -> Stats {
        Stats::Magiciness
    }
}

#[derive(Debug,Copy,Clone,Eq, Hash, PartialEq, Serialize)]
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