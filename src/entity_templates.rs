use glam::Vec2;
use rand::random_range;

use crate::{
    audio::SoundEffect,
    entity::{Alignment, DamageVulnerability, Entity, EntityType, Mood},
    item::{Item, ItemType},
    sprite::Sprite,
};

pub fn init_as_player(entity: &mut Entity) {
    entity.active = true;
    entity.type_ = EntityType::Player;
    entity.sprite = Some(Sprite::Player);
    entity.impassable = true;
    entity.alignment = Alignment::Player;
    entity.move_cooldown = 0.12;
    entity.health = 100;
    entity.max_hp = 100;
    entity.size = Vec2::new(1.0, 1.0); // Player is larger than other entities

    let mut wall_item = Item::new(ItemType::Wall);
    wall_item.count = 99; // Start with 99 walls
    entity.inventory.insert(wall_item);

    let fist_item = Item::new(ItemType::Fist);
    entity.inventory.insert(fist_item);

    let mut medkit_item = Item::new(ItemType::Medkit);
    medkit_item.count = medkit_item.max_count;
    entity.inventory.insert(medkit_item);

    // give the player some bandage
    let mut bandage_item = Item::new(ItemType::Bandage);
    bandage_item.count = 5.max(bandage_item.max_count); // Start with 5 bandages
    entity.inventory.insert(bandage_item);

    // give the player some bandaid
    let mut bandaid_item = Item::new(ItemType::Bandaid);
    bandaid_item.count = 20.max(bandaid_item.max_count); // Start with 20 bandaids
    entity.inventory.insert(bandaid_item);

    // give a conductor hat
    let conductor_hat_item = Item::new(ItemType::ConductorHat);
    entity.inventory.insert(conductor_hat_item);
}

pub fn init_as_zombie(entity: &mut Entity) {
    entity.active = true;
    entity.type_ = EntityType::Zombie;
    entity.sprite = Some(Sprite::Zombie);
    entity.impassable = true;
    entity.alignment = Alignment::Enemy;
    entity.mood = crate::entity::Mood::Wander;
    entity.move_cooldown = 0.8;
    entity.attack_cooldown = 1.0;
    entity.health = 40;
    entity.max_hp = 40;
    // randomize move cooldown timer in range
    entity.move_cooldown_countdown = rand::random::<f32>() * entity.move_cooldown;
    // randomize step sound, 1 or 2
    crate::entity::randomize_step_sound(entity);

    let growl_sound = if random_range(0..2) == 0 {
        SoundEffect::ZombieGrowl1
    } else {
        SoundEffect::ZombieGrowl2
    };
    entity.growl = Some(growl_sound);
}

pub fn init_as_chicken(entity: &mut Entity) {
    entity.active = true;
    entity.type_ = EntityType::Chicken;

    enum ChickenType {
        Chick,
        Hen,
        Rooster,
    }

    let chicken_type = if rand::random::<f32>() < 0.5 {
        ChickenType::Chick
    } else if rand::random::<f32>() < 0.5 {
        ChickenType::Hen
    } else {
        ChickenType::Rooster
    };

    entity.impassable = true;
    entity.alignment = Alignment::Neutral;
    entity.mood = Mood::Wander;

    match chicken_type {
        ChickenType::Chick => {
            entity.sprite = Some(Sprite::Chick);
            entity.move_cooldown = 0.3; // Faster movement for chicks
            entity.size = Vec2::new(0.5, 0.5); // Smaller size for chicks
            entity.health = 1; // Less health for chicks
            entity.max_hp = 1;
            entity.growl = Some(SoundEffect::Chick);
        }
        ChickenType::Hen => {
            entity.sprite = Some(Sprite::Hen);
            entity.move_cooldown = 0.5; // Normal movement for hens
            entity.health = 3; // More health for hens
            entity.max_hp = 3;
            entity.growl = Some(SoundEffect::Hen);
        }
        ChickenType::Rooster => {
            entity.sprite = Some(Sprite::Rooster);
            entity.move_cooldown = 0.7; // Slower movement for roosters
            entity.health = 30; // More health for roosters
            entity.max_hp = 30;
            entity.growl = Some(SoundEffect::Rooster);
        }
    }

    // randomize move cooldown timer in range
    entity.move_cooldown_countdown = rand::random::<f32>() * entity.move_cooldown;

    // randomize step sound, 1 or 2
    crate::entity::randomize_step_sound(entity);
}

// init as rail_layer
// this is a special entity that zips across the stage and places rails
pub fn init_as_rail_layer(entity: &mut Entity) {
    println!("init_as_rail_layer");
    entity.active = true;
    entity.type_ = EntityType::RailLayer;
    entity.sprite = None;
    entity.impassable = false;
    entity.alignment = Alignment::Neutral;
    entity.mood = Mood::Idle;
    entity.move_cooldown = 0.01; // Faster movement for rail layer
    entity.move_cooldown_countdown = entity.move_cooldown;
    entity.health = 10000000;
    entity.max_hp = 10000000;
    entity.damage_vulnerability = DamageVulnerability::Immune;
}

// init as train
pub fn init_as_train(entity: &mut Entity) {
    println!("init_as_train");
    entity.active = true;
    entity.type_ = EntityType::Train;
    entity.sprite = Some(Sprite::TrainHead);
    entity.impassable = true;
    entity.alignment = Alignment::Neutral;
    entity.mood = Mood::Idle;
    entity.move_cooldown = 0.02; // Faster movement for train
    entity.move_cooldown_countdown = entity.move_cooldown;
    entity.health = 10000000;
    entity.max_hp = 10000000;
    entity.damage_vulnerability = DamageVulnerability::Immune;
    entity.size = Vec2::new(2.0, 2.0); // Train is larger than other entities
}
