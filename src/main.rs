#![allow(dead_code)]
use rand::Rng;

use std::collections::HashMap;

#[derive(Eq, Hash, PartialEq)]
struct Coordinate(i8, i8);

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


enum GamePhase {
    NotStarted,
    TurnStart,
    Encounter,
    Forge,
    Travel,
    Finished
}

struct Room {
    coord:Coordinate,
    complete:bool
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

    roll(&mut rng);

    let delvers = Vec::new();
    let rooms = HashMap::new();
    let mut game_state = Game::new_game(delvers, rooms);
    
    let c = Delver::new_delver(String::from("Rogue"));

    game_state.delvers.push(c);

    let c = Delver::new_delver(String::from("Fighter"));
    game_state.delvers.push(c);

}

fn roll(rng: &mut impl Rng) {
    let result:f32 = rng.gen();
    println!("{}",result);
}

fn tick(game:&mut Game) {
    let active_delver = &game.delvers[game.delver_index as usize];
    let current_room = &game.rooms.get(&game.delver_position).expect("Handle Error later");

    match game.phase {
        GamePhase::NotStarted => {
            game.phase = GamePhase::Encounter;
        }
        GamePhase::TurnStart => {
            game.phase = GamePhase::Encounter;
            game.delver_index += 1;
            if game.delver_index >= game.delvers.len().try_into().expect("Impossibly many delvers. Handle later.") {game.delver_index = 0;}
            //active_delver and current_room not currently valid.
        }
        GamePhase::Encounter => {
            if current_room.complete {game.phase = GamePhase::Forge;}
            else {
                game.phase = GamePhase::TurnStart;

            }
        }
        GamePhase::Forge => {}
        GamePhase::Travel => {}
        GamePhase::Finished => {}
    }
}