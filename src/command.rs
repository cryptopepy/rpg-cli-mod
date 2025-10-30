use crate::character;
use crate::character::enemy;
use crate::game::Game;
use crate::item;
use crate::item::key::Key;
use crate::location::Location;
use crate::log;
use crate::randomizer::Randomizer;
use anyhow::{anyhow, bail, Result};

use clap::Parser;

#[derive(Parser)]
#[command()]
pub enum Command {
    /// Display stats for the given items. Defaults to displaying hero stats if no item is specified. [default]
    #[command(aliases=&["s", "status"], display_order=0)]
    Stat { items: Vec<String> },

    /// Moves the hero to the supplied destination, potentially initiating battles along the way.
    #[command(name = "cd", display_order = 1)]
    ChangeDir {
        /// Directory to move to.
        #[arg(default_value = "~")]
        destination: String,

        /// Move the hero's to a different location without spawning enemies.
        /// Intended for scripts and shell integration.
        #[arg(short, long)]
        force: bool,
    },

    /// Inspect the directory contents, possibly finding treasure chests and hero tombstones.
    #[command(name = "ls", display_order = 1)]
    Inspect,

    /// Buys an item from the shop.
    /// If name is omitted lists the items available for sale.
    #[command(alias = "b", display_order = 2)]
    Buy { items: Vec<String> },

    /// Uses an item from the inventory.
    #[command(alias = "u", display_order = 3)]
    Use { items: Vec<String> },

    /// Prints the quest todo list.
    #[command(alias = "t", display_order = 4)]
    Todo,

    /// Resets the current game.
    Reset {
        /// Reset data files, losing cross-hero progress.
        #[arg(long)]
        hard: bool,
    },

    /// Change the character class.
    /// If name is omitted lists the available character classes.
    Class { name: Option<String> },

    /// Prints the hero's current location
    #[command(name = "pwd")]
    PrintWorkDir,

    /// Attack the enemy in the current location
    #[command(alias = "a")]
    Attack,

    /// Attempt to flee from the enemy
    Flee,

    /// Attempt to bribe the enemy
    Bribe,

    /// List available skills
    Skills,

    /// Learn a skill
    Learn {
        #[arg(required = true)]
        skill_name: String,
    },

    /// Use a skill
    UseSkill {
        #[arg(required = true)]
        skill_name: String,
    },

    /// Bet gold on a coin flip
    Bet {
        #[arg(required = true)]
        amount: i32,
    },

    /// Ask the witch to brew a potion
    Brew,

    /// Listen to the ghostly maiden's story
    Listen,

    /// Potentially initiates a battle in the hero's current location.
    Battle,

    /// Save the current game
    #[command(display_order = 5)]
    Save,

    /// Load the game
    #[command(display_order = 6)]
    Load,

    /// Set hardcore mode
    #[command(display_order = 7)]
    Hardcore { on: bool },


    #[command(hide = true)]
    Idkfa { level: i32 },
}

pub fn run(cmd: Option<Command>, game: &mut Game) -> Result<bool> {
    let mut save = true;
    match cmd.unwrap_or(Command::Stat { items: vec![] }) {
        Command::Stat { items } => stat(game, &items)?,
        Command::ChangeDir {
            destination,
            force,
        } => change_dir(game, &destination, force)?,
        Command::Inspect => game.inspect(),
        Command::Class { name } => class(game, &name)?,
        Command::Battle => battle(game)?,
        Command::PrintWorkDir => println!("{}", game.location.path_string()),
        Command::Reset { .. } => game.reset(),
        Command::Buy { items } => shop(game, &items)?,
        Command::Use { items } => use_item(game, &items)?,
        Command::Todo => {
            log::quest_list(game.quests.list());
        }
        Command::Save => save_game(game)?,
        Command::Load => {
            load_game(game)?;
            save = false;
        }
        Command::Hardcore { on } => set_hardcore(game, on)?,
        Command::Attack => attack(game)?,
        Command::Flee => flee(game)?,
        Command::Bribe => bribe(game)?,
        Command::Skills => skills(game)?,
        Command::Learn { skill_name } => learn(game, &skill_name)?,
        Command::UseSkill { skill_name } => use_skill(game, &skill_name)?,
        Command::Bet { amount } => bet(game, amount)?,
        Command::Brew => brew(game)?,
        Command::Listen => listen(game)?,
        Command::Idkfa { level } => debug_command(game, level),
    };

    Ok(save)
}

