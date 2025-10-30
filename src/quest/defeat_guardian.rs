use super::{Event, Quest};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DefeatGuardian {
    finished: bool,
}

impl DefeatGuardian {
    pub fn new() -> Self {
        Self { finished: false }
    }
}

#[typetag::serde]
impl Quest for DefeatGuardian {
    fn description(&self) -> String {
        "Defeat the Guardian.".to_string()
    }

    fn handle(&mut self, event: &Event) -> bool {
        if let Event::BattleWon { enemy, .. } = event {
            if enemy.name() == "guardian" {
                self.finished = true;
            }
        }
        self.finished
    }
}
