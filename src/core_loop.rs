use rand::Rng;

use crate::locations::{Coordinate, Room};
use crate::teams::{DelverStats, DefenderStats};
use crate::events::{Entity, Event, EventType, Outcomes};
use crate::sim::{Sim, roll};


#[derive(PartialEq, Eq)]
pub enum GamePhase {
    NotStarted,
    TurnStart,
    Encounter,
    Forge,
    Travel,
    Finished,
    Combat {target:Entity}
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
        }
        GamePhase::TurnStart => {
            if sim.game.boss_fight_started {
                sim.game.phase = GamePhase::Combat { target: Entity::Defender}
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
                let room_position = sim.game.delver_position;
                current_room.room_type.attempt_clear(&sim.game, room_position, active_delver, &mut sim.eventqueue);

        }
        }
        GamePhase::Forge => {
            sim.game.phase = GamePhase::Travel;
            //Do forging stuff
        }
        GamePhase::Travel => {
            sim.game.phase = GamePhase::TurnStart;
            // Do Travel stuff
            let active_delver = sim.game.delverteam.choose_delver(DelverStats::Exploriness);

            let position = sim.game.delver_position + Coordinate(1,0);

            let message = active_delver.to_string(&sim.game) + " guides the delvers to " + &position.to_string();
            let success = Event::comment_event(EventType::Move {position}.target(Entity::None, active_delver), message);
            
            let message = active_delver.to_string(&sim.game) + " hurts themselves while navigating.";
            let fail = Event::comment_event(EventType::Damage {amount: 1}.target(Entity::None, active_delver), message);
            
            let outcomes = Outcomes{success, fail};
            let event = EventType::Roll { difficulty: sim.game.defenderteam.dungeon.lengthiness, stat: DelverStats::Exploriness, outcomes}.no_target_no_source();
            sim.eventqueue.events.push(event);

            let message = active_delver.to_string(&sim.game) + " begins searching for a way forward.";
            sim.eventqueue.events.push(EventType::Log { message }.no_target_no_source());
        }
        GamePhase::Finished => {}
        GamePhase::Combat {target} => {
            let defender = &sim.game.defenderteam.defender;
            if !defender.active {
                sim.game.rooms.get_mut(&sim.game.delver_position).unwrap().complete = true;
                sim.game.boss_fight_started = false;
                sim.game.phase = GamePhase::TurnStart;
            }
            else {
            let (active_delver, message) = match target {
                Entity::Defender => {
                    let active_delvers = sim.game.delverteam.active_delvers();
                    let index =  active_delvers[rng.gen_range(0..active_delvers.len())];
                    let target = Entity::Delver { index };

                    sim.game.phase = GamePhase::Combat { target };

                    let active_delver = sim.game.delverteam.choose_delver(DelverStats::Fightiness);
                    (active_delver, active_delver.to_string(&sim.game) + " challenges the dungeon's defender, " + &defender.to_string())
                }
                Entity::Delver { index } => {
                    sim.game.phase = GamePhase::TurnStart;
                    let active_delver = Entity::Delver { index };
                    (active_delver, defender.to_string() + " attacks " + &active_delver.to_string(&sim.game))
                }
                _ => panic!("Invalid combat")
            };
            if roll(rng, active_delver.get_delver_stat(&sim.game, DelverStats::Fightiness)) > roll(rng, defender.get_stat(DefenderStats::Fightiness)) {
                let message = active_delver.to_string(&sim.game) + " injures " + &defender.to_string();// + " leaving them on " + &(defender.hp-1).to_string() + " hp";
                sim.eventqueue.events.push(EventType::Log { message }.no_target_no_source());
                sim.eventqueue.events.push (EventType::Damage { amount: 1 }.target(active_delver, Entity::Defender))
            } else {
                if true {//active_delver.hp > 1 {
                    let message = defender.to_string() + " injures " + &active_delver.to_string(&sim.game);
                    sim.eventqueue.events.push(EventType::Log { message }.no_target_no_source());
                }
                sim.eventqueue.events.push (EventType::Damage { amount: 1 }.target(Entity::Defender, active_delver));
            }
            sim.eventqueue.events.push(EventType::Log { message }.no_target_no_source());
        }
        }
    }
}