fn bet(game: &mut Game, amount: i32) -> Result<()> {
    if let Some(character::npc::Encounter::Gambler) = &game.in_encounter {
        if amount > game.gold {
            bail!("You don't have that much gold to bet.");
        }
        if crate::randomizer::random().range(2) == 0 {
            println!("You won! You double your bet.");
            game.gold += amount;
        } else {
            println!("You lost! You lose your bet.");
            game.gold -= amount;
        }
        game.in_encounter = None;
    } else {
        bail!("There is no one to bet with here.");
    }
    Ok(())
}

fn brew(game: &mut Game) -> Result<()> {
    if let Some(character::npc::Encounter::Witch) = &game.in_encounter {
        println!("The witch brews a bubbling potion and hands it to you.");
        let potion = crate::item::Potion::new(game.player.level);
        game.add_item(Box::new(potion));
        game.in_encounter = None;
    } else {
        bail!("There is no witch here to brew a potion.");
    }
    Ok(())
}

fn listen(game: &mut Game) -> Result<()> {
    if let Some(character::npc::Encounter::GhostlyMaiden) = &game.in_encounter {
        let lore = match crate::randomizer::random().range(3) {
            0 => "She whispers of a hidden treasure in a nearby cave.",
            1 => "She speaks of a great evil that slumbers deep within the earth.",
            2 => "She warns of a powerful dragon that guards the mountain pass.",
            _ => unreachable!(),
        };
        println!("The ghostly maiden's voice echoes in your mind: '{}'", lore);
        game.in_encounter = None;
    } else {
        bail!("There is no one to listen to here.");
    }
    Ok(())
}

fn skills(game: &mut Game) -> Result<()> {
    log::skill_list(&game.player);
    Ok(())
}

fn learn(game: &mut Game, skill_name: &str) -> Result<()> {
    game.player.learn_skill(skill_name)?;
    println!("Skill '{}' learned.", skill_name);
    Ok(())
}

fn use_skill(game: &mut Game, skill_name: &str) -> Result<()> {
    if let Err(err) = game.use_skill(skill_name) {
        if err.downcast_ref::<character::Dead>().is_some() {
            game.reset();
            bail!("");
        }
        return Err(err);
    }
    Ok(())
}


fn attack(game: &mut Game) -> Result<()> {
    if let Err(err) = game.battle_round() {
        if err.downcast_ref::<character::Dead>().is_some() {
            game.reset();
            bail!("");
        }
        return Err(err);
    }
    Ok(())
}

fn flee(game: &mut Game) -> Result<()> {
    if let Err(err) = game.player_flee() {
        if err.downcast_ref::<character::Dead>().is_some() {
            game.reset();
            bail!("");
        }
        return Err(err);
    }
    Ok(())
}

fn bribe(game: &mut Game) -> Result<()> {
    if let Err(err) = game.player_bribe() {
        if err.downcast_ref::<character::Dead>().is_some() {
            game.reset();
            bail!("");
        }
        return Err(err);
    }
    Ok(())
}

fn save_game(game: &Game) -> Result<()> {
    crate::datafile::save(game)?;
    println!("Game saved.");
    Ok(())
}

fn load_game(game: &mut Game) -> Result<()> {
    if let Some(loaded_game) = crate::datafile::load()? {
        *game = loaded_game;
        println!("Game loaded.");
    } else {
        bail!("No saved game found.");
    }
    Ok(())
}

fn set_hardcore(game: &mut Game, on: bool) -> Result<()> {
    game.hardcore = on;
    if on {
        println!("Hardcore mode enabled.");
    } else {
        println!("Hardcore mode disabled.");
    }
    Ok(())
}

/// Attempt to move the hero to the supplied location, possibly engaging
/// in combat along the way.
fn change_dir(game: &mut Game, dest: &str, force: bool) -> Result<()> {
    let dest = Location::from(dest)?;
    let result = if force {
        // When change is force, skip enemies along the way
        // but still apply side-effects at destination
        game.visit(dest)
    } else {
        game.go_to(&dest)
    };

    if let Err(err) = result {
        if err.downcast_ref::<character::Dead>().is_some() {
            game.reset();
            bail!("");
        }
        return Err(err.into());
    }

    Ok(())
}

