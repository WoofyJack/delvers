
use serde::{Serialize, Deserialize};

use crate::events::{Event, EventType, EventQueue, Entity, OutcomesWithImmediate};
use crate::sim::Game;
use crate::teams::{Stats};


pub struct ModToApply <'a> {
    pub modifier: &'a BaseModifier,
    pub relation: ModRelation
}
#[derive(Clone, Copy)]
pub enum ModRelation {
    Target, Source
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum BaseModifier {
    Pheonix,
    DoubleOrNothing
}
impl BaseModifier {
    pub fn replace_event(&self, event:Event, relation:ModRelation, game:&Game, queue:&mut EventQueue) -> ReplaceOutcomes {
        match self {
            BaseModifier::Pheonix => Pheonix::replace_event(event, relation, game, queue),
            BaseModifier::DoubleOrNothing => DoubleOrNothing::replace_event(event, relation, game, queue),
            _ => ReplaceOutcomes::Event {event}
        }
    }
    pub fn pre_event(&self, _event:&Event, relation:ModRelation,  _game:&Game, _queue:&mut EventQueue) {
        match self {
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

        let target_name = match event.target {
            Entity::Delver { index } => game.delverteam.delvers[index].to_string(),
            Entity::Defender { index } => game.defenderteam.active_defenders[index].to_string(),
            _ => "".to_string()
        };
        let failmessage = target_name.clone() + "'s Pheonix fails. Their ashes scatter to the wind.";
        let successmessage= target_name + "'s Pheonix activates. They are reborn from their ashes!";
        match event.event_type {
            EventType::Death => {
                let outcomes = OutcomesWithImmediate{
                immediate_success:EventType::Heal {amount: 5 }.target(event.target,event.target),
                immediate_fail: event,
                fail:vec![EventType::Log {message:failmessage}.no_target_no_source()],
                success:vec![EventType::Log { message: successmessage}.no_target_no_source(), ]
                };
                ReplaceOutcomes::Chance { chance: 0.25, outcomes: outcomes }
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
            EventType::Damage { amount } => {
                // let message = self_name + " doubles the risk";
                // queue.log(message);
                let event = EventType::Damage { amount:amount+1 }.target(event.source, event.target);
                ReplaceOutcomes::Event { event }
            },
            _ => ReplaceOutcomes::Event {event}
        }
    }
}

pub enum ReplaceOutcomes{
    Stop,
    Event {event:Event},
    Chance {chance:f32, outcomes:OutcomesWithImmediate}
}