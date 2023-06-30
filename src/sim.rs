use rand::Rng;
use rand::seq::SliceRandom;

use crate::locations::{Coordinate, Room, BossFight};
use crate::modifiers::{Outcomes, ReplaceOutcomes};
use crate::teams::{Dungeon, DelverStats, DelverTeam, Delver, Defender, DefenderStats, DefenderTeam};
use std::collections::HashMap;

use std::{thread, time};

use colored::Colorize;

pub struct  Sim {
    pub game: Game,
    pub eventqueue:EventQueue,
    pub finished:bool
}
#[derive(Debug)]
pub struct Event {
    pub target:Target,
    pub event_type:EventType
}
#[derive(Clone, Copy, Debug)]
pub enum Target {
    Delver {index:usize},
    Room {index:usize},
    Defender,
    None
}
impl Event {
    // pub fn no_target(event_type:EventType) -> Event {
    //     Event {target_index:Option::None, event_type}
    // }
    // pub fn has_target (target:usize, event_type:EventType) -> Event {
    //     Event {target_index:Option::Some(target), event_type}
    // }
    pub fn comment_event(event:Event, message:String) -> Vec<Event> {
        vec![(EventType::Log{message}).no_target(),event]
    }
}
#[derive(Debug)]
pub enum EventType {
    Damage {amount:i8},
    Heal {amount:i8},
    Move {position:Coordinate},
    Death,
    EndGame,
    Log {message:String},
    // Roll {difficulty:f32, stat:} //Figure this out later.
    Roll {difficulty:f32, stat:DelverStats, outcomes:Outcomes},
    // RandomTarget {event:Box<EventType>},
    ClearRoom {room_position:Coordinate},
    BossFight {room_position:Coordinate},
    Cancelled
}
impl EventType {
    pub fn no_target(self) -> Event {
        Event {event_type:self, target:Target::None}
    }
    pub fn target(self, target:Target) -> Event {
        Event {event_type:self, target:target}
    }
    pub fn target_delver(self, index:usize) -> Event {
        Event {event_type:self, target:Target::Delver{index}}
    }
    pub fn target_defender(self) -> Event {
        Event {event_type:self, target:Target::Defender}
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

    pub delverteam:DelverTeam,
    pub defenderteam:DefenderTeam,
    
    rooms: HashMap<Coordinate, Room>,
    delver_position:Coordinate,

    last_log_message:String,
    pub rand_target:usize,
}


impl Game {
    pub fn new_game(delverteam:DelverTeam, defenderteam:DefenderTeam, rooms: HashMap<Coordinate,Room>) -> Game {
        Game {phase:GamePhase::NotStarted,
            delverteam, defenderteam, 
            rooms,
            delver_position: Coordinate(0,0),
            last_log_message:String::from(""),
            rand_target:0
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
                let success = Event::comment_event(EventType::Move {position}.target_delver(delver_index), message);
                
                let message = active_delver.to_string() + " hurts themselves while navigating.";
                let fail = Event::comment_event(EventType::Damage {amount: 1}.target_delver(delver_index), message);
                
                let outcomes = Outcomes{success, fail};
                let event = EventType::Roll { difficulty: roll(rng, self.game.defenderteam.dungeon.lengthiness), stat: DelverStats::Speediness, outcomes}.no_target();
                self.eventqueue.events.push(event);
            }
            GamePhase::Finished => {}
        }
    }
    pub fn render(&self) {
        let waittime = time::Duration::from_secs(1);
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
        {
            let p = &self.game.defenderteam.defender;
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
        
        // for e in &self.eventqueue.events {
        //     print!("{:?}", e.event_type);
        // }
        // println!();
        
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
        self.game.rand_target = match self.game.delverteam.active_delvers().choose(rng) {
            Some(i) => *i,
            None => 100
        };
        let event: Event = {
            let mut event = event;
            let mut modifiers = Vec::new();
            match event.target {
                Target::Delver {index} => {
                    let d = &self.game.delverteam.delvers[index];
                    for m in &d.modifiers {
                        modifiers.push(m);
                    }
                },
                Target::Defender => {
                    let d = &self.game.defenderteam.defender;
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
        match event.event_type {
            EventType::Damage {amount} => {
                match event.target {
                    Target::Delver { index } => {
                        self.game.delverteam.delvers[index].hp -= amount;
                        if self.game.delverteam.delvers[index].hp <= 0 {
                            self.eventqueue.events.insert(0,EventType::Death.target_delver(index));
                        }
                    }
                    Target::Defender => {
                        self.game.defenderteam.defender.hp -= amount;
                        if self.game.defenderteam.defender.hp <= 0 {
                            self.eventqueue.events.insert(0,EventType::Death.target_defender());
                        }
                    }
                    _ => ()
                }
            }
            EventType::Heal {amount } => {
                match event.target {
                    Target::Delver { index } => {
                        self.game.delverteam.delvers[index].hp += amount;
                        if self.game.delverteam.delvers[index].hp > 5 {
                            self.game.delverteam.delvers[index].hp = 5;
                        }
                    }
                    Target::Defender => {
                        self.game.defenderteam.defender.hp += amount;
                        if self.game.defenderteam.defender.hp > 5 {
                            self.game.defenderteam.defender.hp = 5;
                        }
                    }
                    _ => ()
                }

            }
            EventType::Death => {
                match event.target {
                    Target::Delver { index } => {
                        self.game.delverteam.delvers[index].active = false;
                        let alive_delvers = self.game.delverteam.active_delvers();
                        if alive_delvers.len() == 0 {
                            self.eventqueue.events.insert(0,EventType::EndGame.no_target());
                        }
                    }
                    Target::Defender => {self.game.defenderteam.defender.active = false;}
                    _ => ()
            }
            }
            EventType::Move {position } => {
                match event.target {
                    Target::Delver { index } => {}
                    _ => {panic!("Invalid target")}
                }
                self.game.delver_position = position;
                if self.game.delver_position.0 == self.game.rooms.len() as i8 { // temporary, need to implement new conditions. 
                    self.eventqueue.events.insert(0,EventType::EndGame.no_target())
                }
            }
            EventType::BossFight { room_position } => {
                let defender = &self.game.defenderteam.defender;
                if !defender.active {
                    self.game.rooms.get_mut(&room_position).unwrap().complete = true;
                }
                else {
                let active_delver = self.game.delverteam.choose_delver(DelverStats::Fightiness);
                if roll(rng, active_delver.get_stat(DelverStats::Fightiness)) > roll(rng, defender.get_stat(DefenderStats::Fightiness)) {
                    if true{//defender.hp > 1 {
                        let message = active_delver.to_string() + " injures " + &defender.to_string();// + " leaving them on " + &(defender.hp-1).to_string() + " hp";
                        self.eventqueue.events.push(EventType::Log { message }.no_target());
                    }
                    self.eventqueue.events.push (EventType::Damage { amount: 1 }.target_defender())
                } else {
                    if true {//active_delver.hp > 1 {
                        let message = defender.to_string() + " injures " + &active_delver.to_string();
                        self.eventqueue.events.push(EventType::Log { message }.no_target());
                    }
                    self.eventqueue.events.push (EventType::Damage { amount: 1 }.target_delver(self.game.delverteam.get_index(active_delver).unwrap()));
                }
            }
            }
            // Complex events: Cannot have post_events, because their insides are often consumed.
            EventType::Roll { difficulty, stat, outcomes} => {
                let active_delver = self.game.delverteam.choose_delver(stat);
                
                let total_stat = Delver::collect_stats(active_delver, &self.game.delverteam.delvers, stat);

                let mut pushes = outcomes.get(roll(rng, total_stat) > difficulty);
                self.eventqueue.events.append(&mut pushes); //RENAME VARIABLES
                return
            }
            EventType::EndGame => {self.game.phase = GamePhase::Finished; println!("Game Ended")}
            EventType::Log { message } => {
                // if game.last_log_message != "" {panic!("Log message dropped!")};
                self.game.last_log_message = message;
                return
            }
            // EventType::RandomTarget { event } => {
            //     let target = rng.gen_range(0..self.game.delverteam.active_delvers().len());
            //     self.eventqueue.events.push(event.target(target));
            //     return
            // }
            EventType::ClearRoom { room_position } => {self.game.rooms.get_mut(&room_position).unwrap().complete = true; return}
            EventType::Cancelled => return
        }
        {
            let mut modifiers = Vec::new();
            match event.target {
                Target::Delver {index} => {
                    let d = &self.game.delverteam.delvers[index];
                    for m in &d.modifiers {
                        modifiers.push(m);
                    }
                },
                Target::Defender => {
                    let d = &self.game.defenderteam.defender;
                    for m in &d.modifiers {
                        modifiers.push(m);
                    }
                }
                _ => ()
            };
            for m in modifiers {
                m.post_event(&event, &self.game, &mut self.eventqueue);
            }

        }
        }
}