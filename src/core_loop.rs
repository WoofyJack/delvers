use rand::Rng;
use rand::seq::{SliceRandom, IteratorRandom};
use serde::{Serialize, Deserialize};

use crate::room_types::{Coordinate};
use crate::events::{Event, EventType, Outcomes};
use crate::entities::{Entity, Stats, Room};
use crate::sim::{Sim, roll};
use crate::messaging::Message;

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    NotStarted,
    TurnStart,
    Encounter,
    Delve,
    Finished,
    Combat {source:Entity, target:Entity}
}

pub fn tick(sim: &mut Sim, rng:&mut impl Rng) -> () {
    // let current_room: &Room = 
    // // match sim.game.rooms.get(&sim.game.delver_position) { //Checks for if off-map
    // //     Some(n) => n,
    // //     None => {sim.game.delver_position = Coordinate(0,0); return tick(sim, rng)} //TODO: Replace with some special case room, a la hall of flames.
    // // };

    match sim.game.phase {
        GamePhase::NotStarted => {
            sim.game.phase = GamePhase::Encounter;

            let message = Message::Delving;
            sim.eventqueue.log(message);
        }
        GamePhase::TurnStart => {
            if sim.game.defenderteam.active_defenders.len() > 0 {
                let defenders = 0..sim.game.defenderteam.active_defenders.len();
                let (source, target) = 
                if rng.gen_bool(0.5) { // Defender attacks
                    let active_delvers = sim.game.delverteam.active_delvers();
                    let target =  Entity::Delver{index:*active_delvers.choose(rng).unwrap()};
                    let source = sim.game.defenderteam.choose_defender(Stats::Fightiness);
                    (source, target)
                } else { // Delvers attack
                    let source = sim.game.delverteam.choose_delver(Stats::Fightiness);
                    let target = Entity::Defender {index:defenders.choose(rng).unwrap()};
                    (source, target)
                };
                sim.game.phase = GamePhase::Combat {source, target};
            } else {
                sim.game.phase = GamePhase::Encounter;
            }
        }
        GamePhase::Encounter => {
            if sim.game.current_room.complete {sim.game.phase = GamePhase::Delve;}
            else {
                sim.game.phase = GamePhase::TurnStart;
                let base_stat = sim.game.current_room.room_type.base_stat();
                let active_delver = sim.game.delverteam.choose_delver(base_stat);
                let room = Entity::Room;
                sim.game.current_room.room_type.attempt_clear(&sim.game, room, active_delver, &mut sim.eventqueue);
        }
        }
        GamePhase::Delve => {
            sim.game.phase = GamePhase::TurnStart;
            // Do Travel stuff
            let active_delver = sim.game.delverteam.choose_delver(Stats::Exploriness);

            // let position = sim.game.delver_position + Coordinate(1,0);

            let message = Message::Delve(active_delver);
            let success = vec![Event {event_type:EventType::Delve, source:active_delver, target:Entity::None, message}];
            
            let message = Message::FailedDelve (active_delver);
            let fail = vec![Event {event_type:EventType::Damage(1), target:active_delver, source:Entity::Dungeon, message}];
            
            let outcomes = Outcomes{success, fail};
            let message = Message::BeginNavigation(active_delver);
            let event = Event::type_and_message(EventType::Roll { difficulty: sim.game.defenderteam.dungeon.lengthiness, stat: Stats::Exploriness, outcomes}, message);
            sim.eventqueue.events.push(event);

        }
        GamePhase::Finished => {}
        GamePhase::Combat {source, target} => {
            sim.game.phase = GamePhase::TurnStart;

            let source_stat = source.collect_stats(&sim.game, Stats::Fightiness);
            let target_stat = target.collect_stats(&sim.game, Stats::Fightiness);

            let source_name = source.to_string(&sim.game);
            let target_name = target.to_string(&sim.game);
            

            if roll(rng, source_stat) > roll(rng, target_stat) { //  Attack succeeds
                let message = Message::Attack(source, target, 1);
                let event = Event {event_type:EventType::Damage(1), source, target, message };
                sim.eventqueue.events.push(event);
            } else { // Attack fails
                let message = Message::Attack(target, source, 1);
                let event: Event = Event {event_type:EventType::Damage(1), target:source, source:target, message };
                sim.eventqueue.events.push(event);
            }
            let message = Message::Custom(format!("{} attacks {}", source_name, target_name));
            sim.eventqueue.log(message);
        }
    }
}