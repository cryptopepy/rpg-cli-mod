use super::{key::Key, Item};
use crate::game::Game;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Amulet;

impl Amulet {
    pub fn new() -> Self {
        Self {}
    }
}

impl fmt::Display for Amulet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "amulet")
    }
}

#[typetag::serde]
impl Item for Amulet {
    fn key(&self) -> Key {
        Key::Amulet
    }


    fn apply(&mut self, _game: &mut Game) {
        // The amulet's power is passive and checked in quests.
    }

    fn describe(&self) -> String {
        "A mysterious amulet that hums with ancient power.".to_string()
    }
}
