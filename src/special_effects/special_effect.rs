use glam::{IVec2, Vec2};

pub trait SpecialEffect {
    fn step(&mut self);
    fn is_finished(&self) -> bool;

    fn get_pos(&self) -> Vec2;
    fn get_size(&self) -> Vec2;
    fn get_rot(&self) -> f32;
    fn get_counter(&self) -> u32;
    fn get_type(&self) -> SpecialEffectType;
    fn get_alpha(&self) -> f32;
    fn get_sample_region(&self) -> &'static SampleRegion;
}

/** put this at the top of your special effect definitions */
#[macro_export]
macro_rules! special_effect_getters {
    () => {
        fn get_pos(&self) -> Vec2 {
            self.pos
        }
        fn get_size(&self) -> Vec2 {
            self.size
        }
        fn get_rot(&self) -> f32 {
            self.rot
        }
        fn get_counter(&self) -> u32 {
            self.counter
        }
        fn get_type(&self) -> SpecialEffectType {
            self.type_
        }
        fn get_alpha(&self) -> f32 {
            self.alpha
        }
        fn is_finished(&self) -> bool {
            self.counter <= 0
        }

        fn get_sample_region(&self) -> &'static SampleRegion {
            get_sample_region(self.get_type(), self.get_counter())
        }
    };
}

#[derive(Clone, Copy)]
pub enum SpecialEffectType {
    GrenadeBoom,
    BasicSmoke,
    SimpleSpark,
    Pow,
    BloodBall,
    LittleBrownShard,
}

pub enum SpecialEffectSampleRegionName {
    BigExplosion,
    LittleExplosion,
    Spark,
    Pow,
    LittleSmoke,
    BigSmoke,
    ExplosionFrame1,
    ExplosionFrame2,
    ExplosionFrame3,
    ExplosionFrame4,
    BloodBall,
    LittleBrownShard,
}

pub struct SampleRegion {
    pub pos: IVec2,
    pub size: IVec2,
}

pub fn get_sample_region(
    special_effect_type: SpecialEffectType,
    counter: u32,
) -> &'static SampleRegion {
    let region_name = get_special_effect_sample_region_name(special_effect_type, counter);
    get_special_effect_sample_region_from_name(region_name)
}

pub fn get_special_effect_sample_region_name(
    special_effect_type: SpecialEffectType,
    counter: u32,
) -> SpecialEffectSampleRegionName {
    match special_effect_type {
        SpecialEffectType::GrenadeBoom => match counter {
            6..=7 => SpecialEffectSampleRegionName::ExplosionFrame1,
            4..=5 => SpecialEffectSampleRegionName::ExplosionFrame2,
            2..=3 => SpecialEffectSampleRegionName::ExplosionFrame3,
            _ => SpecialEffectSampleRegionName::ExplosionFrame4,
        },
        SpecialEffectType::BasicSmoke => SpecialEffectSampleRegionName::BigSmoke,
        SpecialEffectType::SimpleSpark => SpecialEffectSampleRegionName::Spark,
        SpecialEffectType::Pow => SpecialEffectSampleRegionName::Pow,
        SpecialEffectType::BloodBall => SpecialEffectSampleRegionName::BloodBall,
        SpecialEffectType::LittleBrownShard => SpecialEffectSampleRegionName::LittleBrownShard,
    }
}

pub fn get_special_effect_sample_region_from_name(
    name: SpecialEffectSampleRegionName,
) -> &'static SampleRegion {
    match name {
        SpecialEffectSampleRegionName::BigExplosion => &SampleRegion {
            pos: IVec2 { x: 0, y: 0 },
            size: IVec2 { x: 45, y: 41 },
        },
        SpecialEffectSampleRegionName::LittleExplosion => &SampleRegion {
            pos: IVec2 { x: 60, y: 0 },
            size: IVec2 { x: 10, y: 8 },
        },
        SpecialEffectSampleRegionName::Spark => &SampleRegion {
            pos: IVec2 { x: 63, y: 22 },
            size: IVec2 { x: 7, y: 12 },
        },
        SpecialEffectSampleRegionName::Pow => &SampleRegion {
            pos: IVec2 { x: 61, y: 12 },
            size: IVec2 { x: 9, y: 7 },
        },
        SpecialEffectSampleRegionName::LittleSmoke => &SampleRegion {
            pos: IVec2 { x: 71, y: 0 },
            size: IVec2 { x: 10, y: 10 },
        },
        SpecialEffectSampleRegionName::BigSmoke => &SampleRegion {
            pos: IVec2 { x: 0, y: 269 },
            size: IVec2 { x: 65, y: 61 },
        },
        SpecialEffectSampleRegionName::ExplosionFrame1 => &SampleRegion {
            pos: IVec2 { x: 0, y: 164 },
            size: IVec2 { x: 45, y: 42 },
        },
        SpecialEffectSampleRegionName::ExplosionFrame2 => &SampleRegion {
            pos: IVec2 { x: 0, y: 40 },
            size: IVec2 { x: 62, y: 60 },
        },
        SpecialEffectSampleRegionName::ExplosionFrame3 => &SampleRegion {
            pos: IVec2 { x: 0, y: 101 },
            size: IVec2 { x: 61, y: 62 },
        },
        SpecialEffectSampleRegionName::ExplosionFrame4 => &SampleRegion {
            pos: IVec2 { x: 0, y: 206 },
            size: IVec2 { x: 65, y: 61 },
        },
        SpecialEffectSampleRegionName::BloodBall => &SampleRegion {
            pos: IVec2 { x: 63, y: 38 },
            size: IVec2 { x: 16, y: 17 },
        },
        SpecialEffectSampleRegionName::LittleBrownShard => &SampleRegion {
            pos: IVec2 { x: 89, y: 5 },
            size: IVec2 { x: 2, y: 1 },
        },
    }
}
