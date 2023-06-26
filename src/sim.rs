use rand::Rng;
use crate::locations::{Coordinate, Room};
use crate::teams::{Dungeon, Delver};
use std::collections::HashMap;

pub struct  Sim {
    pub game: Game,
    pub finished:bool
}
impl Sim {
    pub fn tick(&mut self, rng:&mut impl Rng) -> bool {
        while self.game.last_log_message == "" && self.game.events.len() == 0 {
            Game::tick(&mut self.game, rng);            
        }
        if self.game.last_log_message != "" {
                println!("{}", self.game.last_log_message);
                self.game.last_log_message = String::from("");
        }
        else if self.game.events.len() > 0 {
            Game::resolve_last_event(&mut self.game)
        };
        self.game.phase != GamePhase::Finished
    }
}


#[derive(PartialEq, Eq)]
enum GamePhase {
    NotStarted,
    TurnStart,
    Encounter,
    Forge,
    Travel,
    Finished
}


pub struct Game {
    phase:GamePhase,

    delvers:Vec<Delver>,
    delver_index:u8,
    
    rooms: HashMap<Coordinate, Room>,
    delver_position:Coordinate,

    dungeon: Dungeon,

    events:Vec<Event>,
    last_log_message:String,
}

impl Game {
    pub fn new_game(delvers: Vec<Delver>, dungeon:Dungeon, rooms: HashMap<Coordinate,Room>) -> Game {
        Game {phase:GamePhase::NotStarted,
            delver_index:0u8,
            delvers, rooms, dungeon,
            delver_position: Coordinate(0,0),
            events:Vec::new(),
            last_log_message:String::from("")
        }
    }
    fn increment_delver(&mut self) {
        self.delver_index += 1;
        if self.delver_index >= self.delvers.len().try_into().unwrap()// Converts len() into a u8. Shouldn't be possible to get 256 delvers, but should add in error handling.
            {self.delver_index = 0;}
    }
}
enum Event {
    Damage {delver_index:u8, amount:i8},
    Move {delver_index:u8, position:Coordinate},
    Death {delver_index:u8},
    EndGame
}

fn roll(rng: &mut impl Rng, stat:f32) -> f32 {
    let x:f32 = rng.gen(); //There's a better way to do this
    x * stat
}

impl Game { // replace game with self to make it nicer.
pub fn tick(game:&mut Game, rng:&mut impl Rng) -> () {
    let active_delver = &game.delvers[game.delver_index as usize];
    if !active_delver.active {
        game.increment_delver(); return Game::tick(game, rng); // Could go infinite, should add some protections at the start.
    }
    let mut current_room: &mut Room = 
    match game.rooms.get_mut(&game.delver_position) {
        Some(n) => n,
        None => {game.delver_position = Coordinate(0,0); return Game::tick(game, rng)} //TODO: Replace with some special case room, a la hall of flames.
    };

    match game.phase {
        GamePhase::NotStarted => {
            game.phase = GamePhase::Encounter;
        }
        GamePhase::TurnStart => {
            game.phase = GamePhase::Encounter;
            game.increment_delver()

            //active_delver and current_room not currently valid.
        }
        GamePhase::Encounter => {
            if current_room.complete {game.phase = GamePhase::Forge;}
            else {
                game.phase = GamePhase::TurnStart;
                // Do encounter rolls
                if roll(rng, active_delver.base.fightiness) > roll(rng, game.dungeon.deadliness) {
                    current_room.complete = true;
                    game.last_log_message = active_delver.to_string() + " clears room at " + &game.delver_position.to_string(); // Move this to an event.
                } else {
                    game.events.push(Event::Damage {delver_index:game.delver_index, amount: 1});
                    game.last_log_message = active_delver.to_string() + " fails to clear room."
                }
        }
        }
        GamePhase::Forge => {
            game.phase = GamePhase::Travel;
            //Do forging stuff
        }
        GamePhase::Travel => {
            game.phase = GamePhase::TurnStart;
            // Do Travel stuff
            if roll(rng, active_delver.base.speediness) > roll(rng, game.dungeon.lengthiness) {
                let position = game.delver_position + Coordinate(1,0);
                game.events.push(Event::Move {delver_index:game.delver_index, position});
                game.last_log_message = active_delver.to_string() + " guides the delvers to " + &position.to_string();
            } else {
                game.events.push(Event::Damage {delver_index:game.delver_index, amount: 1});
                game.last_log_message = active_delver.to_string() + " fails to navigate."
            }
        }
        GamePhase::Finished => {}
    }
}

pub fn resolve_last_event(game:&mut Game) {
    let event = match game.events.pop() {
        Some(n) => n,
        None => return
    };
    match event {
        Event::Damage {delver_index, amount } => {
            game.delvers[delver_index as usize].hp -= amount;
            if game.delvers[delver_index as usize].hp <= 0 {
                game.events.push(Event::Death {delver_index});
            }
            game.last_log_message = game.delvers[delver_index as usize].to_string() + " takes damage, bringing them down to " + &game.delvers[delver_index as usize].hp.to_string() + " hp";
        }
        Event::Death { delver_index} => {
            game.delvers[delver_index as usize].active = false;
            let alive_delvers = game.delvers.iter().any(|r| r.active);
            if !alive_delvers {
                game.events.push(Event::EndGame);
            }
            game.last_log_message = game.delvers[delver_index as usize].to_string() + " dies.";
        }
        Event::Move {delver_index, position } => {
            game.delver_position = position;
            if game.delver_position.0 == 4 { // temporary, need to implement new conditions. 
                game.events.push(Event::EndGame)
            }
        }
        Event::EndGame => {game.phase = GamePhase::Finished; println!("Game Ended")}
    }
}
}