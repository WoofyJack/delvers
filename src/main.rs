#![allow(dead_code)]
use rand::Rng;
use std::ops::Add;
use std::fmt;
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
    exploriness: f64,
    fightiness: f64,
    hp:i8
}

impl Delver {
    fn new_delver(name: String) -> Delver{
        Delver {name, exploriness:0.5, fightiness:0.5, hp:5}
    }
}
impl fmt::Display for Delver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}


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
    last_log_message:String,
}
impl Game {
    fn new_game(delvers: Vec<Delver>, rooms: HashMap<Coordinate,Room>) -> Game {
        Game {phase:GamePhase::NotStarted,
            delver_index:0u8,
            delvers, rooms,
            delver_position: Coordinate(0,0),
            last_log_message:String::from("")
        }
    }
}
fn main() {
    let mut rng = rand::thread_rng();

    let delvers = Vec::new();
    let rooms: HashMap<Coordinate, Room> = HashMap::new();
    let mut game_state = Game::new_game(delvers, rooms);
    
    let c = Delver::new_delver(String::from("Rogue"));
    game_state.delvers.push(c);
    let c = Delver::new_delver(String::from("Fighter"));
    game_state.delvers.push(c);

    game_state.rooms.insert(Coordinate(0,0), Room::new_room());
    game_state.rooms.insert(Coordinate(1,0), Room::new_room());
    game_state.rooms.insert(Coordinate(2,0), Room::new_room());

    for _ in 0..10 {
    tick(&mut game_state, &mut rng);
    }
}

fn roll(rng: &mut impl Rng) -> f32 {
    rng.gen()
}

fn tick(game:&mut Game, rng:&mut impl Rng) -> () {
    let active_delver = &game.delvers[game.delver_index as usize];
    let mut current_room: &mut Room = game.rooms.get_mut(&game.delver_position).expect("Handle Error later");

    match game.phase {
        GamePhase::NotStarted => {
            game.phase = GamePhase::Encounter;
        }
        GamePhase::TurnStart => {
            game.phase = GamePhase::Encounter;
            game.delver_index += 1;
            if game.delver_index >= game.delvers.len().try_into()// Converts len() into a u8. Shouldn't be possible to get 256 delvers, but should add in error handling.
            .expect("Impossibly many delvers. Handle later.") {game.delver_index = 0;}
            //active_delver and current_room not currently valid.
        }
        GamePhase::Encounter => {
            if current_room.complete {game.phase = GamePhase::Forge;}
            else {
                game.phase = GamePhase::TurnStart;
                // Do encounter rolls
                current_room.complete = true;
                println!("{}", active_delver);
            }
        }
        GamePhase::Forge => {
            game.phase = GamePhase::Travel;
            //Do forging stuff
        }
        GamePhase::Travel => {
            game.phase = GamePhase::TurnStart;
            // Do Travel stuff
            game.delver_position = game.delver_position + Coordinate(1,0);
            println!("{}",game.delver_position);
        }
        GamePhase::Finished => {}
    }
}