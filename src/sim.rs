use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::room_types::{Coordinate, RoomType};
use crate::messaging::Message;
use crate::modifiers::{ReplaceOutcomes, ModToApply, ModRelation};
use crate::entities::{Entity, Room, Stats, Delver, Defender, DelverTeam, DefenderTeam, Dungeon};
use crate::events::{EventQueue, Event, EventType, Outcomes};
use crate::core_loop::GamePhase;

use std::collections::HashMap;

use std::{thread, time};

use colored::{Colorize, ColoredString};

#[derive(Serialize, Deserialize)]
pub struct  Sim {
    pub game: Game,
    pub eventqueue:EventQueue,
    pub finished:bool
}


#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Game {
    pub phase:GamePhase,

    pub delverteam:DelverTeam,
    pub defenderteam:DefenderTeam,
    
    // #[serde_as(as = "Vec<(_, _)>")]
    // pub room: HashMap<Coordinate, Room>,
    pub current_room:Room,
    pub depth:i8,
    // pub delver_position:Coordinate,

    pub last_log_message:String,
    pub rand_target:usize,
}


impl Game {
    pub fn new_game(delverteam:DelverTeam, defenderteam:DefenderTeam) -> Game {
        Game {phase:GamePhase::NotStarted,
            delverteam, defenderteam,
            current_room:Room {complete:false, room_type:RoomType::Empty},
            depth:0,
            last_log_message:String::from(""),
            rand_target:0
        }
    }
}


pub fn roll(rng: &mut impl Rng, stat:f32) -> f32 {
    let x:f32 = rng.gen(); //There's a better way to do this
    x * stat
}

