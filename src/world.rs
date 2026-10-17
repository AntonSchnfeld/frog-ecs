use std::any::{TypeId};
use std::collections::HashMap;
use crate::entity::Entity;
use crate::sparseset::{ComponentStorage, SparseSet};
use crate::util::AsAny;

pub struct World {
    id_counter: u32,
    generation_table: Vec<u32>,
    free_ids: Vec<u32>,
    sparse_sets: HashMap<TypeId, Box<dyn ComponentStorage>>,
}

impl World {
    pub fn new() -> Self {
        World {
            id_counter: 0,
            generation_table: Vec::new(),
            free_ids: Vec::new(),
            sparse_sets: HashMap::new(),
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
            for set in self.sparse_sets.values_mut() {
                if set.contains(entity.id as usize) {
                    set.remove(entity.id as usize);
                }
            }
        }
        // index oob
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        if let Some(generation) = self.generation_table.get(entity.id as usize) {
            return entity.generation == *generation;
        }
        false
    }

    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        if !self.is_alive(entity) {
            return;
        }
        let type_id = TypeId::of::<T>();
        if self.sparse_sets.get(&type_id).is_none() {
            self.sparse_sets.insert(type_id, Box::new(SparseSet::<T>::new()));
        }
        let temp = self.sparse_sets.get_mut(&type_id).unwrap();
        let set = temp.as_any_mut().downcast_mut::<SparseSet<T>>().unwrap();
        set.set(entity.id as usize, component);
    }

    pub fn remove_component<T: 'static>(&mut self, entity: Entity) {
        if !self.is_alive(entity) {
            return;
        }
        let type_id = TypeId::of::<T>();
        if self.sparse_sets.get(&type_id).is_none() {
            return;
        }
        let set = self.sparse_sets.get_mut(&type_id).unwrap();
        set.remove(entity.id as usize);
    }

    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
        if !self.is_alive(entity) {
            return None;
        }
        let type_id = TypeId::of::<T>();
        let set = self.sparse_sets.get(&type_id)?.as_any().downcast_ref::<SparseSet<T>>().unwrap();
        set.get(entity.id as usize)
    }

    pub fn has_component<T: 'static>(&self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }
        let type_id = TypeId::of::<T>();
        if self.sparse_sets.get(&type_id).is_none() {
            return false;
        }
        let set = self.sparse_sets.get(&type_id).unwrap();
        set.contains(entity.id as usize)
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::Entity;
    use crate::world::World;
    use std::cell::Cell;
    use std::collections::HashSet;
    use std::rc::Rc;

    #[derive(Debug, Eq, PartialEq)]
    struct Position(i32);

    #[derive(Debug, Eq, PartialEq)]
    struct Velocity(i32);

    #[derive(Debug)]
    struct DropProbe(Rc<Cell<usize>>);

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

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

    #[test]
    fn given_alive_entity_when_add_component_then_component_is_available() {
        let mut world = World::new();
        let entity = world.create_entity();

        world.add_component(entity, Position(10));

        assert!(world.has_component::<Position>(entity));
        assert_eq!(world.get_component::<Position>(entity), Some(&Position(10)));
    }

    #[test]
    fn given_entity_with_component_when_add_component_then_value_is_replaced() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(entity, Position(10));

        world.add_component(entity, Position(20));

        assert_eq!(world.get_component::<Position>(entity), Some(&Position(20)));
    }

    #[test]
    fn given_two_entities_when_add_component_then_same_type_values_are_isolated() {
        let mut world = World::new();
        let first = world.create_entity();
        let second = world.create_entity();
        let without_component = world.create_entity();

        world.add_component(first, Position(10));
        world.add_component(second, Position(20));

        assert_eq!(world.get_component::<Position>(first), Some(&Position(10)));
        assert_eq!(world.get_component::<Position>(second), Some(&Position(20)));
        assert!(!world.has_component::<Position>(without_component));
        assert_eq!(world.get_component::<Position>(without_component), None);
    }

    #[test]
    fn given_one_entity_when_add_component_then_multiple_types_are_isolated() {
        let mut world = World::new();
        let entity = world.create_entity();

        world.add_component(entity, Position(10));
        world.add_component(entity, Velocity(20));

        assert_eq!(world.get_component::<Position>(entity), Some(&Position(10)));
        assert_eq!(world.get_component::<Velocity>(entity), Some(&Velocity(20)));
    }

    #[test]
    fn given_entity_with_component_when_remove_component_then_get_and_has_report_absent() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(entity, Position(10));

        world.remove_component::<Position>(entity);

        assert!(!world.has_component::<Position>(entity));
        assert_eq!(world.get_component::<Position>(entity), None);
    }

    #[test]
    fn given_live_entity_without_component_storage_when_remove_component_then_operation_is_a_no_op() {
        let mut world = World::new();
        let entity = world.create_entity();

        world.remove_component::<Position>(entity);

        assert!(!world.has_component::<Position>(entity));
        assert_eq!(world.get_component::<Position>(entity), None);
        assert!(world.is_alive(entity));
    }

    #[test]
    fn given_missing_component_type_when_remove_component_then_other_types_are_unchanged() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(entity, Position(10));

        world.remove_component::<Velocity>(entity);

        assert!(!world.has_component::<Velocity>(entity));
        assert_eq!(world.get_component::<Velocity>(entity), None);
        assert_eq!(world.get_component::<Position>(entity), Some(&Position(10)));
    }

    #[test]
    fn given_missing_component_in_existing_storage_when_remove_component_then_other_entity_is_preserved() {
        let mut world = World::new();
        let with_component = world.create_entity();
        let without_component = world.create_entity();
        world.add_component(with_component, Position(10));

        world.remove_component::<Position>(without_component);

        assert_eq!(
            world.get_component::<Position>(with_component),
            Some(&Position(10))
        );
        assert_eq!(world.get_component::<Position>(without_component), None);
    }

    #[test]
    fn given_entity_with_multiple_components_when_kill_entity_then_all_components_are_absent() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(entity, Position(10));
        world.add_component(entity, Velocity(20));

        world.kill_entity(entity);

        assert!(!world.has_component::<Position>(entity));
        assert!(!world.has_component::<Velocity>(entity));
        assert_eq!(world.get_component::<Position>(entity), None);
        assert_eq!(world.get_component::<Velocity>(entity), None);
    }

    #[test]
    fn given_two_entities_with_components_when_kill_entity_then_other_component_is_preserved() {
        let mut world = World::new();
        let killed = world.create_entity();
        let survivor = world.create_entity();
        world.add_component(killed, Position(10));
        world.add_component(survivor, Position(20));

        world.kill_entity(killed);

        assert_eq!(world.get_component::<Position>(killed), None);
        assert_eq!(
            world.get_component::<Position>(survivor),
            Some(&Position(20))
        );
    }

    #[test]
    fn given_killed_entity_when_add_component_then_component_is_rejected_and_dropped() {
        let drops = Rc::new(Cell::new(0));
        let mut world = World::new();
        let entity = world.create_entity();
        world.kill_entity(entity);

        world.add_component(entity, DropProbe(drops.clone()));

        assert!(!world.has_component::<DropProbe>(entity));
        assert_eq!(drops.get(), 1);
    }

    #[test]
    fn given_killed_entity_when_remove_component_then_world_is_unchanged() {
        let mut world = World::new();
        let killed = world.create_entity();
        world.add_component(killed, Position(10));
        world.kill_entity(killed);

        world.remove_component::<Position>(killed);

        assert!(!world.has_component::<Position>(killed));
        assert_eq!(world.get_component::<Position>(killed), None);
    }

    #[test]
    fn given_killed_entity_with_components_when_create_entity_then_recycled_entity_has_no_components() {
        let mut world = World::new();
        let old = world.create_entity();
        world.add_component(old, Position(10));
        world.kill_entity(old);

        let recycled = world.create_entity();

        assert_eq!(old.id, recycled.id);
        assert!(!world.has_component::<Position>(recycled));
        assert_eq!(world.get_component::<Position>(recycled), None);
    }

    #[test]
    fn given_recycled_entity_with_component_when_remove_component_then_old_handle_preserves_new_component() {
        let mut world = World::new();
        let old = world.create_entity();
        world.add_component(old, Position(10));
        world.kill_entity(old);
        let recycled = world.create_entity();
        world.add_component(recycled, Position(20));

        world.remove_component::<Position>(old);

        assert_eq!(
            world.get_component::<Position>(recycled),
            Some(&Position(20))
        );
    }

    #[test]
    fn given_existing_component_when_remove_component_then_value_is_dropped_immediately() {
        let drops = Rc::new(Cell::new(0));
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(entity, DropProbe(drops.clone()));

        world.remove_component::<DropProbe>(entity);

        assert_eq!(drops.get(), 1);
        drop(world);
        assert_eq!(drops.get(), 1);
    }

    #[test]
    fn given_existing_component_when_add_component_then_old_value_is_dropped_exactly_once() {
        let drops = Rc::new(Cell::new(0));
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(entity, DropProbe(drops.clone()));

        world.add_component(entity, DropProbe(drops.clone()));

        assert_eq!(drops.get(), 1);
        drop(world);
        assert_eq!(drops.get(), 2);
    }

    #[test]
    fn given_entity_with_components_when_kill_entity_then_all_values_are_dropped_immediately() {
        let drops = Rc::new(Cell::new(0));
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(entity, DropProbe(drops.clone()));

        world.kill_entity(entity);

        assert_eq!(drops.get(), 1);
        drop(world);
        assert_eq!(drops.get(), 1);
    }

    #[test]
    fn given_multiple_freed_slots_when_create_entities_then_each_slot_is_recycled_once() {
        let mut world = World::new();
        let first = world.create_entity();
        let second = world.create_entity();
        let third = world.create_entity();
        world.kill_entity(first);
        world.kill_entity(third);
        world.kill_entity(second);

        let recycled = [
            world.create_entity(),
            world.create_entity(),
            world.create_entity(),
        ];
        let recycled_ids: HashSet<_> = recycled.iter().map(|entity| entity.id).collect();

        assert_eq!(recycled_ids, HashSet::from([first.id, second.id, third.id]));
        assert!(recycled.iter().all(|entity| entity.generation == 1));
        assert!(recycled.iter().all(|entity| world.is_alive(*entity)));
        assert_eq!(world.create_entity().id, 3);
    }

    #[test]
    fn given_entity_handle_from_another_world_when_is_alive_then_return_false() {
        let mut first_world = World::new();
        let foreign_entity = first_world.create_entity();
        let mut second_world = World::new();
        let local_entity = second_world.create_entity();

        assert!(first_world.is_alive(foreign_entity));
        assert!(second_world.is_alive(local_entity));
        assert!(!second_world.is_alive(foreign_entity));
    }

    #[test]
    fn given_fabricated_handle_for_free_slot_when_is_alive_then_return_false() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.kill_entity(entity);
        let free_slot_handle = Entity::new(entity.id, entity.generation + 1);

        assert!(!world.is_alive(free_slot_handle));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn given_maximum_entity_id_when_create_entity_then_overflow_panics_in_debug_builds() {
        let mut world = World::new();
        world.id_counter = u32::MAX;

        let _ = world.create_entity();
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn given_maximum_generation_when_kill_entity_then_overflow_panics_in_debug_builds() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.generation_table[entity.id as usize] = u32::MAX;
        let last_generation = Entity::new(entity.id, u32::MAX);

        world.kill_entity(last_generation);
    }
}
