use serde::{Serialize, Deserialize};

use crate::entities::{Entity, Stats, Delver, Defender};
use crate::room_types::Coordinate;
use crate::sim::Game;
use crate::messaging::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub target:Entity,
    pub source:Entity,
    pub message:Message,
    pub event_type:EventType
}
impl Event {
    pub fn cancelled() -> Event {
        Event::type_only(EventType::Cancelled)
    }
    pub fn type_only(event_type:EventType) -> Event {
        Event {event_type, target:Entity::None, source:Entity::None, message:Message::None}
    }
    pub fn type_and_message(event_type:EventType, message:Message) -> Event {
        Event {event_type, target:Entity::None, source:Entity::None, message}
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    Damage (i8), //amount
    Heal (i8), //amount
    Delve,
    Death,
    EndGame,
    Log, // Do nothing, still log.
    Roll {difficulty:f32, stat:Stats, outcomes:Outcomes},
    Chance {chance:f32, success:Box<Event>, fail:Box<Event>},
    // Scene {scene:Box<Scene>},
    ClearRoom,
    StartBossFight,
    SpawnDefender (Defender),
    Tick, // Continue with core game loop. TO IMPLEMENT: Should probably error if Message is not None
    Cancelled //"Do nothing" event. TO IMPLEMENT: Should probably error if Message is not None
}

#[derive(Serialize, Deserialize)]
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
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Serialize, Deserialize)]
pub struct EventQueue
{
    pub events:Vec<Event> //Probably doesnt need to be a struct. Fix later.
}
impl EventQueue {
    pub fn new_queue() -> EventQueue {
        EventQueue {events:Vec::new()}
    }
    pub fn log(&mut self, message:Message) {
        self.events.push(Event::type_and_message(EventType::Log, message));
    }
}