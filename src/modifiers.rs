
use serde::{Serialize, Deserialize};

use crate::events::{Event, EventType, EventQueue, Entity, OutcomesWithImmediate};
use crate::sim::Game;
use crate::teams::{Stats};
use crate::messaging::Message;

pub struct ModToApply <'a> {
    pub modifier: &'a BaseModifier,
    pub relation: ModRelation
}
#[derive(Clone, Copy, PartialEq)]
pub enum ModRelation {
    Target, Source
}
pub enum ReplaceOutcomes{
    Stop,
    Event {event:Event},
    Chance {chance:f32, success:Event, fail:Event}
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum BaseModifier {
    Pheonix,
    DoubleOrNothing,
    CheeseThirst
}
impl BaseModifier {
    pub fn replace_event(&self, event:Event, relation:ModRelation, game:&Game, queue:&mut EventQueue) -> ReplaceOutcomes {
        match self {
            BaseModifier::Pheonix => Pheonix::replace_event(event, relation, game, queue),
            BaseModifier::DoubleOrNothing => DoubleOrNothing::replace_event(event, relation, game, queue),
            _ => ReplaceOutcomes::Event {event}
        }
    }
    pub fn pre_event(&self, event:&Event, relation:ModRelation,  game:&Game, queue:&mut EventQueue) {
        match self {
            BaseModifier::CheeseThirst => CheeseThirst::pre_event(event, relation, game, queue),
            _ => ()
        }
    }
    pub fn get_delver_stat(&self, _stat:Stats, statvalue:f32) -> f32 {statvalue}
    pub fn get_defender_stat(&self, _stat:Stats, statvalue:f32) -> f32 {statvalue}
}


mod Pheonix {
    use crate::modifiers::*;

    pub fn replace_event(event:Event, relation:ModRelation, game:&Game, _queue:&mut EventQueue) -> ReplaceOutcomes {
        match relation { 
            ModRelation::Target => (),
            _ => return ReplaceOutcomes::Event {event}
        };

        match event.event_type {
            EventType::Death => {
                let target_name = event.target.to_string(game);
                let message = Message::Custom(target_name.clone() + "'s Pheonix activates. They are reborn from their ashes!");
                let success = Event {event_type:EventType::Heal (5), target:event.target, source:event.target, message};
                
                let message = Message::Custom(target_name + "'s Pheonix fails. Their ashes scatter to the wind.");
                let mut fail = event;
                fail.message = message; 


                ReplaceOutcomes::Chance { chance: 0.25, success, fail }
            },
            _ => ReplaceOutcomes::Event {event}
        }
    }
}
mod DoubleOrNothing {
    use crate::modifiers::*;
    pub fn replace_event(event:Event,  relation:ModRelation, game:&Game, queue:&mut EventQueue) -> ReplaceOutcomes {
        // let self_name = match relation {
        //     ModRelation::Source => event.source.to_string(game),
        //     ModRelation::Target => event.target.to_string(game),
        //     _ => return ReplaceOutcomes::Event {event}
        // };
        match event.event_type {
            EventType::Damage (amount ) => {
                // let message = self_name + " doubles the risk";
                // queue.log(message);
                let event = Event{event_type:EventType::Damage (amount+1), source:event.source, target:event.target, message:Message::None};
                ReplaceOutcomes::Event { event }
            },
            _ => ReplaceOutcomes::Event {event}
        }
    }
}

mod CheeseThirst {
    use crate::modifiers::*;
    pub fn pre_event(event:&Event, relation:ModRelation,  game:&Game, queue:&mut EventQueue) {
        if relation == ModRelation::Source {
        match event.event_type {
            EventType::Death => {
                let message = Message::Custom(format!("{} devours their cheese", event.source.to_string(game)));
                let event = Event {event_type:EventType::Heal(2), target:event.source, source:event.source, message};
                queue.events.push(event);
            },
            _ => ()
        }
    }
    }
}