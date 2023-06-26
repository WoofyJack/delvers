//TODO:
// active_delver should depend on the phase, not just rotate.
// paths ugh
// Interface
// Modifiers
// Could be fun to add some tests

#![allow(dead_code)]
mod teams;
mod sim;
mod locations;


use std::collections::HashMap;

use crate::teams::{Delver, Dungeon, Team};
use crate::locations::{Coordinate, Room};
use crate::sim::{Game, Sim};


fn main() {
    let mut rng = rand::thread_rng();
    let team = Team::load_from_file("Teams.json", 0);

    let mut delvers = Vec::new();
    for c in team.delvers {
        let c:Delver = Delver::load_delver(c);
        delvers.push(c);
    }
    let mut rooms: HashMap<Coordinate, Room> = HashMap::new();
    for i in 0..5 {
        rooms.insert(Coordinate(i,0), Room::new_room());
    }
    let dungeon = Dungeon::new_dungeon(String::from("The Dungeon"));
    let game = Game::new_game(delvers, dungeon, rooms);

    let mut sim = Sim {game, finished:false};

    loop {
        if !sim.tick(&mut rng) {
            break
        }

    }
}