impl Sim {
    pub fn next_frame(&mut self, rng:&mut impl Rng) -> bool {
        while self.game.last_log_message == "" { 
            self.resolve_last_event(rng);
        };
        self.render();
        self.game.last_log_message = String::from("");
        return self.game.phase != GamePhase::Finished
    }

    
    pub fn render(&self) {
        let waittime = time::Duration::from_secs(1);
        
        // Rooms
        // let mut from_room = false;
        // for x in 0..10 {
        //     let coord = &Coordinate(x, 0);
        //     if self.game.rooms.contains_key(coord) {
        //         if from_room {print!("---")}
        //         from_room = true;
        //         print!("{}",self.game.rooms[coord].to_string());
        //     } else{
        //         print!("  ")
        //     };
        // }
        // println!("");

        // Delver Names + Hp.
        for p in &self.game.delverteam.delvers {
            let delvername =p.to_string();
            print!("{}: ",delvername);

            let active = "O".red();
            let inactive = "O";
            for i in 0..p.maxhp {
                if i+1 <= p.hp {
                    print!("{}", active);
                }
                else{
                    print!("{}", inactive);
                }
            }
            println!();
        }
        // Defender Names + Hp.
        for p in &self.game.defenderteam.active_defenders{
            let delvername = p.to_string();
            print!("{}: ",delvername);

            let active = "O".red();
            let inactive = "O";
            for i in 0..p.maxhp {
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
        
        // Printout events, can be enabled for debugging
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
            None =>  Event::type_only(EventType::Tick)
        };
        self.resolve_event(rng,event);
    }
    pub fn resolve_event(&mut self, rng: &mut impl Rng, event:Event) {
        // Janky solution for traps to randomly target.
        self.game.rand_target = match self.game.delverteam.active_delvers().choose(rng) {
            Some(i) => *i,
            None => 100 // Events that can occur with 0 alive delvers should not target a random alive delver until the jank is fixed.
        };
        // ------------------------- Gather and apply modifiers to event: --------------------------
        let event: Event = {
            let mut event = event;
            let mut modifiers = Vec::new();
            match event.target {
                Entity::Delver {index} => {
                    let d = &self.game.delverteam.delvers[index];
                    for modifier in &d.modifiers {
                        modifiers.push(ModToApply {modifier, relation:ModRelation::Target} );
                    }
                },
                Entity::Defender { index } => {
                    let d = &self.game.defenderteam.active_defenders[index];
                    for modifier in &d.modifiers {
                        modifiers.push(ModToApply {modifier, relation:ModRelation::Target});
                    }
                }
                _ => ()
            };
            match event.source {
                Entity::Delver {index} => {
                    let d = &self.game.delverteam.delvers[index];
                    for modifier in &d.modifiers {
                        modifiers.push(ModToApply {modifier, relation:ModRelation::Source} );
                    }
                },
                Entity::Defender {index} => {
                    let d = &self.game.defenderteam.active_defenders[index];
                    for modifier in &d.modifiers {
                        modifiers.push(ModToApply {modifier, relation:ModRelation::Source});
                    }
                }
                _ => ()
            };
            // -------------------------- Modifiers' replace_event called. ---------------------
            for m in &modifiers {
                let (m, relation) = (m.modifier, m.relation);
                event = 
                match m.replace_event(event, relation, &self.game, &mut self.eventqueue) {
                    ReplaceOutcomes::Stop => Event::cancelled(),
                    ReplaceOutcomes::Event { event } => event,
                    ReplaceOutcomes::Chance { chance, success, fail } => {
                        if rng.gen::<f32>() < chance {success} else {fail} 
                    }
                };
            }
            // -------------------------- Modifiers' pre_event called. ---------------------
            for m in &modifiers {
                let (m, relation) = (m.modifier, m.relation);
                m.pre_event(&event, relation, &self.game, &mut self.eventqueue);
    
            }
            event
        };

        // ------------------------ Events Happen -----------------------------------------------------
        self.game.last_log_message = event.message.to_string(&self.game);



        match event.event_type {
            EventType::Damage (amount) => {
                match event.target {
                    Entity::Delver { index } => {
                        self.game.delverteam.delvers[index].hp -= amount;
                        if self.game.delverteam.delvers[index].hp <= 0 {
                            let event = Event {event_type:EventType::Death, source:event.source, target:event.target, message:Message::Death(event.target)};
                            self.eventqueue.events.insert(0,event);
                        }
                    }
                    Entity::Defender {index} => {
                        let mut defender:&mut Defender = self.game.defenderteam.active_defenders.get_mut(index).unwrap();
                        defender.hp -= amount;
                        if defender.hp <= 0 {
                            let event = Event {event_type:EventType::Death, source:event.source, target:event.target, message:Message::Death(event.target)};
                            self.eventqueue.events.insert(0,event);
                        }
                    }
                    _ => ()
                }
            }
            EventType::Heal (amount) => {
                match event.target {
                    Entity::Delver { index } => {
                        let delver = self.game.delverteam.delvers.get_mut(index).unwrap();
                        delver.hp += amount;
                        if delver.hp > delver.maxhp {
                            delver.hp = delver.maxhp;
                        }
                    }
                    Entity::Defender {index}=> {
                        let mut defender = self.game.defenderteam.active_defenders.get_mut(index).unwrap();
                        defender.hp += amount;
                        if defender.hp > defender.maxhp {
                            defender.hp = defender.maxhp;
                        }
                    }
                    _ => ()
                }

            }
            EventType::Death => {
                match event.target {
                    Entity::Delver { index } => {
                        self.game.delverteam.delvers[index].active = false;
                        let alive_delvers = self.game.delverteam.active_delvers();
                        if alive_delvers.len() == 0 {
                            self.eventqueue.events.insert(0,Event::type_only(EventType::EndGame));
                        }
                    }
                    Entity::Defender {index} => {
                        // self.game.defenderteam.active_defenders[index].active = false;
                        self.game.defenderteam.active_defenders.remove(index);
                        if self.game.defenderteam.active_defenders.len() == 0 {
                            self.game.phase = GamePhase::TurnStart;
                        }
                    }
                    _ => ()
            }
            }
            EventType::Delve => {
                match event.source {
                    Entity::Delver {..} => {}
                    _ => {panic!("Invalid source")}
                }
                self.game.depth += 1;
                match self.game.depth {
                    5 => self.eventqueue.events.push(Event::type_only(EventType::StartBossFight)),
                    6.. => self.eventqueue.events.insert(0,Event::type_only(EventType::EndGame)),
                    _ => self.game.current_room = Room::new_room(rng)
                }
            }
            EventType::StartBossFight => {
                let mut defender = self.game.defenderteam.defender.clone().to_game_defender();
                defender.maxhp = 7;
                defender.hp = 7;
                let message = Message::Custom(self.game.delverteam.to_string() + " challenge " + &self.game.defenderteam.to_string() +"'s defender " + &defender.to_string());
                self.eventqueue.log(message);
                self.game.defenderteam.active_defenders.push(defender);
                
            },
            EventType::ClearRoom => { self.game.current_room.complete = true;}
            EventType::Tick => {
                crate::core_loop::tick(self, rng);
            }
            //  -------------------- Complex events: Cannot be accessed afterwards, because their insides are consumed. -------------
            EventType::SpawnDefender (defender) => {
                self.game.defenderteam.active_defenders.push(defender);
            }
            EventType::Roll { difficulty, stat, outcomes} => {
                let active_delver = self.game.delverteam.choose_delver(stat);
                
                let total_stat = Delver::collect_stats(&active_delver, &self.game.delverteam.delvers, stat);

                let mut pushes = outcomes.get(roll(rng, total_stat) > difficulty * rng.gen::<f32>());
                self.eventqueue.events.append(&mut pushes);
            }
            EventType::Chance {chance, success, fail} => {
                let event = if chance > rng.gen::<f32>() {success} else {fail};
                self.eventqueue.events.push(*event);
            }
            EventType::EndGame => {self.game.phase = GamePhase::Finished; self.eventqueue.log(Message::Custom(String::from("Game Ended")));}
            EventType::Log => (),
            EventType::Cancelled => ()
        }
        }
}