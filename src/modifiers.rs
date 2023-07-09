
use serde::{Serialize, Deserialize};

use crate::events::{Event, EventType, EventQueue, OutcomesWithImmediate};
use crate::sim::Game;
use crate::entities::{Entity,Stats};
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
    CheeseThirst,
    TrailBlazer
}
impl BaseModifier {
    pub fn replace_event(&self, event:Event, relation:ModRelation, game:&Game, queue:&mut EventQueue) -> ReplaceOutcomes {
        match self {
            BaseModifier::Pheonix => pheonix::replace_event(event, relation, game, queue),
            BaseModifier::TrailBlazer => trail_blazer::replace_event(event, relation, game, queue),
            _ => ReplaceOutcomes::Event {event}
        }
    }
    pub fn pre_event(&self, event:&Event, relation:ModRelation,  game:&Game, queue:&mut EventQueue) {
        match self {
            BaseModifier::CheeseThirst => cheese_thirst::pre_event(event, relation, game, queue),
            BaseModifier::TrailBlazer => trail_blazer::pre_event(event, relation, game, queue),
            _ => ()
        }
    }
    pub fn get_stat(&self, stat:Stats, statvalue:f32) -> f32 {
        match self {
            BaseModifier::TrailBlazer => trail_blazer::get_stat(stat, statvalue),
            _ => statvalue
        }
    }
}


mod pheonix {
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
                let success = Event {event_type:EventType::Heal (100), target:event.target, source:event.target, message};
                
                let message = Message::Custom(target_name + "'s Pheonix fails. Their ashes scatter to the wind.");
                let mut fail = event;
                fail.message = message; 


                ReplaceOutcomes::Chance { chance: 0.25, success, fail }
            },
            _ => ReplaceOutcomes::Event {event}
        }
    }
}

mod cheese_thirst {
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

mod trail_blazer {
    use crate::modifiers::*;
    pub fn replace_event(event:Event, relation:ModRelation, game:&Game, _queue:&mut EventQueue) -> ReplaceOutcomes {
        if relation == ModRelation::Source {
            match event.event_type {
                EventType::Move(..) => (),
                _ => return ReplaceOutcomes::Event { event }
            }
            let mut event = event;
            event.message = Message::Custom(format!("{} burns a trail forward.", event.source.to_string(game)));
            return ReplaceOutcomes::Event { event };
        }
        return ReplaceOutcomes::Event { event };
    }
    pub fn pre_event(event:&Event, relation:ModRelation,  game:&Game, queue:&mut EventQueue) {
        if relation == ModRelation::Source {
        match event.event_type {
            EventType::Move (..) => {
                let success = Box::new(Event::cancelled());
                
                let message = Message::Custom(format!("{} burns up slightly.", event.source.to_string(game)));
                let fail = Box::new(Event {event_type:EventType::Damage(1), target:event.source, source:event.source, message});

                let event = Event::type_only(EventType::Chance {chance:0.5, success, fail});
                queue.events.push(event);
            },
            _ => ()
        }
    }
    }
    pub fn get_stat(stat:Stats,  statvalue:f32) -> f32 {
        match stat {
            Stats::Exploriness => statvalue + 0.3,
            _ => statvalue
        }
    }
}