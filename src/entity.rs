use glam::{IVec2, Vec2};

use crate::{
    audio::Audio,
    sprite::Sprite,
    state::State,
    step::entity_step_sound_lookup,
    tile::{self, is_tile_occupied},
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EntityType {
    None,
    Player,
    Zombie,
}

/** these are the low level current actions of the entity */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityState {
    Idle,
    Walking,
    Dead,
}

/** Use for entity state machine, marking intention on stored locations.*/
#[derive(Debug)]
pub enum PointLabel {
    None,
    Target,
    GoingHere,
    Boundary,
    Avoid,
}

/** Use for entity state machine, for filtering attacks so they dont hit neutral enemies or only hit allys.*/
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Alignment {
    Player,
    Neutral,
    Enemy,
}

/** Use for entity state machine, marking intention on stored entities.*/
#[derive(Debug)]
pub enum EntityLabel {
    None,
    AttackThis,
    GetThis,
    AvoidThis,
    BeNearThis,
    GoToThis,
    AttachedToThis,
}

/** the entities have to have these set so they get rendered in the correct order */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawLayer {
    Background,
    Middle,
    Foreground,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DamageType {
    Attack,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DamageVulnerability {
    Immune,
    NotImmune,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VID {
    pub id: usize,
    pub version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepSound {
    Step1,
    Step2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mood {
    Idle,
    Wander,
    Noticing,
    ChasingTarget,
    LosingTarget,
}

#[derive(Debug)]
pub struct Entity {
    //  Basic
    pub active: bool,
    pub marked_for_destruction: bool,
    pub type_: EntityType,
    pub vid: VID,

    pub impassable: bool, // does entity block other entities

    //  Shape
    pub pos: Vec2,
    pub vel: Vec2,
    pub acc: Vec2,
    pub size: Vec2,
    pub dist_traveled_this_frame: f32,
    pub rot: f32,
    pub shake: f32,

    //  Rendering
    pub draw_layer: DrawLayer,
    pub sprite: Sprite,

    // StateMachine
    pub state: EntityState,

    pub health: u32,
    pub damage_vulnerability: DamageVulnerability,
    pub can_be_stunned: bool,
    pub move_cooldown: f32,
    pub move_cooldown_countdown: f32,

    pub alignment: Alignment,

    pub counter_a: f32,
    pub threshold_a: f32,
    pub mood: Mood,
    pub target_pos: Option<Vec2>,
    pub target_entity: Option<VID>,

    pub step_sound: StepSound,
    pub detection_radius: f32,

    pub attack_cooldown: f32,
    pub attack_cooldown_countdown: f32,

    pub inventory: Vec<InvEntry>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            //  Basic
            active: false,
            marked_for_destruction: false,
            type_: EntityType::None,
            vid: VID { id: 0, version: 0 },
            impassable: false,
            damage_vulnerability: DamageVulnerability::NotImmune,

            //  Shape
            pos: Vec2::new(0.0, 0.0),
            vel: Vec2::new(0.0, 0.0),
            acc: Vec2::new(0.0, 0.0),
            size: Vec2::new(8.0, 8.0),
            dist_traveled_this_frame: 0.0,
            rot: 0.0,
            shake: 0.0,

            // Rendering
            draw_layer: DrawLayer::Middle,
            sprite: Sprite::NoSprite,

            // StateMachine
            state: EntityState::Idle,

            health: 0,
            can_be_stunned: false,
            move_cooldown: 0.0,
            move_cooldown_countdown: 0.0,

            alignment: Alignment::Neutral,

            counter_a: 0.0,
            threshold_a: 0.0,
            mood: Mood::Idle,
            target_pos: None,

            step_sound: StepSound::Step1,
            target_entity: None,

            detection_radius: 16.0, // Default detection radius
            attack_cooldown: 0.0,
            attack_cooldown_countdown: 0.0,

            inventory: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        let vid = self.vid;
        *self = Self::new();
        self.vid = vid;
        self.active = true;
    }

    pub fn get_tl_and_br_corners(&self) -> (Vec2, Vec2) {
        // origin is always center
        let half_size = self.size / 2.0;
        let top_left = Vec2::new(self.pos.x - half_size.x, self.pos.y - half_size.y);
        let bottom_right = Vec2::new(self.pos.x + half_size.x, self.pos.y + half_size.y);
        (top_left, bottom_right)
    }
}

pub fn swap_step_sound(entity: &mut Entity) {
    entity.step_sound = match entity.step_sound {
        StepSound::Step1 => StepSound::Step2,
        StepSound::Step2 => StepSound::Step1,
    };
}

pub fn randomize_step_sound(entity: &mut Entity) {
    entity.step_sound = if rand::random::<bool>() {
        StepSound::Step1
    } else {
        StepSound::Step2
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvEntry {
    pub index: usize,
    pub item: Item,
    pub amount: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Item {
    Wall,
    Medkit,
}

pub struct ItemInfo {
    pub name: &'static str,
    pub description: &'static str,
}

pub fn item_info(item: Item) -> ItemInfo {
    match item {
        Item::Wall => ItemInfo {
            name: "Wall",
            description: "A solid wall that blocks movement.",
        },
        Item::Medkit => ItemInfo {
            name: "Medkit",
            description: "A medkit that restores health.",
        },
    }
}
