use rand::Rng;
use crate::locations::{Coordinate, Room};
use crate::teams::{Dungeon, Delver};
use std::collections::HashMap;

use std::{thread, time};

use colored::Colorize;

pub struct  Sim {
    pub game: Game,
    pub eventqueue:EventQueue,
    pub finished:bool
}
impl Sim {
    pub fn next_frame(&mut self, rng:&mut impl Rng) -> bool {
        while self.game.last_log_message == "" && self.eventqueue.events.len() == 0 {
            self.tick(rng);            
        }
        if self.game.last_log_message != "" {
                self.render();
                // println!("{}", self.game.last_log_message);
                self.game.last_log_message = String::from("");
        }
        else if self.eventqueue.events.len() > 0 {
            self.resolve_last_event()
        };
        self.game.phase != GamePhase::Finished
    }

    pub fn tick(&mut self, rng:&mut impl Rng) -> () {
        let active_delver = &self.game.delvers[self.game.delver_index as usize];
        if !active_delver.active {
            self.game.increment_delver(); return self.tick(rng); // Could go infinite, should add some protections at the start.
        }
        let mut current_room: &mut Room = 
        match self.game.rooms.get_mut(&self.game.delver_position) {
            Some(n) => n,
            None => {self.game.delver_position = Coordinate(0,0); return Sim::tick(self, rng)} //TODO: Replace with some special case room, a la hall of flames.
        };
    
        match self.game.phase {
            GamePhase::NotStarted => {
                self.game.phase = GamePhase::Encounter;
            }
            GamePhase::TurnStart => {
                self.game.phase = GamePhase::Encounter;
                self.game.increment_delver()
    
                //active_delver and current_room not currently valid.
            }
            GamePhase::Encounter => {
                if current_room.complete {self.game.phase = GamePhase::Forge;}
                else {
                    self.game.phase = GamePhase::TurnStart;
                    // Do encounter rolls
                    if roll(rng, active_delver.base.fightiness) > roll(rng, self.game.dungeon.deadliness) {
                        current_room.complete = true;
                        self.game.last_log_message = active_delver.to_string() + " clears room at " + &self.game.delver_position.to_string(); // Move this to an event.
                    } else {
                        self.eventqueue.events.push(Event::Damage {delver_index:self.game.delver_index, amount: 1});
                        self.game.last_log_message = active_delver.to_string() + " fails to clear room."
                    }
            }
            }
            GamePhase::Forge => {
                self.game.phase = GamePhase::Travel;
                //Do forging stuff
            }
            GamePhase::Travel => {
                self.game.phase = GamePhase::TurnStart;
                // Do Travel stuff
                if roll(rng, active_delver.base.speediness) > roll(rng, self.game.dungeon.lengthiness)*10f32 {
                    let position = self.game.delver_position + Coordinate(1,0);
                    self.eventqueue.events.push(Event::Move {delver_index:self.game.delver_index, position});
                    self.game.last_log_message = active_delver.to_string() + " guides the delvers to " + &position.to_string();
                } else {
                    self.eventqueue.events.push(Event::Damage {delver_index:self.game.delver_index, amount: 1});
                    self.game.last_log_message = active_delver.to_string() + " fails to navigate."
                }
            }
            GamePhase::Finished => {}
        }
    }
    pub fn render(&self) {
        let waittime = time::Duration::from_secs(0);
        for p in &self.game.delvers {
            let delvername = if p.active {p.base.name.normal()} else {p.base.name.truecolor(100,100,100)};
            print!("{}: ",delvername);

            let active = "O".red();
            let inactive = "O";
            for i in 0..5 {
                if i+1 <= p.hp {
                    print!("{}", active);
                }
                else{
                    print!("{}", inactive);
                }
            }
            println!();
        }
        println!("{}", self.game.last_log_message);
        thread::sleep(waittime);
        println!()
    }

    pub fn resolve_last_event(&mut self) {
        let event = match self.eventqueue.events.pop() {
            Some(n) => n,
            None => return
        };
        self.resolve_event(event);
    }
    pub fn resolve_event(&mut self, event:Event) {    
        let event: Event = {
            let mut event = event;
            let modifiers = {
                let d = &self.game.delvers[self.game.delver_index as usize];
                &d.modifiers
            };
    
            for m in modifiers {
                event = m.replace_event(event, &self.game, &mut self.eventqueue);
            }
            for m in modifiers {
                m.pre_event(&event, &self.game, &mut self.eventqueue);
    
            }
            event
        };
        match event {
            Event::Damage {delver_index, amount } => {
                self.game.delvers[delver_index as usize].hp -= amount;
                if self.game.delvers[delver_index as usize].hp <= 0 {
                    self.eventqueue.events.push(Event::Death {delver_index});
                }
                self.game.last_log_message = self.game.delvers[delver_index as usize].to_string() + " takes damage, bringing them down to " + &self.game.delvers[delver_index as usize].hp.to_string() + " hp";
            }
            Event::Death { delver_index} => {
                self.game.delvers[delver_index as usize].active = false;
                let alive_delvers = self.game.delvers.iter().any(|r| r.active);
                if !alive_delvers {
                    self.eventqueue.events.push(Event::EndGame);
                }
                self.game.last_log_message = self.game.delvers[delver_index as usize].to_string() + " dies.";
            }
            Event::Move {delver_index, position } => {
                self.game.delver_position = position;
                if self.game.delver_position.0 == 4 { // temporary, need to implement new conditions. 
                    self.eventqueue.events.push(Event::EndGame)
                }
            }
            Event::EndGame => {self.game.phase = GamePhase::Finished; println!("Game Ended")}
            Event::Log { message } => {
                // if game.last_log_message != "" {panic!("Log message dropped!")};
                self.game.last_log_message = message;
                return // don't trigger post_event.
            }
        }
        {
            let modifiers = {
                let d = &self.game.delvers[self.game.delver_index as usize];
                &d.modifiers
            };
            for m in modifiers {
                m.post_event(&event, &self.game, &mut self.eventqueue);
            }
        }
        }
}

pub enum Event {
    Damage {delver_index:u8, amount:i8},
    Move {delver_index:u8, position:Coordinate},
    Death {delver_index:u8},
    EndGame,
    Log {message:String}
}
pub struct EventQueue {
    pub events:Vec<Event>
}
impl EventQueue {
    pub fn new_queue() -> EventQueue {
        EventQueue { events: Vec::new() }
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

    pub delvers:Vec<Delver>,
    delver_index:u8,
    
    rooms: HashMap<Coordinate, Room>,
    delver_position:Coordinate,

    dungeon: Dungeon,

    last_log_message:String,
}


impl Game {
    pub fn new_game(delvers: Vec<Delver>, dungeon:Dungeon, rooms: HashMap<Coordinate,Room>) -> Game {
        Game {phase:GamePhase::NotStarted,
            delver_index:0u8,
            delvers, rooms, dungeon,
            delver_position: Coordinate(0,0),
            last_log_message:String::from("")
        }
    }
    fn increment_delver(&mut self) {
        self.delver_index += 1;
        if self.delver_index >= self.delvers.len().try_into().unwrap()// Converts len() into a u8. Shouldn't be possible to get 256 delvers, but should add in error handling.
            {self.delver_index = 0;}
    }
}


pub fn roll(rng: &mut impl Rng, stat:f32) -> f32 {
    let x:f32 = rng.gen(); //There's a better way to do this
    x * stat
}

impl Game { // replace game with self to make it nicer.
}