//TODO:
// active_delver should depend on the phase, not just rotate.
// paths ugh
// Interface
// Modifiers

#![allow(dead_code)]
use rand::Rng;
use std::ops::Add;
use std::fmt::{self, Debug};
use std::collections::HashMap;

#[derive(Debug,Copy,Clone,Eq, Hash, PartialEq)]
struct Coordinate(i8, i8);
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

#[derive(Debug)]
struct Delver {
    name: String,
    exploriness: f32,
    fightiness: f32,
    speediness: f32,
    hp:i8,

    alive:bool // might be wise to do as an enum to avoid accidentally using dead delvers.
}

impl Delver {
    fn new_delver(name: String) -> Delver{
        Delver {name, exploriness:0.5, fightiness:0.5, speediness:0.5, hp:3, alive:true}
    }
}
impl fmt::Display for Delver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

struct Dungeon {
    name: String,
    twistiness: f32,
    deadliness: f32,
    lengthiness: f32
}
impl Dungeon {
    fn new_dungeon(name: String) -> Dungeon{
        Dungeon {name, twistiness:0.5, deadliness:0.5, lengthiness:0.5}
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

struct Room {
    complete:bool
}
impl Room {
    fn new_room() -> Room {
        Room {complete: false}
    }
}

struct Game {
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
    fn new_game(delvers: Vec<Delver>, dungeon:Dungeon, rooms: HashMap<Coordinate,Room>) -> Game {
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
        if self.delver_index >= self.delvers.len().try_into()// Converts len() into a u8. Shouldn't be possible to get 256 delvers, but should add in error handling.
        .expect("Impossibly many delvers. Handle later.") {self.delver_index = 0;}
    }
}
enum Event {
    Damage {delver_index:u8, amount:i8},
    Move {delver_index:u8, position:Coordinate},
    Death {delver_index:u8},
    EndGame
}

fn main() {
    let mut rng = rand::thread_rng();

    let delvers = Vec::new();
    let rooms: HashMap<Coordinate, Room> = HashMap::new();
    let dungeon = Dungeon::new_dungeon(String::from("The Dungeon"));
    let mut game_state = Game::new_game(delvers, dungeon, rooms);
    
    let c = Delver::new_delver(String::from("Rogue"));
    game_state.delvers.push(c);
    let c = Delver::new_delver(String::from("Fighter"));
    game_state.delvers.push(c);

    for i in 0..5 {
        game_state.rooms.insert(Coordinate(i,0), Room::new_room());
    }
    while game_state.phase != GamePhase::Finished{
        while game_state.last_log_message == "" && game_state.events.len() == 0 {
            tick(&mut game_state, &mut rng);            
        }
        if game_state.last_log_message != "" {
        println!("{}", game_state.last_log_message);
        game_state.last_log_message = String::from("");
        }
        else if game_state.events.len() > 0 {
            resolve_last_event(&mut game_state)
        }
    }
}

fn roll(rng: &mut impl Rng, stat:f32) -> f32 {
    let x:f32 = rng.gen(); //There's a better way to do this
    x * stat
}

fn tick(game:&mut Game, rng:&mut impl Rng) -> () {
    let active_delver = &game.delvers[game.delver_index as usize];
    if !active_delver.alive {game.increment_delver(); return tick(game, rng);}

    let mut current_room: &mut Room = match game.rooms.get_mut(&game.delver_position) {
        Some(n) => n,
        None => {game.delver_position = Coordinate(0,0); return tick(game, rng);} //Replace with some special case room, a la hall of flames.
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
                if roll(rng, active_delver.fightiness) > roll(rng, game.dungeon.deadliness) {
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
            if roll(rng, active_delver.speediness) > roll(rng, game.dungeon.lengthiness) {
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

fn resolve_last_event(game:&mut Game) {
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
            game.delvers[delver_index as usize].alive = false;
            let alive_delvers = game.delvers.iter().any(|r| r.alive);
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