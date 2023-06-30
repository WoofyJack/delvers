
use serde::{Serialize, Deserialize};

use crate::{sim::{Event, EventType, Game, EventQueue}, teams::DelverStats};


#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum PermanentModifiers {
    Pheonix,
}
impl PermanentModifiers {
    pub fn to_game_mod(&self) -> Box<dyn Modifier> {
        match self {
            PermanentModifiers::Pheonix => Box::new(Pheonix)
        }
    }
}

// Jumping through hoops cus we aren't allowed to modify game while we're iterating through modifiers stored in game.
pub trait Modifier {
    // on_phase
    fn replace_event(&self, event:Event, _game:&Game, _queue:&mut EventQueue) -> ReplaceOutcomes {ReplaceOutcomes::Event {event}}
    fn pre_event(&self, _event:&Event, _game:&Game, _queue:&mut EventQueue) {}
    fn post_event(&self, _event:&Event, _game:&Game, _queue:&mut EventQueue) {}
    fn get_stat(&self, _stat:DelverStats, statvalue:f32) -> f32 {statvalue}
}

pub enum ReplaceOutcomes{
    Stop,
    Event {event:Event},
    Chance {chance:f32, outcomes:OutcomesWithImmediate}
}

pub struct OutcomesWithImmediate {
    pub immediate_success:Event,
    pub success:Vec<Event>,
    pub immediate_fail:Event,
    pub fail:Vec<Event>
}
impl OutcomesWithImmediate {
    pub fn get(self, bool:bool) -> (Event, Vec<Event>) {
        if bool {
            return (self.immediate_success, self.success)
        }
        else {
            return (self.immediate_fail, self.fail)
        }
    }
}
pub struct Outcomes {
    pub success:Vec<Event>,
    pub fail:Vec<Event>
}
impl Outcomes {
    pub fn get(self, bool:bool) -> Vec<Event> {
        if bool {
            return self.success
        }
        else {
            return self.fail
        }
    }
}

pub struct Pheonix;
impl Modifier for Pheonix { // I guess just allow modifiers to do their own rolls. No, trait objects don't like being passed rng.
    fn replace_event(&self, event:Event, game:&Game, _queue:&mut EventQueue) -> ReplaceOutcomes {
        let delver_index = event.target_index.unwrap();
        let failmessage = String::from(game.delverteam.delvers[delver_index].to_string()) + " fails to defy death!";
        let successmessage= String::from(game.delverteam.delvers[delver_index].to_string()) + " defies death!";
        match event.event_type {
            EventType::Death => {
                let outcomes = OutcomesWithImmediate{
                immediate_fail: event,
                fail:vec![EventType::Log {message:failmessage}.no_target()],
                immediate_success:EventType::Heal {amount: 5 }.target(delver_index),
                success:vec![EventType::Log { message: successmessage}.no_target(), ]
                };
                ReplaceOutcomes::Chance { chance: 0.25, outcomes: outcomes }
            },
            _ => ReplaceOutcomes::Event {event}
        }
    }
    fn get_stat(&self, stat:DelverStats, statvalue:f32) -> f32 {
        match stat {
            DelverStats::Fightiness => statvalue * 2.0,
            _ => statvalue
        }
    }
}