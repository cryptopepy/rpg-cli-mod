use super::{Event, Quest};
use crate::item::key::Key;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FindAmulet {
    finished: bool,
}

impl FindAmulet {
    pub fn new() -> Self {
        Self { finished: false }
    }
}

#[typetag::serde]
impl Quest for FindAmulet {
    fn description(&self) -> String {
        "Find the Amulet of Power.".to_string()
    }

    fn handle(&mut self, event: &Event) -> bool {
        if let Event::ItemAdded { item } = event {
            if *item == Key::Amulet {
                self.finished = true;
            }
        }
        self.finished
    }
}
