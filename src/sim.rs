use rand::Rng;
use crate::locations::{Coordinate, Room};
use crate::modifiers::{Outcomes, ReplaceOutcomes};
use crate::teams::{Dungeon, DelverStats, GameTeam};
use std::collections::HashMap;

use std::{thread, time};

use colored::Colorize;

pub struct  Sim {
    pub game: Game,
    pub eventqueue:EventQueue,
    pub finished:bool
}

pub struct Event {
    pub target_index:Option<usize>,
    pub event_type:EventType
}
impl Event {
    // pub fn no_target(event_type:EventType) -> Event {
    //     Event {target_index:Option::None, event_type}
    // }
    // pub fn has_target (target:usize, event_type:EventType) -> Event {
    //     Event {target_index:Option::Some(target), event_type}
    // }
    pub fn comment_event(event:Event, message:String) -> Vec<Event> {
        vec![event, (EventType::Log{message}).no_target()]
    }
}

pub enum EventType {
    Damage {amount:i8},
    Heal {amount:i8},
    Move {position:Coordinate},
    Death,
    EndGame,
    Log {message:String},
    // Roll {difficulty:f32, stat:} //Figure this out later.
    Roll {difficulty:f32, stat:DelverStats, outcomes:Outcomes},
    ClearRoom {room_position:Coordinate},
    Cancelled
}
impl EventType {
    pub fn no_target(self) -> Event {
        Event {event_type:self, target_index:Option::None}
    }
    pub fn target(self, index:usize) -> Event {
        Event {event_type:self, target_index:Option::Some(index)}
    }
}
pub struct EventQueue {
    pub events:Vec<Event> //Probably doesnt need to be a struct. Fix later.
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

    pub delverteam:GameTeam,
    
    rooms: HashMap<Coordinate, Room>,
    delver_position:Coordinate,

    dungeon: Dungeon,

    last_log_message:String,
}


impl Game {
    pub fn new_game(delverteam:GameTeam, dungeon:Dungeon, rooms: HashMap<Coordinate,Room>) -> Game {
        Game {phase:GamePhase::NotStarted,
            delverteam, rooms, dungeon,
            delver_position: Coordinate(0,0),
            last_log_message:String::from("")
        }
    }
    // fn increment_delver(&mut self) {
    //     self.delver_index += 1;
    //     if self.delver_index >= self.delvers.len().try_into().unwrap()// Converts len() into a u8. Shouldn't be possible to get 256 delvers, but should add in error handling.
    //         {self.delver_index = 0;}
    // }
}


