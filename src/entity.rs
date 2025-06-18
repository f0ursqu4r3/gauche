use glam::{IVec2, Vec2};

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

    pub can_be_picked_up: bool,
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

    // rotation? rotational velocity? makes sense for some entities
    //  Jumping and Climbing
    pub jump_delay_frame_count: u32,
    pub jumping: bool,
    pub jumped_this_frame: bool,
    pub left_hanging: bool,
    pub right_hanging: bool,
    pub no_hang: bool,
    pub running: bool,
    pub holding: bool,
    pub climbing: bool,

    //  Player specific items
    pub money: u32,
    pub bombs: u32,
    pub ropes: u32,
    pub back_vid: Option<VID>,

    // TravelSound: TODO: probably should be its own system. kind of annoying for each entity to do this.
    pub travel_sound_countdown: f32,
    pub travel_sound: TravelSound,

    // StateMachine
    pub super_state: EntitySuperState,
    pub last_super_state: EntitySuperState,
    pub state: EntityState,

    pub health: u32,
    pub last_health: u32,
    pub hurt_on_contact: bool,
    pub attack_weight: f32,
    pub weight: f32,
    pub bomb_throw_delay_countdown: u32,
    pub rope_throw_delay_countdown: u32,
    pub attack_delay_countdown: u32,
    pub equip_delay_countdown: u32,
    pub thrown_by: Option<VID>,
    pub thrown_immunity_timer: u32,
    pub collided: bool,
    pub collided_last_frame: bool,
    pub collided_trigger_cooldown: u32,
    pub damage_vulnerability: DamageVulnerability,
    pub can_be_stunned: bool,

    pub point_a: IVec2,
    pub point_b: IVec2,
    pub point_c: IVec2,
    pub point_d: IVec2,
    pub point_label_a: PointLabel,
    pub point_label_b: PointLabel,
    pub point_label_c: PointLabel,
    pub point_label_d: PointLabel,

    pub holding_vid: Option<VID>,
    pub held_by_vid: Option<VID>,
    pub holding_timer: u32, //TODO look into removing holding_timer

    pub entity_a: Option<VID>,
    pub entity_b: Option<VID>,
    pub entity_c: Option<VID>,
    pub entity_d: Option<VID>,
    pub entity_label_a: EntityLabel,
    pub entity_label_b: EntityLabel,
    pub entity_label_c: EntityLabel,
    pub entity_label_d: EntityLabel,

    pub alignment: Alignment,

    pub counter_a: f32,
    pub counter_b: f32,
    pub counter_c: f32,
    pub counter_d: f32,

    pub threshold_a: f32,
    pub threshold_b: f32,
    pub threshold_c: f32,
    pub threshold_d: f32,
}

impl Entity {
    pub const HANGHAND_SIZE: Vec2 = Vec2::new(1.0, 4.0);

