
use crate::sim::{Event, Game, EventQueue};


// Jumping through hoops cus we aren't allowed to modify game while we're iterating through modifiers stored in game.
// Should probably store delvers seperately, game stores refs to it?
pub trait Modifier {
    // on_phase
    fn replace_event(&self, event:Event, _game:&Game, _queue:&mut EventQueue) -> Event {event}
    fn pre_event(&self, _event:&Event, _game:&Game, _queue:&mut EventQueue) {}
    fn post_event(&self, _event:&Event, _game:&Game, _queue:&mut EventQueue) {}
}

pub struct Outcomes { // DOESN'T WORK. Replace_event needs more immediacy.
    pub success:Vec<Event>,
    pub fail:Vec<Event>
}

pub struct Pheonix;
impl Modifier for Pheonix {
    fn replace_event(&self, event:Event, game:&Game, queue:&mut EventQueue) -> Event { // Could be allowed an immutable reference to the game.
        match event {
            Event::Death { delver_index } => {
                let outcomes = Outcomes{
                success: vec![event, Event::Log { message: String::from(game.delvers[delver_index as usize].to_string()) + " fails to defy death!" }],
                fail:vec![Event::Log { message: String::from(game.delvers[delver_index as usize].to_string()) + " defies death!" }, Event::Heal { delver_index: delver_index, amount: 5 }]
                };
                Event::Chance { chance:0.25, outcomes }
            },
            _ => event
        }
    }
}