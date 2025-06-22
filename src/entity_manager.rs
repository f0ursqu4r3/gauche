/*
    Entity Manager
    The entity manager is used to interact with entities.
    It is used to add entities to the game, and to update them, and get values from them.
    Nobody should store refs to entities, they have to fetch them from the entity manager.

*/

use std::println;

use crate::entity::{Entity, EntityType, VID};

pub struct EntityManager {
    pub entities: Vec<Entity>,
    pub available_ids: Vec<usize>,
}

impl EntityManager {
    pub const MAX_NUM_ENTITIES: usize = 1024; //128;
    pub fn new() -> Self {
        let mut entities = Vec::with_capacity(Self::MAX_NUM_ENTITIES);
        let mut available_ids = Vec::with_capacity(Self::MAX_NUM_ENTITIES);
        for i in 0..Self::MAX_NUM_ENTITIES {
            let mut new_entity = Entity::new();
            new_entity.vid.id = i;
            entities.push(new_entity);
            // available_ids.push(i as u32);
            available_ids.insert(0, i);
        }
        Self {
            entities,
            available_ids,
        }
    }

    pub fn new_entity(&mut self) -> Option<VID> {
        if let Some(id) = self.available_ids.pop() {
            self.entities[id].active = true;
            self.entities[id].vid.version += 1;
            return Some(self.entities[id].vid);
        }
        // TODO: actual warning queue needed
        println!("Entity budget bounce!");
        None
    }

    pub fn set_inactive(&mut self, entity_id: usize) {
        self.entities[entity_id].active = false;
        self.available_ids.insert(0, entity_id);
    }

    pub fn set_inactive_vid(&mut self, vid: VID) {
        let entity = &self.entities[vid.id];
        if vid.version == entity.vid.version && entity.active {
            self.set_inactive(vid.id);
        }
    }

    pub fn set_entity_inactive(&mut self, entity: &mut Entity) {
        entity.active = false;
        self.available_ids.insert(0, entity.vid.id);
    }

    /** dude just get the vid from the entity, wtf are you doing */
    pub fn get_vid(&self, id: usize) -> VID {
        self.entities[id].vid
    }

    pub fn get_entity_by_id(&self, id: usize) -> &Entity {
        &self.entities[id]
    }

    pub fn get_entity(&self, vid: VID) -> Option<&Entity> {
        let entity = &self.entities[vid.id];
        if vid.version == entity.vid.version && entity.active {
            return Some(entity);
        }
        None
    }

    pub fn get_entity_mut(&mut self, vid: VID) -> Option<&mut Entity> {
        let entity = &mut self.entities[vid.id];
        if entity.active && vid.version == entity.vid.version {
            return Some(entity);
        }
        None
    }

    pub fn _get_entities(&mut self) -> &mut Vec<Entity> {
        &mut self.entities
    }

    pub fn num_entities(&self) -> usize {
        self.entities.len()
    }

    /** This is a very expensive function. Don't call it a lot... */
    pub fn num_active_entities(&self) -> u32 {
        let mut count = 0;
        for entity in self.iter() {
            if entity.active {
                count += 1;
            }
        }
        count
    }

    pub fn iter(&self) -> std::slice::Iter<Entity> {
        self.entities.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Entity> {
        self.entities.iter_mut()
    }

    /// Iter all vids of active entities, collect
    pub fn get_active_vids(&self) -> Vec<VID> {
        self.entities
            .iter()
            .filter(|e| e.active)
            .map(|e| e.vid)
            .collect()
    }

    pub fn clear_all_entities(&mut self) {
        self.available_ids.clear();
        for i in 0..Self::MAX_NUM_ENTITIES {
            self.available_ids.insert(0, i);
            self.entities[i].active = false;
            self.entities[i].type_ = EntityType::None;
        }
    }

    pub fn clear_all_non_player_entities(&mut self) {
        self.available_ids.clear();
        for i in 0..Self::MAX_NUM_ENTITIES {
            if self.entities[i].type_ != EntityType::Player {
                self.available_ids.insert(0, i);
                self.entities[i].active = false;
                self.entities[i].type_ = EntityType::None;
            }
        }
    }
}
