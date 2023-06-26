use std::ops::Add;
use std::fmt;

pub struct Room {
    pub complete:bool
}
impl Room {
    pub fn new_room() -> Room {
        Room {complete: false}
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