pub fn roll(rng: &mut impl Rng, stat:f32) -> f32 {
    let x:f32 = rng.gen(); //There's a better way to do this
    x * stat
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
            self.resolve_last_event(rng)
        };
        self.game.phase != GamePhase::Finished
    }

    pub fn tick(&mut self, rng:&mut impl Rng) -> () {
        let current_room: &Room = 
        match self.game.rooms.get(&self.game.delver_position) { //Checks for if off-map
            Some(n) => n,
            None => {self.game.delver_position = Coordinate(0,0); return Sim::tick(self, rng)} //TODO: Replace with some special case room, a la hall of flames.
        };
    
        match self.game.phase {
            GamePhase::NotStarted => {
                self.game.phase = GamePhase::Encounter;
            }
            GamePhase::TurnStart => {
                self.game.phase = GamePhase::Encounter;
            }
            GamePhase::Encounter => {
                if current_room.complete {self.game.phase = GamePhase::Forge;}
                else {
                    self.game.phase = GamePhase::TurnStart;
                    // Do encounter rolls
                    // DO IMMEDIATE FIX
                    let base_stat = current_room.room_type.base_stat();
                    let active_delver = self.game.delverteam.choose_delver(base_stat);
                    let delver_index = self.game.delverteam.get_index(active_delver).unwrap();
                    let room_position = self.game.delver_position;
                    current_room.room_type.attempt_clear(&self.game, room_position, delver_index, &mut self.eventqueue);

            }
            }
            GamePhase::Forge => {
                self.game.phase = GamePhase::Travel;
                //Do forging stuff
            }
            GamePhase::Travel => {
                self.game.phase = GamePhase::TurnStart;
                // Do Travel stuff
                let active_delver = self.game.delverteam.choose_delver(DelverStats::Speediness);
                let delver_index = self.game.delverteam.get_index(active_delver).unwrap();

                let position = self.game.delver_position + Coordinate(1,0);

                let message = active_delver.to_string() + " guides the delvers to " + &position.to_string();
                let success = Event::comment_event(EventType::Move {position}.target(delver_index), message);
                
                let message = active_delver.to_string() + " hurts themselves while navigating.";
                let fail = Event::comment_event(EventType::Damage {amount: 1}.target(delver_index), message);
                
                let outcomes = Outcomes{success, fail};
                let event = EventType::Roll { difficulty: roll(rng, self.game.dungeon.lengthiness*100.0), stat: DelverStats::Speediness, outcomes}.no_target();
                self.eventqueue.events.push(event);
            }
            GamePhase::Finished => {}
        }
    }
    pub fn render(&self) {
        let waittime = time::Duration::from_secs(0);
        for p in &self.game.delverteam.delvers {
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

    pub fn resolve_last_event(&mut self, rng: &mut impl Rng) {
        let event = match self.eventqueue.events.pop() {
            Some(n) => n,
            None => return
        };
        self.resolve_event(rng,event);
    }
    pub fn resolve_event(&mut self, rng: &mut impl Rng, event:Event) {    
        let event: Event = {
            let mut event = event;
            let mut modifiers = Vec::new();
            match event.target_index {
                Some(delver_index) => {
                    let d = &self.game.delverteam.delvers[delver_index];
                    for m in &d.modifiers {
                        modifiers.push(m);
                    }
                }
                _ => ()
            };
    
            for m in &modifiers {
                event = 
                match m.replace_event(event, &self.game, &mut self.eventqueue) {
                    ReplaceOutcomes::Stop => EventType::Cancelled.no_target(),
                    ReplaceOutcomes::Event { event } => event,
                    ReplaceOutcomes::Chance { chance, outcomes } => {
                        let (immediate, mut pushed) = outcomes.get(rng.gen::<f32>() < chance);
                        self.eventqueue.events.append(&mut pushed);
                        immediate
                    }
                };
            }
            for m in &modifiers {
                m.pre_event(&event, &self.game, &mut self.eventqueue);
    
            }
            event
        };
        let target_index = event.target_index;
        match event.event_type {
            EventType::Damage {amount} => {
                let delver_index = target_index.unwrap();
                self.game.delverteam.delvers[delver_index].hp -= amount;
                if self.game.delverteam.delvers[delver_index].hp <= 0 {
                    self.eventqueue.events.push(EventType::Death.target(delver_index));
                }
                // self.game.last_log_message = self.game.delverteam.delvers[delver_index].to_string() + " takes damage, bringing them down to " + &self.game.delverteam.delvers[delver_index as usize].hp.to_string() + " hp";
            }
            EventType::Heal {amount } => {
                let delver_index = target_index.unwrap();
                self.game.delverteam.delvers[delver_index as usize].hp += amount;
                if self.game.delverteam.delvers[delver_index as usize].hp > 5 {
                    self.game.delverteam.delvers[delver_index as usize].hp = 5;
                }
                // self.game.last_log_message = self.game.delverteam.delvers[delver_index as usize].to_string() + " heals, bringing them up to " + &self.game.delverteam.delvers[delver_index as usize].hp.to_string() + " hp";
            }
            EventType::Death => {
                let delver_index = target_index.unwrap();

                self.game.delverteam.delvers[delver_index as usize].active = false;
                let alive_delvers = self.game.delverteam.delvers.iter().any(|r| r.active);
                if !alive_delvers {
                    self.eventqueue.events.push(EventType::EndGame.no_target());
                }
                // self.game.last_log_message = self.game.delverteam.delvers[delver_index as usize].to_string() + " dies.";
            }
            EventType::Move {position } => {
                self.game.delver_position = position;
                if self.game.delver_position.0 == 4 { // temporary, need to implement new conditions. 
                    self.eventqueue.events.push(EventType::EndGame.no_target())
                }
            }
            // Complex events: Cannot have post_events, because their insides are often consumed.
            EventType::Roll { difficulty, stat, outcomes} => {
                let delver = self.game.delverteam.choose_delver(stat);
                let mut pushes = outcomes.get(roll(rng,delver.get_stat(stat)) > difficulty);
                self.eventqueue.events.append(&mut pushes); //RENAME VARIABLES
                return
            }
            EventType::EndGame => {self.game.phase = GamePhase::Finished; println!("Game Ended")}
            EventType::Log { message } => {
                // if game.last_log_message != "" {panic!("Log message dropped!")};
                self.game.last_log_message = message;
                return
            }
            EventType::ClearRoom { room_position } => {self.game.rooms.get_mut(&room_position).unwrap().complete = true; return}
            EventType::Cancelled => return
        }
        {
            match event.target_index {
                Some(delver_index) => {
                    let d = &self.game.delverteam.delvers[delver_index];
                    for m in &d.modifiers {
                        m.post_event(&event, &self.game, &mut self.eventqueue);
                    }
                }
                _ => ()
            };

        }
        }
}