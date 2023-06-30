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
mod modifiers;


use std::collections::HashMap;

use sim::EventQueue;

use crate::teams::{BaseTeam, GameTeam};
use crate::locations::{Coordinate, Room};
use crate::sim::{Game, Sim};

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

fn main() {
    colored::control::set_virtual_terminal(true).unwrap();
    // let mut rng = ChaCha8Rng::seed_from_u64(100);
    let mut rng = rand::thread_rng();

    let team = BaseTeam::load_from_file("Teams.json", 0);
    let game_team = GameTeam::load_team(&team);

    let mut rooms: HashMap<Coordinate, Room> = HashMap::new();
    for i in 0..5 {
        rooms.insert(Coordinate(i,0), Room::new_room());
    }
    let dungeon = team.dungeon.clone();
    let game = Game::new_game(game_team, dungeon, rooms);
    let mut sim = Sim {game, finished:false, eventqueue:EventQueue::new_queue()};
    println!("Play dlungeon!");
    loop {
        if !sim.next_frame(&mut rng) {
            break
        }
    }
}