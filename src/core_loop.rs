use rand::Rng;
use serde::Serialize;

use crate::locations::{Coordinate, Room};
use crate::teams::{Stats};
use crate::events::{Entity, Event, EventType, Outcomes};
use crate::sim::{Sim, roll};


#[derive(PartialEq, Eq, Serialize)]
pub enum GamePhase {
    NotStarted,
    TurnStart,
    Encounter,
    Forge,
    Travel,
    Finished,
    Combat {source:Entity, target:Entity}
}

pub fn tick(sim: &mut Sim, rng:&mut impl Rng) -> () {
    let current_room: &Room = 
    match sim.game.rooms.get(&sim.game.delver_position) { //Checks for if off-map
        Some(n) => n,
        None => {sim.game.delver_position = Coordinate(0,0); return tick(sim, rng)} //TODO: Replace with some special case room, a la hall of flames.
    };

    match sim.game.phase {
        GamePhase::NotStarted => {
            sim.game.phase = GamePhase::Encounter;

            let message = sim.game.delverteam.to_string() + " are delving into " + &sim.game.defenderteam.to_string() +"'s dungeon " + &sim.game.defenderteam.dungeon.to_string();
            sim.eventqueue.log(message);
        }
        GamePhase::TurnStart => {
            if sim.game.defenderteam.active_defenders.len() > 0 {
                sim.game.phase = GamePhase::Combat {source:sim.game.delverteam.choose_delver(Stats::Fightiness), target: Entity::Defender {index:0}}
            } else {
                sim.game.phase = GamePhase::Encounter;
            }
        }
        GamePhase::Encounter => {
            if current_room.complete {sim.game.phase = GamePhase::Forge;}
            else {
                sim.game.phase = GamePhase::TurnStart;
                // Do encounter rolls
                // DO IMMEDIATE FIX
                let base_stat = current_room.room_type.base_stat();
                let active_delver = sim.game.delverteam.choose_delver(base_stat);
                let room = Entity::Room { index: sim.game.delver_position};
                current_room.room_type.attempt_clear(&sim.game, room, active_delver, &mut sim.eventqueue);
        }
        }
        GamePhase::Forge => {
            sim.game.phase = GamePhase::Travel;
            //Do forging stuff
        }
        GamePhase::Travel => {
            sim.game.phase = GamePhase::TurnStart;
            // Do Travel stuff
            let active_delver = sim.game.delverteam.choose_delver(Stats::Exploriness);

            let position = sim.game.delver_position + Coordinate(1,0);

            let message = active_delver.to_string(&sim.game) + " guides the delvers to " + &position.to_string();
            let success = Event::comment_event(EventType::Move {position}.target(Entity::None, active_delver), message);
            
            let message = active_delver.to_string(&sim.game) + " hurts themselves while navigating.";
            let fail = Event::comment_event(EventType::Damage {amount: 1}.target(Entity::None, active_delver), message);
            
            let outcomes = Outcomes{success, fail};
            let event = EventType::Roll { difficulty: sim.game.defenderteam.dungeon.lengthiness, stat: Stats::Exploriness, outcomes}.no_target_no_source();
            sim.eventqueue.events.push(event);

            let message = active_delver.to_string(&sim.game) + " begins searching for a way forward.";
            sim.eventqueue.events.push(EventType::Log { message }.no_target_no_source());
        }
        GamePhase::Finished => {}
        GamePhase::Combat {source, target} => {
            let source_stat = source.get_stat(&sim.game, Stats::Fightiness);
            let target_stat = target.get_stat(&sim.game, Stats::Fightiness);

            let source_name = source.to_string(&sim.game);
            let target_name = target.to_string(&sim.game);
            
            let message = format!("{} attacks {}", source_name, target_name);

            if roll(rng, source_stat) > roll(rng, target_stat) { //  Attack succeeds
                let message = format!("{} injures {}", source_name, target_name);
                sim.eventqueue.events.push(EventType::Log { message }.no_target_no_source());
                sim.eventqueue.events.push (EventType::Damage { amount: 1 }.target(source, target))
            } else { // Attack fails
                let message = format!("{} injures {}", target_name, source_name);
                sim.eventqueue.events.push(EventType::Log { message }.no_target_no_source());
                sim.eventqueue.events.push (EventType::Damage { amount: 1 }.target(target, source))
            }
            sim.eventqueue.events.push(EventType::Log { message }.no_target_no_source());
            sim.game.phase = GamePhase::Combat { source: target, target: source };
        }
    }
}