use std::ops::Add;
use std::fmt;

use crate::{sim::{Event,EventType,Game,EventQueue}, teams::{DelverStats}, modifiers::Outcomes};

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
    fn on_enter(&self, _game:&Game,  _room_position:Coordinate, _queue:&mut EventQueue) {}
    fn attempt_clear(&self, _game:&Game,  room_position:Coordinate, _delver_index:usize, queue:&mut EventQueue) {queue.events.push((EventType::ClearRoom {room_position}).no_target());}
    fn on_exit(&self, _game:&Game,  _room_position:Coordinate, _queue:&mut EventQueue) {}
    fn base_stat(&self) -> DelverStats;
}

pub struct Trapped;
impl RoomType for Trapped {
    fn attempt_clear(&self, game:&Game,  room_position:Coordinate, delver_index:usize, _queue:&mut EventQueue) {
        let delvername = &game.delverteam.delvers[delver_index].base.name;
        let outcomes = Outcomes {
            success: Event::comment_event((EventType::ClearRoom {room_position}).no_target(),delvername.clone() + " clears a trapped room"),
            fail: Event::comment_event((EventType::Damage {amount: 1 }).target(delver_index), delvername.clone() + " triggers a trapped room, hurting themselves")};
        _queue.events.push((EventType::Roll {difficulty: 0.25, stat: self.base_stat(), outcomes}).no_target())
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