    pub fn new() -> Self {
        Self {
            //  Basic
            active: false,
            marked_for_destruction: false,
            type_: EntityType::None,
            vid: VID { id: 0, version: 0 },

            // Control Flags
            was_horizontally_controlled_this_frame: false,

            //  Physics
            has_physics: true,
            can_collide: true,
            grounded: false,
            coyote_time: 0,
            stun_timer: 0,
            can_be_picked_up: true,
            impassable: false,
            fall_distance: 0.0,

            //  Shape
            pos: Vec2::new(0.0, 0.0),
            vel: Vec2::new(0.0, 0.0),
            acc: Vec2::new(0.0, 0.0),
            size: Vec2::new(8.0, 8.0),
            dist_traveled_this_frame: 0.0,
            origin: Origin::Foot,

            // Rendering
            facing: LeftOrRight::Left,
            vertical_flip: false,
            draw_layer: DrawLayer::Middle,
            display_state: EntityDisplayState::Neutral,
            sprite_animator: SpriteAnimator::default(),

            //  Jumping and Climbing
            jump_delay_frame_count: JUMP_DELAY_FRAMES,
            jumping: false,
            jumped_this_frame: false,
            left_hanging: false,
            right_hanging: false,
            no_hang: false,
            running: false,
            holding: false,
            climbing: false,

            //  Player specific items
            money: 0,
            bombs: 4,
            ropes: 4,
            back_vid: None,

            // TravelSound
            travel_sound_countdown: TRAVEL_SOUND_DIST_INTERVAL,
            travel_sound: TravelSound::One,

            // StateMachine
            super_state: EntitySuperState::Idle,
            last_super_state: EntitySuperState::Idle,
            state: EntityState::Idle,

            health: 0,
            last_health: 0,
            hurt_on_contact: false,
            damage_vulnerability: DamageVulnerability::Vulnerable,
            attack_weight: 0.0,
            weight: 0.0,
            bomb_throw_delay_countdown: 0,
            rope_throw_delay_countdown: 0,
            attack_delay_countdown: 0,
            equip_delay_countdown: 0,
            thrown_by: None,
            thrown_immunity_timer: 0,
            collided: false,
            collided_last_frame: false,
            collided_trigger_cooldown: 0,
            can_be_stunned: false,

            point_a: IVec2::new(0, 0),
            point_b: IVec2::new(0, 0),
            point_c: IVec2::new(0, 0),
            point_d: IVec2::new(0, 0),

            point_label_a: PointLabel::None,
            point_label_b: PointLabel::None,
            point_label_c: PointLabel::None,
            point_label_d: PointLabel::None,

            holding_vid: None,
            held_by_vid: None,
            holding_timer: DEFAULT_HOLDING_TIMER,

            entity_a: None,
            entity_b: None,
            entity_c: None,
            entity_d: None,

            entity_label_a: EntityLabel::None,
            entity_label_b: EntityLabel::None,
            entity_label_c: EntityLabel::None,
            entity_label_d: EntityLabel::None,

            alignment: Alignment::Neutral,

            counter_a: 0.0,
            counter_b: 0.0,
            counter_c: 0.0,
            counter_d: 0.0,

            threshold_a: 0.0,
            threshold_b: 0.0,
            threshold_c: 0.0,
            threshold_d: 0.0,
        }
    }

    // TODO, try to get rid of reset...
    pub fn reset(&mut self) {
        let vid = self.vid;
        *self = Self::new();
        self.vid = vid;
        self.active = true;
    }

    /**
        Returns the top left and bottom right corners of the entity.

        Depends on the origin type:
            - if the origin is top left, the pos is the top left corner of the entity
            - if the origin is center, the pos is the center of the entity
            - if the origin is foot, the pos is the bottom center of the entity
    */
    pub fn get_bounds(&self) -> (Vec2, Vec2) {
        match self.origin {
            Origin::TopLeft => (self.pos, self.pos + self.size),
            Origin::Center => (self.pos - self.size / 2.0, self.pos + self.size / 2.0),
            Origin::Foot => (
                self.pos - Vec2::new(self.size.x / 2.0, self.size.y),
                self.pos,
            ),
        }
    }

    pub fn get_aabb(&self) -> AABB {
        match self.origin {
            Origin::TopLeft => AABB {
                tl: self.pos,
                br: self.pos + self.size,
            },
            Origin::Center => AABB {
                tl: self.pos - self.size / 2.0,
                br: self.pos + self.size / 2.0,
            },
            Origin::Foot => AABB {
                tl: self.pos - Vec2::new(self.size.x / 2.0, self.size.y),
                br: self.pos + Vec2::new(self.size.x / 2.0, 0.0),
            },
        }
    }

    pub fn get_center(&self) -> Vec2 {
        match self.origin {
            Origin::TopLeft => self.pos + self.size / 2.0,
            Origin::Center => self.pos,
            Origin::Foot => self.pos + Vec2::new(self.size.x / 2.0, 0.0),
        }
    }

