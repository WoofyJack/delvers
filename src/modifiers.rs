
use crate::sim::{Event, Game, roll, EventQueue};


// Jumping through hoops cus we aren't allowed to modify game while we're iterating through modifiers stored in game.
// Should probably store delvers seperately, game stores refs to it?
pub trait Modifier {
    // on_phase
    fn replace_event(&self, event:Event, game:&Game, queue:&mut EventQueue) -> Event; 
    fn pre_event(&self, event:&Event, game:&Game, queue:&mut EventQueue);
    fn post_event(&self, event:&Event, game:&Game, queue:&mut EventQueue);
}


pub struct Pheonix;
impl Modifier for Pheonix {
    fn pre_event(&self, _event:&Event, _game:&Game, _queue:&mut EventQueue) {}
    fn replace_event(&self, event:Event, game:&Game, queue:&mut EventQueue) -> Event { // Could be allowed an immutable reference to the game.
        match event {
            Event::Death { delver_index } => {
                if false{// roll(rng, game.delvers[delver_index as usize].base.exploriness) > 0.25 {
                    queue.events.push(event);
                    Event::Log { message: String::from(game.delvers[delver_index as usize].to_string()) + " fails to defy death!" }
                }
                else {
                    queue.events.push(Event::Damage { delver_index: delver_index, amount: -5 });
                    Event::Log { message: String::from(game.delvers[delver_index as usize].to_string()) + " defies death!" }
                }
            },
            _ => event
        }
    }
    fn post_event(&self, _event:&Event, _game:&Game, _queue:&mut EventQueue) {}
}