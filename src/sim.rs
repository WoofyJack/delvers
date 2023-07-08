use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::locations::{Coordinate, Room};
use crate::modifiers::{ReplaceOutcomes, ModToApply, ModRelation};
use crate::teams::{Dungeon, Stats, DelverTeam, Delver, Defender, DefenderTeam};
use crate::events::{EventQueue, Entity, Event, EventType, Scene, Outcomes};
use crate::core_loop::{GamePhase};

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
    
    #[serde_as(as = "Vec<(_, _)>")]
    pub rooms: HashMap<Coordinate, Room>,
    pub delver_position:Coordinate,

    pub last_log_message:String,
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
        let waittime = time::Duration::from_secs(0);
        for p in &self.game.delverteam.delvers {
            let delvername =p.to_string();
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
        for p in &self.game.defenderteam.active_defenders{
            // let p = &self.game.defenderteam.defender;
            let delvername = if p.active {p.to_string().normal()} else {p.to_string().truecolor(100,100,100)};
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
            None => EventType::Tick.no_target_no_source()
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
                    ReplaceOutcomes::Stop => EventType::Cancelled.no_target_no_source(),
                    ReplaceOutcomes::Event { event } => event,
                    ReplaceOutcomes::Chance { chance, outcomes } => {
                        let (immediate, mut pushed) = outcomes.get(rng.gen::<f32>() < chance);
                        self.eventqueue.events.append(&mut pushed);
                        immediate
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
        match event.event_type {
            EventType::Damage {amount} => {
                match event.target {
                    Entity::Delver { index } => {
                        self.game.delverteam.delvers[index].hp -= amount;
                        if self.game.delverteam.delvers[index].hp <= 0 {
                            self.eventqueue.events.insert(0,EventType::Death.target(event.source,event.target));
                        }
                    }
                    Entity::Defender {index} => {
                        let mut defender:&mut Defender = self.game.defenderteam.active_defenders.get_mut(index).unwrap();
                        defender.hp -= amount;
                        if defender.hp <= 0 {
                            self.eventqueue.events.insert(0,EventType::Death.target(event.source,event.target));
                        }
                    }
                    _ => ()
                }
            }
            EventType::Heal {amount } => {
                match event.target {
                    Entity::Delver { index } => {
                        self.game.delverteam.delvers[index].hp += amount;
                        if self.game.delverteam.delvers[index].hp > 5 {
                            self.game.delverteam.delvers[index].hp = 5;
                        }
                    }
                    Entity::Defender {index}=> {
                        let mut defender = self.game.defenderteam.active_defenders.get_mut(index).unwrap();
                        defender.hp += amount;
                        if defender.hp > 5 {
                            defender.hp = 5;
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
                            self.eventqueue.events.insert(0,EventType::EndGame.no_target_no_source());
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
            EventType::Move {position } => {
                match event.target {
                    Entity::Delver { index } => {}
                    _ => {panic!("Invalid target")}
                }
                self.game.delver_position = position;
                if self.game.delver_position.0 == self.game.rooms.len() as i8 { // temporary, need to implement new conditions. 
                    self.eventqueue.events.insert(0,EventType::EndGame.no_target_no_source())
                }
            }
            EventType::StartBossFight => {
                let defender = self.game.defenderteam.defender.clone().to_game_defender();
                let message = self.game.delverteam.to_string() + " challenge " + &self.game.defenderteam.to_string() +"'s defender " + &defender.to_string();
                self.eventqueue.log(message);
                self.game.defenderteam.active_defenders.push(defender);
                
            }
            EventType::Tick => {
                crate::core_loop::tick(self, rng);
            }
            //  -------------------- Complex events: Cannot be accessed afterwards, because their insides are often consumed. -------------
            EventType::Roll { difficulty, stat, outcomes} => {
                let active_delver = self.game.delverteam.choose_delver(stat);
                
                let total_stat = Delver::collect_stats(active_delver, &self.game.delverteam.delvers, stat);

                let mut pushes = outcomes.get(roll(rng, total_stat) > difficulty * rng.gen::<f32>());
                self.eventqueue.events.append(&mut pushes);
            }
            EventType::EndGame => {self.game.phase = GamePhase::Finished; self.eventqueue.log(String::from("Game Ended"));}
            EventType::Log { message } => {
                self.game.last_log_message = message;
            }
            EventType::Scene { scene } => {
                let (message, difficulty, stat, outcomes) = scene.unpack();
                self.eventqueue.events.push(EventType::Roll {difficulty, stat, outcomes }.no_target_no_source());
                self.eventqueue.events.push(EventType::Log { message }.no_target_no_source());
            }
            EventType::ClearRoom => {
                let index =match event.target {
                    Entity::Room { index } => index,
                    _ => panic!("Invalid Target"),
                };
                self.game.rooms.get_mut(&index).unwrap().complete = true;}
            EventType::Cancelled => ()
        }
        }
}