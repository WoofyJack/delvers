use std::ops::Add;
use std::fmt;

use crate::sim::{Event,Game,EventQueue};

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
    fn on_enter(&self,  _room:&Room, _queue:&mut EventQueue) {}
    fn attempt_clear(&self,  room:&mut Room, _queue:&mut EventQueue) { room.complete = true;} //Passing both self and room is janky. Need self for dyn to work.
    fn on_exit(&self,  _room:&Room, _queue:&mut EventQueue) {}
}

pub struct Trapped;
impl RoomType for Trapped {
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