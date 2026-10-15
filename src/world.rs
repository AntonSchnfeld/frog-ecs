use crate::entity::Entity;

pub struct World {
    entities: Vec<Entity>,
    id_counter: u32,
    generation_table: Vec<u32>
}

impl World {
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
            id_counter: 0,
            generation_table: Vec::new(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let id = self.id_counter;
        self.id_counter += 1;

        if id < u32::MAX {
            let entity = Entity::new(id, 0);
            self.entities.push(entity);
            self.generation_table.push(0);
            return entity;
        }

        todo!();
    }

    pub fn kill_entity(&mut self, entity: &Entity) {
        self.generation_table[entity.id as usize] += 1;
    }

    pub fn is_alive(&self, entity: &Entity) -> bool {
        self.generation_table[entity.id as usize] == entity.generation
    }
}

#[cfg(test)]
mod tests {
    use crate::world::World;

    #[test]
    fn given_new_entity_when_is_alive_then_return_true() {
        let mut world = World::new();
        let entity = world.create_entity();
        assert!(world.is_alive(&entity));
    }

    #[test]
    fn given_killed_entity_when_is_alive_then_return_false() {
        let mut world = World::new();
        let entity = world.create_entity();
        assert!(world.is_alive(&entity));
        world.kill_entity(&entity);
        assert!(!world.is_alive(&entity));
    }

    #[test]
    fn given_two_alive_entities_when_comparing_handles_then_unequal() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        assert!(world.is_alive(&a));
        assert!(world.is_alive(&b));
        assert_ne!(a, b);
    }

    #[test]
    fn given_two_alive_entities_when_killing_another_then_alive_entities_persist() {
        let mut world = World::new();
        let a = world.create_entity();
        let b = world.create_entity();
        let c = world.create_entity();

        assert!(world.is_alive(&a));
        assert!(world.is_alive(&b));
        assert!(world.is_alive(&c));

        world.kill_entity(&c);

        assert!(world.is_alive(&a));
        assert!(world.is_alive(&b));
        assert!(!world.is_alive(&c));
    }

    #[test]
    fn given_alive_entity_when_killing_and_creating_new_and_killing_old_handle_again_then_new_handle_persists() {
        let mut world = World::new();
        let old = world.create_entity();

        assert!(world.is_alive(&old));

        world.kill_entity(&old);

        assert!(!world.is_alive(&old));

        let new = world.create_entity();

        assert!(world.is_alive(&new));

        world.kill_entity(&old);

        assert!(!world.is_alive(&old));
        assert!(world.is_alive(&new));
    }
}
