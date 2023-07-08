//TODO:
// active_delver should depend on the phase, not just rotate.
// paths ugh
// Interface
// Modifiers
// Could be fun to add some tests

#![allow(dead_code, unused_imports)]
mod teams;
mod sim;
mod locations;
mod modifiers;
mod core_loop;
mod events;

mod combat;

use std::time;
use std::thread;

use std::collections::HashMap;
use crate::events::EventQueue;

use crate::teams::{BaseTeam, DelverTeam, DefenderTeam, Defender, BaseDefender};
use crate::locations::{Coordinate, Room, RoomType};
use crate::sim::{Game, Sim};

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

fn main() {
    colored::control::set_virtual_terminal(true).unwrap();
    // let mut rng = ChaCha8Rng::seed_from_u64(100);
    let mut rng = rand::thread_rng();

    let team1 = BaseTeam::load_from_file("Teams.json", 0);
    let delver_team = DelverTeam::load_team(&team1);
    
    let team2= BaseTeam::load_from_file("Teams.json", 1);
    let defender_team = DefenderTeam::load_team(&team2);
    
    let mut rooms: HashMap<Coordinate, Room> = HashMap::new();
    for i in 0..5 {
        rooms.insert(Coordinate(i,0), Room::new_room(&mut rng));
    }
    let room = Room {complete:false, room_type:RoomType::BossFight};
    rooms.insert(Coordinate(rooms.len() as i8, 0), room);

    let game = Game::new_game(delver_team, defender_team, rooms);
    let mut sim = Sim {game, finished:false, eventqueue:EventQueue::new_queue()};
    // println!("{} are delving into the {}'s dungeon, {}", team1.to_string(), team2.to_string(), team2.dungeon.to_string());

    let waittime = time::Duration::from_secs(2);
    thread::sleep(waittime);
    println!("Play dlungeon!");
    
    loop {
        if !sim.next_frame(&mut rng) {
            break
        }
    }
}