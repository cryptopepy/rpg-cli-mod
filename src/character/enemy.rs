use super::{class::Category, class::Class, Character};
use crate::item::ring::Ring;
use crate::location;
use crate::log;
use crate::randomizer::{random, Randomizer};
use rand::prelude::IteratorRandom;
use rand::Rng;

/// Randomly spawn an enemy character at the given location, based on the
/// current character stats.
/// The distance from home will influence the enemy frequency and level.
/// Under certain conditions, special (quest-related) enemies may be spawned.
pub fn spawn(game: &crate::game::Game) -> Option<Character> {
    let player = &game.player;
    let location = &game.location;

    if player.enemies_evaded() {
        return None;
    }

    let distance = location.distance_from_home();
    if random().should_enemy_appear(&distance) {
        let guardian_quest_unlocked = game.quests.list().iter().any(|(completed, description)| {
            !completed && description == "Defeat the Guardian."
        });

        let (class, level) = if guardian_quest_unlocked && distance.len() > 10 {
            (Class::player_by_name("guardian").unwrap().clone(), player.level + 5)
        } else {
            spawn_gorthaur(player, location)
                .or_else(|| spawn_shadow(player, location))
                .or_else(|| spawn_dev(player, location))
                .unwrap_or_else(|| spawn_random(player, &distance))
        };

        let level = random().enemy_level(level);
        let enemy = Character::new(class, level);
        log::enemy_appears(&enemy, location);
        Some(enemy)
    } else {
        None
    }
}

/// Final boss, only appears at level +100 when wearing the ruling ring
fn spawn_gorthaur(player: &Character, location: &location::Location) -> Option<(Class, i32)> {
    let wearing_ring =
        player.left_ring == Some(Ring::Ruling) || player.right_ring == Some(Ring::Ruling);

    if wearing_ring && location.distance_from_home().len() >= 100 {
        let mut class = Class::player_first().clone();
        class.name = String::from("gorthaur");
        class.hp.0 *= 2;
        class.strength.0 *= 2;
        class.category = Category::Legendary;
        Some((class, player.level))
    } else {
        None
    }
}

/// Player shadow, appears at home directory
fn spawn_shadow(player: &Character, location: &location::Location) -> Option<(Class, i32)> {
    let mut rng = rand::thread_rng();
    if location.is_home() && rng.gen_ratio(1, 10) {
        let mut class = player.class.clone();
        class.name = String::from("shadow");
        class.category = Category::Rare;
        Some((class, player.level + 3))
    } else {
        None
    }
}

/// Easter egg, appears at rpg data dir
fn spawn_dev(player: &Character, location: &location::Location) -> Option<(Class, i32)> {
    let mut rng = rand::thread_rng();

    if location.is_rpg_dir() && rng.gen_ratio(1, 10) {
        let mut class = Class::player_first().clone();
        class.name = String::from("dev");
        class.hp.0 /= 2;
        class.strength.0 /= 2;
        class.speed.0 /= 2;
        class.category = Category::Rare;
        Some((class, player.level))
    } else {
        None
    }
}

/// Choose an enemy randomly, with higher chance to difficult enemies the further from home.
fn spawn_random(player: &Character, distance: &location::Distance) -> (Class, i32) {
    let mut rng = rand::thread_rng();
    let enemies = Class::enemies();

    let mut enemy_groups: std::collections::HashMap<String, Vec<&Class>> =
        std::collections::HashMap::new();
    for enemy in enemies {
        let base_name = enemy.name.split(' ').next().unwrap().to_string();
        enemy_groups.entry(base_name).or_default().push(enemy);
    }

    let group_name = enemy_groups.keys().choose(&mut rng).unwrap();
    let enemy_group = &enemy_groups[group_name];

    let player_level = player.level;
    let enemy = enemy_group
        .iter()
        .filter(|e| {
            let level_req = match e.category {
                Category::Common => 1,
                Category::Rare => 5,
                Category::Legendary => 10,
                _ => 1,
            };
            player_level >= level_req
        })
        .max_by_key(|e| e.hp.0)
        .unwrap_or(&enemy_group[0]);

    let level = std::cmp::max(player.level / 10 + distance.len() - 1, 1);
    ((*enemy).clone(), level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enemy_level() {
        let mut player = Character::player();
        let d1 = location::Distance::from(1);
        let d2 = location::Distance::from(2);
        let d3 = location::Distance::from(3);
        let d10 = location::Distance::from(10);

        assert_eq!(1, spawn_random(&player, &d1).1);
        assert_eq!(1, spawn_random(&player, &d2).1);
        assert_eq!(2, spawn_random(&player, &d3).1);
        assert_eq!(9, spawn_random(&player, &d10).1);

        player.level = 5;
        assert_eq!(1, spawn_random(&player, &d1).1);
        assert_eq!(1, spawn_random(&player, &d2).1);
        assert_eq!(2, spawn_random(&player, &d3).1);
        assert_eq!(9, spawn_random(&player, &d10).1);

        player.level = 10;
        assert_eq!(1, spawn_random(&player, &d1).1);
        assert_eq!(2, spawn_random(&player, &d2).1);
        assert_eq!(3, spawn_random(&player, &d3).1);
        assert_eq!(10, spawn_random(&player, &d10).1);
    }

    #[test]
    fn test_run_ring() {
        let mut player = Character::player();
        let location = location::tests::location_from("~/1/");
        assert!(spawn(&location, &player).is_some());

        player.equip_ring(Ring::Evade);
        assert!(spawn(&location, &player).is_none());

        player.equip_ring(Ring::Void);
        assert!(spawn(&location, &player).is_none());

        player.equip_ring(Ring::Void);
        assert!(spawn(&location, &player).is_some());
    }
}