/// Potentially run a battle at the current location, independently from
/// the hero's movement.
fn battle(game: &mut Game) -> Result<()> {
    if game.in_combat.is_some() {
        bail!("Already in combat.");
    }
    if let Some(enemy) = enemy::spawn(game) {
        log::enemy_appears(&enemy, &game.location);
        game.in_combat = Some(enemy);
    } else {
        println!("No enemies found here.");
    }
    Ok(())
}

/// Set the class for the player character
fn class(game: &mut Game, class_name: &Option<String>) -> Result<()> {
    if !game.location.is_home() {
        bail!("Class change is only allowed at home.")
    }

    if let Some(class_name) = class_name {
        let class_name = class_name.to_lowercase();
        game.player
            .change_class(&class_name)
            .map_err(|_| anyhow!("Unknown class name."))
    } else {
        let player_classes: Vec<String> =
            character::class::Class::names(character::class::Category::Player)
                .iter()
                .cloned()
                .collect();
        println!("Options: {}", player_classes.join(", "));
        Ok(())
    }
}

/// Buy an item from the shop or list the available items if no item name is provided.
/// Shopping is only allowed when the player is at the home directory.
fn shop(game: &mut Game, items: &[String]) -> Result<()> {
    if items.is_empty() {
        item::shop::list(game)
    } else {
        // parse items and break if any is invalid/unknown
        let mut keys = Vec::new();
        for item in items {
            keys.push(Key::from(item)?);
        }

        item::shop::buy(game, &keys)
    }
}

fn stat(game: &mut Game, items: &[String]) -> Result<()> {
    if items.is_empty() {
        log::status(game);
        Ok(())
    } else {
        for item_name in items {
            let item_name = Key::from(item_name)?;
            let (display, description) = game.describe(item_name)?;
            println!("{}: {}", display, description);
        }
        Ok(())
    }
}

/// Use an item from the inventory or list the inventory contents if no item name is provided.
fn use_item(game: &mut Game, items: &[String]) -> Result<()> {
    if items.is_empty() {
        println!("{}", log::format_inventory(game));
    } else {
        for item_name in items {
            let item_name = Key::from(item_name)?;
            game.use_item(item_name)?
        }
    }
    Ok(())
}