    pub fn set_center(&mut self, center: Vec2) {
        match self.origin {
            Origin::TopLeft => self.pos = center - self.size / 2.0,
            Origin::Center => self.pos = center,
            Origin::Foot => self.pos = center - Vec2::new(self.size.x / 2.0, 0.0),
        }
    }

    pub fn inc_travel_sound(&mut self) {
        self.travel_sound = match self.travel_sound {
            TravelSound::One => TravelSound::Two,
            TravelSound::Two => TravelSound::One,
        }
    }

    pub fn is_hanging(&self) -> bool {
        self.left_hanging ^ self.right_hanging
    }

    pub fn is_horizontally_controlled(&self) -> bool {
        self.was_horizontally_controlled_this_frame
    }

    /**
       Returns the height of the entities feet.

       This happens to be the bottom of the entity, plus one pixel
    */
    pub fn get_feet(&self) -> AABB {
        match self.origin {
            Origin::TopLeft => AABB {
                tl: Vec2::new(self.pos.x, self.pos.y + self.size.y),
                br: Vec2::new(self.pos.x + self.size.x, self.pos.y + self.size.y + 1.0),
            },
            Origin::Center => AABB {
                tl: Vec2::new(
                    self.pos.x - self.size.x / 2.0,
                    self.pos.y + self.size.y / 2.0,
                ),
                br: Vec2::new(
                    self.pos.x + self.size.x / 2.0,
                    self.pos.y + self.size.y / 2.0 + 1.0,
                ),
            },
            Origin::Foot => AABB {
                tl: Vec2::new(self.pos.x - self.size.x / 2.0, self.pos.y),
                br: Vec2::new(self.pos.x + self.size.x / 2.0, self.pos.y + 1.0),
            },
        }
    }

    //** TODO: should be extracted out into a general entity thing? */
    pub fn set_grounded(&mut self, stage: &Stage) {
        // check just below player
        let feet = self.get_feet();
        if feet.br.y >= stage.get_height() as f32 {
            self.grounded |= true;
            return;
        }

        // get tiles in player bounds
        let feet_tl_tile_pos = feet.tl.as_ivec2() / Tile::SIZE as i32;
        let feet_br_tile_pos = feet.br.as_ivec2() / Tile::SIZE as i32;
        let tiles_at_feet = stage.get_tiles_in_rect(&feet_tl_tile_pos, &feet_br_tile_pos);
        let collided = collidable_tile_in_list(&tiles_at_feet);
        self.grounded |= collided;
    }

    pub fn get_tl_and_tr_corners(&self) -> (Vec2, Vec2) {
        match self.origin {
            Origin::TopLeft => (
                Vec2::new(self.pos.x, self.pos.y),
                Vec2::new(self.pos.x + self.size.x, self.pos.y),
            ),
            Origin::Center => (
                Vec2::new(
                    self.pos.x - self.size.x / 2.0,
                    self.pos.y - self.size.y / 2.0,
                ),
                Vec2::new(
                    self.pos.x + self.size.x / 2.0,
                    self.pos.y - self.size.y / 2.0,
                ),
            ),
            Origin::Foot => (
                Vec2::new(self.pos.x - self.size.x / 2.0, self.pos.y - self.size.y),
                Vec2::new(self.pos.x + self.size.x / 2.0, self.pos.y - self.size.y),
            ),
        }
    }

    /**
        returns the bounding corners of the left and right hang hands
        TODO: reimplement hanghands as line distance check
    */
    pub fn get_hang_hands(&self) -> HangHands {
        let (tl, tr) = self.get_tl_and_tr_corners();
        HangHands {
            left: tl,
            right: tr,
        }
    }
}

pub fn can_go_on_back(type_: EntityType) -> bool {
    match type_ {
        EntityType::JetPack => true,
        _ => false,
    }
}
