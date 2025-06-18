use glam::Vec2;

use crate::sprite::Sprite;

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
    Ally,
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
        }
    }

    // TODO, try to get rid of reset...
    pub fn reset(&mut self) {
        let vid = self.vid;
        *self = Self::new();
        self.vid = vid;
        self.active = true;
    }

    pub fn get_tl_and_tr_corners(&self) -> (Vec2, Vec2) {
        // origin is always center
        let half_width = self.size.x / 2.0;
        let tl = Vec2::new(self.pos.x - half_width, self.pos.y - self.size.y / 2.0);
        let tr = Vec2::new(self.pos.x + half_width, self.pos.y - self.size.y / 2.0);
        (tl, tr)
    }
}