fn debug_command(game: &mut Game, level: i32) {
    game.reset();
    game.gold = 5000 * level;
    for _ in 1..level {
        game.player.add_experience(game.player.xp_for_next());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_dir_battle() {
        let mut game = Game::new();
        let cmd = Command::ChangeDir {
            destination: "~/..".to_string(),
            run: false,
            bribe: false,
            force: false,
        };

        // increase level to ensure win
        for _ in 0..5 {
            game.player.add_experience(game.player.xp_for_next());
        }

        let result = run(Some(cmd), &mut game);

        assert!(result.is_ok());
        assert!(game.player.xp > 0);
        assert!(game.gold > 0);
    }

    #[test]
    fn change_dir_dead() {
        let mut game = Game::new();
        let cmd = Command::ChangeDir {
            destination: "~/..".to_string(),
            run: false,
            bribe: false,
            force: false,
        };

        // reduce stats to ensure loss
        let weak_class = character::class::Class {
            hp: character::class::Stat(1, 1),
            speed: character::class::Stat(1, 1),
            ..game.player.class
        };
        game.player = character::Character::new(weak_class, 1);
        game.gold = 100;
        game.player.xp = 100;

        let result = run(Some(cmd), &mut game);

        assert!(result.is_err());

        // game reset
        assert_eq!(game.player.max_hp(), game.player.current_hp);
        assert_eq!(0, game.gold);
        assert_eq!(0, game.player.xp);
        assert!(!game.tombstones.is_empty());
    }

    #[test]
    fn status_effect_dead() {
        let mut game = Game::new();

        // using force prevents battle but effects should apply anyway
        let cmd = Command::ChangeDir {
            destination: "~/..".to_string(),
            run: false,
            bribe: false,
            force: true,
        };

        // reduce stats to ensure loss
        let weak_class = character::class::Class {
            hp: character::class::Stat(1, 1),
            speed: character::class::Stat(1, 1),
            ..game.player.class
        };
        game.player = character::Character::new(weak_class, 1);
        game.player.status_effect = Some(character::StatusEffect::Burn);
        game.gold = 100;
        game.player.xp = 100;

        let result = run(Some(cmd), &mut game);

        assert!(result.is_err());

        // game reset
        assert_eq!(game.player.max_hp(), game.player.current_hp);
        assert_eq!(0, game.gold);
        assert_eq!(0, game.player.xp);
        assert!(!game.tombstones.is_empty());
    }

    #[test]
    fn change_dir_home() {
        let mut game = Game::new();

        assert!(game.location.is_home());

        // force move to a non home location
        let cmd = Command::ChangeDir {
            destination: "~/..".to_string(),
            run: false,
            bribe: false,
            force: true,
        };

        let result = run(Some(cmd), &mut game);
        assert!(result.is_ok());
        assert!(!game.location.is_home());

        game.player.current_hp = 1;

        // back home (without forcing)
        let cmd = Command::ChangeDir {
            destination: "~".to_string(),
            run: false,
            bribe: false,
            force: false,
        };

        let result = run(Some(cmd), &mut game);
        assert!(result.is_ok());
        assert!(game.location.is_home());
        assert_eq!(game.player.max_hp(), game.player.current_hp);
    }

    #[test]
    fn change_dir_home_force() {
        let mut game = Game::new();

        assert!(game.location.is_home());

        // force move to a non home location
        let cmd = Command::ChangeDir {
            destination: "~/..".to_string(),
            run: false,
            bribe: false,
            force: true,
        };

        let result = run(Some(cmd), &mut game);
        assert!(result.is_ok());
        assert!(!game.location.is_home());

        game.player.current_hp = 1;

        // force back home should restore hp
        let cmd = Command::ChangeDir {
            destination: "~".to_string(),
            run: false,
            bribe: false,
            force: true,
        };

        let result = run(Some(cmd), &mut game);
        assert!(result.is_ok());
        assert!(game.location.is_home());
        assert_eq!(game.player.max_hp(), game.player.current_hp);
    }

    #[test]
    fn inspect_tombstone() {
        // die at non home with some gold
        let mut game = Game::new();
        assert!(game.tombstones.is_empty());

        let cmd = Command::ChangeDir {
            destination: "~/..".to_string(),
            run: false,
            bribe: false,
            force: false,
        };

        // reduce stats to ensure loss
        game.player.current_hp = 1;

        game.gold = 100;
        assert!(run(Some(cmd), &mut game).is_err());

        assert_eq!(0, game.gold);
        assert!(!game.tombstones.is_empty());

        // force move to the previous dead location
        let cmd = Command::ChangeDir {
            destination: "~/..".to_string(),
            run: false,
            bribe: false,
            force: true,
        };
        run(Some(cmd), &mut game).unwrap();

        // inspect to pick up lost gold
        let cmd = Command::Inspect;
        let result = run(Some(cmd), &mut game);
        assert!(result.is_ok());
        assert!(game.tombstones.is_empty());

        // includes +200g for visit tombstone quest
        assert_eq!(300, game.gold);
    }

    #[test]
    fn buy_use_item() {
        let mut game = Game::new();
        assert!(game.inventory().is_empty());

        // not buy if not enough money
        let cmd = Command::Buy {
            items: vec![String::from("potion")],
        };
        let result = run(Some(cmd), &mut game);
        assert!(result.is_err());
        assert!(game.inventory().is_empty());

        // buy potion
        game.gold = 200;
        let cmd = Command::Buy {
            items: vec![String::from("potion")],
        };
        let result = run(Some(cmd), &mut game);
        assert!(result.is_ok());
        assert!(!game.inventory().is_empty());
        assert_eq!(0, game.gold);

        // use potion
        game.player.current_hp -= 1;
        let cmd = Command::Use {
            items: vec![String::from("potion")],
        };
        let result = run(Some(cmd), &mut game);
        assert!(result.is_ok());
        assert!(game.inventory().is_empty());
        assert_eq!(game.player.max_hp(), game.player.current_hp);

        // not buy if not home
        let cmd = Command::ChangeDir {
            destination: "~/..".to_string(),
            run: false,
            bribe: false,
            force: true,
        };
        run(Some(cmd), &mut game).unwrap();

        game.gold = 200;
        let cmd = Command::Buy {
            items: vec![String::from("potion")],
        };
        let result = run(Some(cmd), &mut game);
        assert!(result.is_err());
        assert!(game.inventory().is_empty());
    }
}
