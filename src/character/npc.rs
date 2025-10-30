use crate::game::Game;
use crate::log;
use crate::randomizer::{random, Randomizer};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum Encounter {
    Gambler,
    Witch,
    GhostlyMaiden,
}

pub fn spawn(game: &mut Game) {
    if random().should_enemy_appear(&game.location.distance_from_home()) {
        let encounter = match random().range(3) {
            0 => Some(Encounter::Gambler),
            1 => Some(Encounter::Witch),
            2 => Some(Encounter::GhostlyMaiden),
            _ => None,
        };

        if let Some(encounter) = encounter {
            game.in_encounter = Some(encounter.clone());
            log::npc_encounter(&encounter);
        }
    }
}
