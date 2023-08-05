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
    Target, Source,
    Team,
    // FriendlyDungeon, EnemyDungeon
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
                EventType::Delve => (),
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
            EventType::Delve => {
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




#[derive(Debug,Deserialize, Serialize)]
enum GenericEntity {
    Source,
    Target,
    None
}
impl GenericEntity {
    fn to_entity(&self, triggering_event:&Event) -> Entity {
        match self {
            GenericEntity::Target => triggering_event.target,
            GenericEntity::Source => triggering_event.source,
            GenericEntity::None => Entity::None
        }
    }
}
#[derive(Debug,Deserialize, Serialize)]
enum GenericString {
    Phrase(String),
    EntityName(GenericEntity)
}
impl GenericString {
    fn to_string(&self, triggering_event:&Event, game:&Game) -> String {
        match self {
            GenericString::Phrase(str) => str.clone(),
            GenericString::EntityName(entity) => entity.to_entity(triggering_event).to_string(game)
        }
    }
}
#[derive(Debug,Deserialize, Serialize)]
enum GenericMessage {
    Default (Message),
    Custom (Vec<GenericString>)
}
impl GenericMessage {
    fn to_message(self, triggering_event:&Event, game:&Game) -> Message {
        match self {
            GenericMessage::Default(message) => message,
            GenericMessage::Custom(strings) => {
                let mut base = String::new();
                for i in strings {
                    base += &i.to_string(triggering_event, game)
                };
                Message::Custom(base)
            }
        }
    }
}

#[derive(Debug,Deserialize, Serialize)]
struct GenericEvent {
    target:GenericEntity,
    source:GenericEntity,
    message:GenericMessage,
    event_type:EventType
}

impl GenericEvent {
    fn to_event(self, triggering_event:&Event, game:&Game) -> Event{
        Event { target: self.target.to_entity(triggering_event),
                source: self.source.to_entity(triggering_event),
                message:self.message.to_message(triggering_event, game),
                event_type: self.event_type}
    }
}
#[derive(Debug,Deserialize, Serialize)]
enum GenericReplace {
    AlwaysReplace{event_type:EventType, replace_with:GenericEvent},
    ChanceReplace{event_type:EventType, replace_with:GenericEvent, else_message:GenericMessage, chance:f32},
}

#[derive(Debug,Deserialize, Serialize)]
enum GenericPre {
    AlwaysEvent (GenericEvent),
    ChanceEvent (GenericEvent,GenericEvent, f32)
}
#[derive(Debug,Deserialize, Serialize)]
enum GenericGet {

}
#[derive(Debug,Deserialize, Serialize)]
pub struct GenericModifier {
    replaces:Vec<GenericReplace>,
    pres:Vec<GenericPre>,
    gets:Vec<GenericGet>
}

use std::mem::discriminant;
fn apply_generic_replace (triggering_event:Event, modifier:GenericReplace, game:&Game) -> ReplaceOutcomes {
    match modifier {
        GenericReplace::AlwaysReplace { event_type, replace_with } => {
            if discriminant(&replace_with.event_type) == discriminant(&event_type) {
                return ReplaceOutcomes::Event {event:replace_with.to_event(&triggering_event, game)};
            }
        }
        GenericReplace::ChanceReplace { event_type, replace_with, else_message, chance } => {
            if discriminant(&event_type) == discriminant(&event_type) {
                let success = replace_with.to_event(&triggering_event, game);
                let mut fail = triggering_event;
                fail.message = else_message.to_message(&fail, game);
                return ReplaceOutcomes::Chance { chance, success, fail};
            }
        }
    };
    return ReplaceOutcomes::Event { event:triggering_event }
}


pub fn example_event() -> GenericModifier {
    use GenericEntity::*;
    use EventType::*;
    use GenericString::*;

    let mut replaces = Vec::new();
    
    let message = GenericMessage::Custom(vec![EntityName(Source), Phrase("'s Pheonix activates. They are reborn from their ashes!".to_string())]);
    let replace_with = GenericEvent {event_type:Heal(5), target:Target, source:Target, message};
    let else_message = GenericMessage::Custom(vec![EntityName(Source), Phrase("'s Pheonix fails. Their ashes scatter to the wind.".to_string())]);

    let modifier = GenericReplace::ChanceReplace{event_type:EventType::Death, chance:0.25, replace_with, else_message};

    println!("{}",serde_json::to_string_pretty(&modifier).unwrap());

    replaces.push(modifier);
    let pres = Vec::new();
    let gets = Vec::new();
    GenericModifier { replaces, pres, gets }
}

// pub fn human_to_modifier() {
//     let human = "{"name":}"
// }