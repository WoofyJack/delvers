//TODO:
// active_delver should depend on the phase, not just rotate.
// paths ugh
// Interface
// Modifiers
// Could be fun to add some tests

#![allow(dead_code, unused_imports)]
mod teams;
mod entities;
mod base_entities;
mod sim;
mod room_types;
mod modifiers;
mod core_loop;
mod events;

mod messaging;
mod combat;

use std::fs;
use std::io::Write;
use std::time;
use std::thread;
use std::fs::File;

use std::collections::HashMap;
use crate::events::EventQueue;

use crate::base_entities::{BaseTeam, BaseDefender};
use crate::entities::{DelverTeam, DefenderTeam};
use crate::room_types::{Coordinate, RoomType};
use crate::sim::{Game, Sim};

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use rand_pcg::Pcg32;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize,Deserialize)]
struct RngSaver {
    rng_seed:[u8;32],
    rng_stream:u64,
    rng_word:u128,
}
impl RngSaver {
    fn save_rng(rng:&ChaCha8Rng) -> RngSaver {
        let rng_seed = rng.get_seed();
        let rng_stream = rng.get_stream();
        let rng_word = rng.get_word_pos();
        RngSaver { rng_seed, rng_stream, rng_word}
    }
    fn load_rng(&self) -> ChaCha8Rng {
        let mut rng = ChaCha8Rng::from_seed(self.rng_seed);
        rng.set_stream(self.rng_stream);
        rng.set_word_pos(self.rng_word);
        rng
    }
}

fn main() {
    colored::control::set_virtual_terminal(true).unwrap();


    let mut rng = rand::thread_rng();

    let mut rng = ChaCha8Rng::seed_from_u64(rng.gen());

    let team1 = BaseTeam::load_from_file("Teams.json", 0);
    let delver_team = DelverTeam::load_team(&team1);
    
    let team2 = BaseTeam::load_from_file("Teams.json", 1);
    let defender_team = DefenderTeam::load_team(&team2);

    let game = Game::new_game(delver_team, defender_team);
    let mut sim = Sim {game, finished:false, eventqueue:EventQueue::new_queue()};
    // println!("{} are delving into the {}'s dungeon, {}", team1.to_string(), team2.to_string(), team2.dungeon.to_string());

    println!("Play dlungeon!");
    let waittime = time::Duration::from_secs(2);
    thread::sleep(waittime);

    loop {
        if !sim.next_frame(&mut rng) {
            break
        }

    }

}

fn load() -> (ChaCha8Rng, Sim) {
    let file = std::fs::read_to_string("rngsave.json").unwrap();
    let rngsave:RngSaver = serde_json::from_str(&file).unwrap();
    rngsave.load_rng();

    let file = std::fs::read_to_string("GameSave.json").unwrap();
    let sim = serde_json::from_str(&file).unwrap();

    (rngsave.load_rng(), sim) 
}

fn save(rng: ChaCha8Rng, sim:Sim) {
    let game_save = serde_json::to_string_pretty(&sim).unwrap();
    let mut file = File::create("GameSave.json").unwrap();
    write!(file, "{}", game_save).unwrap();

    let rngsaver = RngSaver::save_rng(&rng);
    let rng_save = serde_json::to_string(&rngsaver).unwrap();
    let mut file = File::create("rngsave.json").unwrap();
    write!(file, "{}", rng_save).unwrap();
}