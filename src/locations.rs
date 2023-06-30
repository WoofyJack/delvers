use std::ops::Add;
use std::fmt;

use crate::{sim::{Event,Game,EventQueue}, teams::{DelverStats, Delver}, modifiers::Outcomes};

pub struct Room {
    pub complete:bool,
    pub room_type:Box<dyn RoomType>
}
impl Room {
    pub fn new_room() -> Room {
        Room {complete: false, room_type:Box::new(Trapped)}
    }
}
pub trait RoomType {
    fn on_enter(&self,  _room_position:Coordinate, _queue:&mut EventQueue) {}
    fn attempt_clear(&self,  room_position:Coordinate, _delver_index:u8, queue:&mut EventQueue) {queue.events.push(Event::ClearRoom {room_position});}
    fn on_exit(&self,  _room_position:Coordinate, _queue:&mut EventQueue) {}
    fn base_stat(&self) -> DelverStats;
}

pub struct Trapped;
impl RoomType for Trapped {
    fn attempt_clear(&self,  room_position:Coordinate, delver_index:u8, _queue:&mut EventQueue) {
        let outcomes = Outcomes {
            success: Event::comment_event(Event::ClearRoom {room_position}, "Trapped room cleared".to_string()),
            fail: Event::comment_event(Event::Damage { delver_index, amount: 1 }, "Trapped room triggered".to_string())};
        _queue.events.push(Event::Roll {difficulty: 0.25, stat: self.base_stat(), outcomes })
    }
    fn base_stat(&self) -> DelverStats {
        DelverStats::Fightiness
    }
}

#[derive(Debug,Copy,Clone,Eq, Hash, PartialEq)]
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