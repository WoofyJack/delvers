use crate::teams::{Stats};
use crate::locations::Coordinate;
use crate::sim::Game;
use crate::messaging::Message;
use serde::{Serialize, Deserialize};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Entity {
    Delver {index:usize},
    Room {index:Coordinate},
    Defender {index:usize},
    None
}
impl Entity {
    pub fn to_string(&self, game:&Game) -> String{
        match self {
            Entity::Delver{index} => {game.delverteam.delvers[*index].to_string()},
            Entity::Defender {index} => {game.defenderteam.active_defenders[*index].to_string()},
            _ => panic!("Invalid to_string")
        }
    }
    pub fn get_delver_index(self) -> usize {
        match self {
            Entity::Delver {index} => index,
            _ => panic!("Expected delver")
        }
    }
    pub fn get_stat(self,game:&Game, stat:Stats) -> f32 {
        match self {
            Entity::Delver {index} => game.delverteam.delvers[index].get_stat(stat),
            Entity::Defender{index} => game.defenderteam.active_defenders[index].get_stat(stat),
            _ => panic!("Expected delver or defender")
        }
    }
    // pub fn get_defender_stat(self,game:&Game, stat:Stats) -> f32 {
    //     match self {
    //         Entity::Defender{index} => game.defenderteam.active_defenders[index].get_stat(stat),
    //         _ => panic!("Expected defender")
    //     }
    // }
    
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Scene { //TODO: Implement
//     pub begin:String,
//     pub difficulty:f32,
//     pub stat:Stats,
//     pub success:(Event, String),
//     pub fail: (Event, String)
// }
// impl Scene {
//     pub fn unpack (self) -> (String,f32, Stats,Outcomes) {
//         let success = vec![EventType::Log { message:self.success.1}.no_target_no_source(), self.success.0];
//         let fail = vec![EventType::Log { message:self.fail.1}.no_target_no_source(), self.fail.0];
//         (self.begin,self.difficulty, self.stat, Outcomes {success, fail})
//     }
//     fn create(begin: String, difficulty: f32, stat: Stats, success: (Event, String), fail: (Event, String)) -> Box<Scene> {
//         Box::new(Scene {begin, difficulty, stat, success, fail})
//     }
// }

#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    Damage (i8), //amount
    Heal (i8), //amount
    Move (Coordinate), //Destination
    Death,
    EndGame,
    Log, // Do nothing, still log.
    Roll {difficulty:f32, stat:Stats, outcomes:Outcomes},
    // Scene {scene:Box<Scene>},
    ClearRoom,
    StartBossFight,
    Tick, // Continue with core game loop. TO IMPLEMENT: Should probably error if Message is not None
    Cancelled //"Do nothing" event. TO IMPLEMENT: Should probably error if Message is not None
}
impl EventType {
    // pub fn no_target_no_source(self) -> Event {
    //     Event {event_type:self, target:Entity::None, source:Entity::None, message:Message::None}
    // }
    // pub fn no_target(self, source:Entity) -> Event {
    //     Event {event_type:self, target:Entity::None, source, message:Message::None}
    // }
    // pub fn target(self, source:Entity, target:Entity) -> Event {
    //     Event {event_type:self, target:target ,source, message:Message::None}
    // }
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