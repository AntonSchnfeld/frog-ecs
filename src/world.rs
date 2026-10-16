use crate::entity::Entity;

pub struct World {
    id_counter: u32,
    generation_table: Vec<u32>,
    free_ids: Vec<u32>,
}

impl World {
    pub fn new() -> Self {
        World {
            id_counter: 0,
            generation_table: Vec::new(),
            free_ids: Vec::new(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        // recycle ids if some are leftover
        if !self.free_ids.is_empty() {
            // exists fosho
            let reused_id = self.free_ids.pop().unwrap();
            return Entity {
                id: reused_id,
                // has to be safe since id is recycled
                generation: self.generation_table[reused_id as usize],
            }
        }
        // to create new entity we must
        // a) get new id
        // b) create its generation table entry
        let new_id = self.id_counter;
        self.id_counter += 1;
        self.generation_table.push(0);
        Entity {
            id: new_id,
            generation: 0,
        }
    }

    pub fn kill_entity(&mut self, entity: Entity) {
        // ya cant kill what's already dead
        if !self.is_alive(entity) {
            return;
        }
        if let Some(generation) = self.generation_table.get_mut(entity.id as usize) {
            // recycle id
            self.free_ids.push(entity.id);
            *generation += 1;
        }
        // index oob
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        if let Some(generation) = self.generation_table.get(entity.id as usize) {
            return entity.generation == *generation;
        }
        false
    }

    pub fn add_component<T>(&self, entity: Entity, _component: T) {
        if !self.is_alive(entity) {
            return;
        }
        todo!()
    }

    pub fn remove_component<T>(&self, entity: Entity, _component: T) {
        if !self.is_alive(entity) {
            return;
        }
        todo!()
    }

    pub fn get_component<T>(&self, entity: Entity) -> Option<T> {
        if !self.is_alive(entity) {
            return None;
        }
        todo!()
    }

    pub fn has_component<T>(&self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::Entity;
    use crate::world::World;

    #[test]
    fn given_entity_with_out_of_bounds_id_when_is_alive_then_return_false() {
        let world = World::new();
        // fake entity
        let fentity = Entity::new(1000, 0);
        assert!(!world.is_alive(fentity));
    }

    #[test]
    fn given_entity_with_out_of_bounds_generation_when_is_alive_then_return_false() {
        let mut world = World::new();
        // create entity to make id 0 valid
        world.create_entity();
        let fentity = Entity::new(0, 1000);
        assert!(!world.is_alive(fentity));
    }

    #[test]
    fn given_entity_with_out_of_bounds_generation_when_kill_entity_then_not_kill_valid_entity() {
        let mut world = World::new();
        let entity = world.create_entity();
        assert!(world.is_alive(entity));
        let fentity = Entity::new(0, 1000);
        world.kill_entity(fentity);
        assert!(world.is_alive(entity));
    }

    #[test]
    fn given_recyclable_id_when_create_entity_then_id_is_recycled() {
        let mut world = World::new();
        let entity = world.create_entity();
        assert!(world.is_alive(entity));
        world.kill_entity(entity);
        assert!(!world.is_alive(entity));
        let recycled_entity = world.create_entity();
        assert!(world.is_alive(recycled_entity));
        assert_eq!(recycled_entity.id, entity.id);
        assert_ne!(recycled_entity.generation, entity.generation);
    }

    #[test]
    fn given_old_generation_on_recycled_id_slot_when_is_alive_then_return_false() {
        let mut world = World::new();
        let old_entity = world.create_entity();
        assert!(world.is_alive(old_entity));
        world.kill_entity(old_entity);
        assert!(!world.is_alive(old_entity));
        let recycled_entity = world.create_entity();
        assert!(!world.is_alive(old_entity));
        assert!(world.is_alive(recycled_entity));
        assert_eq!(recycled_entity.generation, old_entity.generation + 1);
    }

    #[test]
    fn given_new_entity_when_is_alive_then_return_true() {
        let mut world = World::new();
        let entity = world.create_entity();
        assert!(world.is_alive(entity));
    }

    #[test]
    fn given_killed_entity_when_is_alive_then_return_false() {
        let mut world = World::new();
        let entity = world.create_entity();
        assert!(world.is_alive(entity));
        world.kill_entity(entity);
        assert!(!world.is_alive(entity));
    }

    #[test]
    fn given_two_alive_entities_when_comparing_handles_then_unequal() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        assert!(world.is_alive(a));
        assert!(world.is_alive(b));
        assert_ne!(a, b);
    }

    #[test]
    fn given_two_alive_entities_when_killing_another_then_alive_entities_persist() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let c = world.create_entity();

        assert!(world.is_alive(a));
        assert!(world.is_alive(b));
        assert!(world.is_alive(c));

        world.kill_entity(c);

        assert!(world.is_alive(a));
        assert!(world.is_alive(b));
        assert!(!world.is_alive(c));
    }

    #[test]
    fn given_alive_entity_when_killing_and_creating_new_and_killing_old_handle_again_then_new_handle_persists() {
        let mut world = World::new();
        let old = world.create_entity();

        assert!(world.is_alive(old));

        world.kill_entity(old);

        assert!(!world.is_alive(old));

        let new = world.create_entity();

        assert!(world.is_alive(new));

        world.kill_entity(old);

        assert!(!world.is_alive(old));
        assert!(world.is_alive(new));
    }

    #[test]
    fn given_twice_killed_entity_when_create_entity_then_slot_is_properly_recycled() {
        let mut world = World::new();
        let old_entity = world.create_entity();
        assert!(world.is_alive(old_entity));
        world.kill_entity(old_entity);
        assert!(!world.is_alive(old_entity));
        world.kill_entity(old_entity);
        assert!(!world.is_alive(old_entity));
        let recycled_entity = world.create_entity();
        assert_eq!(recycled_entity.generation, old_entity.generation + 1);
        assert_eq!(recycled_entity.id, old_entity.id);
    }
}
