
use crate::{sim::{Event, Game, EventQueue}, teams::DelverStats};


// Jumping through hoops cus we aren't allowed to modify game while we're iterating through modifiers stored in game.
// Should probably store delvers seperately, game stores refs to it?
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
    fn replace_event(&self, event:Event, game:&Game, queue:&mut EventQueue) -> ReplaceOutcomes {
        match event {
            Event::Death { delver_index } => {
                let outcomes = OutcomesWithImmediate{
                immediate_fail: event,
                fail:vec![Event::Log { message: String::from(game.delvers[delver_index as usize].to_string()) + " fails to defy death!" }],
                immediate_success:Event::Heal { delver_index: delver_index, amount: 5 },
                success:vec![Event::Log { message: String::from(game.delvers[delver_index as usize].to_string()) + " defies death!" }, ]
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