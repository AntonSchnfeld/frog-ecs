use crate::util::AsAny;

// TODO: Think of better names than id_mapper and mapped_id, horrible for rubber ducking
// TODO: Think of if SparseSet should be pub or pub(crate) or in a util library
#[derive(Debug)]
pub struct SparseSet<T> {
    // index = id, output = idx of component storage
    sparse: Vec<usize>,
    dense_data: Vec<T>,
    dense_indices: Vec<usize>,
}

impl<T> SparseSet<T> {
    pub fn new() -> Self {
        SparseSet {
            sparse: Vec::new(),
            dense_data: Vec::new(),
            dense_indices: Vec::new(),
        }
    }

    pub fn get(&self, id: usize) -> Option<&T> {
        let dense_idx = self.sparse.get(id)?;
        self.dense_data.get(*dense_idx)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        let dense_idx = self.sparse.get(id)?;
        self.dense_data.get_mut(*dense_idx)
    }

    pub fn set(&mut self, id: usize, component: T) {
        if let Some(mapped_id) = self.sparse.get(id) {
            // idx exists already, no need to resize
            if *mapped_id != usize::MAX {
                // we write since theres already a dense slot allocated
                self.dense_data[*mapped_id] = component;
                self.dense_indices[*mapped_id] = id;
                return;
            }
        } else {
            // we need to resize since the idx doesnt exist
            self.sparse.resize(id + 1, usize::MAX);
        }
        // We know that
        // a) the sparse array supports at least id + 1 elements
        // b) the idx at id is NOT currently mapped
        self.dense_data.push(component);
        self.dense_indices.push(id);
        self.sparse[id] = self.dense_data.len() - 1;
    }
}

pub trait ComponentStorage: AsAny {
    fn contains(&self, id: usize) -> bool;
    fn remove(&mut self, id: usize);
}

impl<T: 'static> ComponentStorage for SparseSet<T> {
    fn contains(&self, id: usize) -> bool {
        if let Some(idx) = self.sparse.get(id) && *idx != usize::MAX {
            return true;
        }
        false
    }

    fn remove(&mut self, id: usize) {
        // dont gotta do anything if theres nothing to be done
        if !self.contains(id) {
            return;
        }
        // where is the dense value at
        let dense_idx = self.sparse[id];
        // mark sparse spot as empty
        self.sparse[id] = usize::MAX;
        // remove actual data, replace empty spot with last element in dense array
        self.dense_data.swap_remove(dense_idx);
        // State now:
        // 1) Data is removed from storage
        // 2) Sparse Mapping to removed Data is gone too
        // 3) Last dense element was moved to id
        // 4) Sparse entry for previously last element now points to garbage
        // 5) dense_indices entry for removed data still exists
        // 6) dense_indices entry for moved element is now wrong
        // TO FIX:
        // Step 1) Get sparse idx for moved (previously last) element
        let sparse_idx = self.dense_indices[self.dense_data.len()];
        // Step 2) Remove dense_indices entry for deleted data
        self.dense_indices.swap_remove(dense_idx);
        // Step 3) Point sparse entry to new position IF there even is still data in the first place
        if !self.dense_data.is_empty() {
            self.sparse[sparse_idx] = dense_idx;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::sparseset::{ComponentStorage, SparseSet};
    use std::cell::Cell;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[derive(Debug)]
    struct DropProbe(Rc<Cell<usize>>);

    #[derive(Debug, Eq, PartialEq)]
    struct Marker;

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    #[test]
    fn given_new_set_when_get_then_return_none_for_any_id() {
        let set: SparseSet<String> = SparseSet::new();

        for id in [0, 1, 100, u32::MAX as usize] {
            assert_eq!(set.get(id), None);
        }
    }

    #[test]
    fn given_new_set_when_contains_then_return_false_for_any_id() {
        let set: SparseSet<String> = SparseSet::new();

        for id in [0, 1, 100, u32::MAX as usize] {
            assert!(!set.contains(id));
        }
    }

    #[test]
    fn given_empty_set_when_set_then_store_value_and_leave_holes_empty() {
        let mut set = SparseSet::new();
        set.set(10_000, String::from("component"));

        assert_eq!(set.get(10_000).map(String::as_str), Some("component"));
        for id in [0, 1, 9_999] {
            assert!(!set.contains(id));
            assert_eq!(set.get(id), None);
        }
    }

    #[test]
    fn given_existing_id_when_set_then_replace_value() {
        let mut set = SparseSet::new();
        set.set(0, String::from("old"));
        set.set(0, String::from("new"));

        assert!(set.contains(0));
        assert_eq!(set.get(0).map(String::as_str), Some("new"));
    }

    #[test]
    fn given_two_distinct_ids_when_set_then_preserve_both_values() {
        let mut set = SparseSet::new();
        set.set(0, String::from("zero"));
        set.set(1, String::from("one"));

        assert_eq!(set.get(0).map(String::as_str), Some("zero"));
        assert_eq!(set.get(1).map(String::as_str), Some("one"));
    }

    #[test]
    fn given_zero_sized_components_when_set_then_track_components_independently() {
        let mut set = SparseSet::new();
        set.set(0, Marker);
        set.set(3, Marker);

        set.remove(0);

        assert_eq!(set.get(0), None);
        assert_eq!(set.get(3), Some(&Marker));
    }

    #[test]
    fn given_sparse_value_and_hole_when_set_then_preserve_existing_value() {
        let mut set = SparseSet::new();
        set.set(5, String::from("five"));
        set.set(2, String::from("two"));

        assert_eq!(set.get(2).map(String::as_str), Some("two"));
        assert_eq!(set.get(5).map(String::as_str), Some("five"));
        assert!(!set.contains(3));
    }

    #[test]
    fn given_existing_id_when_remove_then_contains_and_get_report_absent() {
        let mut set = SparseSet::new();
        set.set(0, String::from("removed"));
        println!("{:?}", set);
        set.remove(0);
        println!("{:?}", set);

        assert!(!set.contains(0));
        assert_eq!(set.get(0), None);
    }

    #[test]
    fn given_multiple_ids_when_remove_then_preserve_other_ids() {
        let mut set = SparseSet::new();
        set.set(0, String::from("zero"));
        set.set(1, String::from("one"));
        set.set(2, String::from("two"));

        set.remove(1);

        assert_eq!(set.get(0).map(String::as_str), Some("zero"));
        assert_eq!(set.get(1), None);
        assert_eq!(set.get(2).map(String::as_str), Some("two"));
    }

    #[test]
    fn given_out_of_bounds_id_when_remove_then_preserve_existing_values() {
        let mut set = SparseSet::new();
        set.set(0, String::from("kept"));

        set.remove(1_000);
        set.remove(u32::MAX as usize);

        assert_eq!(set.get(0).map(String::as_str), Some("kept"));
    }

    #[test]
    fn given_removed_id_when_remove_then_operation_is_a_no_op() {
        let mut set = SparseSet::new();
        set.set(0, String::from("removed"));

        set.remove(0);
        set.remove(0);

        assert!(!set.contains(0));
        assert_eq!(set.get(0), None);
    }

    #[test]
    fn given_removed_nonzero_id_when_set_then_return_new_value() {
        let mut set = SparseSet::new();
        set.set(7, String::from("old"));
        set.remove(7);
        set.set(7, String::from("new"));

        assert!(set.contains(7));
        assert_eq!(set.get(7).map(String::as_str), Some("new"));
    }

    #[test]
    fn given_existing_component_when_set_then_drop_old_value_exactly_once() {
        let drops = Rc::new(Cell::new(0));
        let mut set = SparseSet::new();
        set.set(0, DropProbe(drops.clone()));

        set.set(0, DropProbe(drops.clone()));

        assert_eq!(drops.get(), 1);
        drop(set);
        assert_eq!(drops.get(), 2);
    }

    #[test]
    fn given_existing_component_when_remove_then_drop_value_immediately() {
        let drops = Rc::new(Cell::new(0));
        let mut set = SparseSet::new();
        set.set(0, DropProbe(drops.clone()));

        set.remove(0);

        assert_eq!(drops.get(), 1);
        drop(set);
        assert_eq!(drops.get(), 1);
    }

    #[test]
    fn given_reinserted_components_when_remove_then_dense_storage_does_not_grow() {
        let mut set = SparseSet::new();

        for value in 0..32 {
            set.set(0, value);
            set.remove(0);
        }

        assert!(set.dense_data.is_empty());
    }

    #[test]
    fn given_mixed_operation_history_when_get_then_match_hash_map_reference_model() {
        enum Operation {
            Set(usize, i32),
            Remove(usize),
            Observe,
        }

        let operations = [
            Operation::Set(4, 40),
            Operation::Set(0, 0),
            Operation::Set(7, 70),
            Operation::Remove(4),
            Operation::Set(4, 41),
            Operation::Set(7, 71),
            Operation::Remove(2),
            Operation::Observe,
            Operation::Remove(0),
            Operation::Set(2, 20),
        ];
        let mut set = SparseSet::new();
        let mut model = HashMap::new();

        for operation in operations {
            match operation {
                Operation::Set(id, value) => {
                    set.set(id, value);
                    model.insert(id, value);
                }
                Operation::Remove(id) => {
                    set.remove(id);
                    model.remove(&id);
                }
                Operation::Observe => {}
            }

            for id in 0..=8 {
                assert_eq!(
                    set.contains(id),
                    model.contains_key(&id),
                    "contains differed for id {id} | Set: {:?}, Map: {:?}", set, model
                );
                assert_eq!(
                    set.get(id),
                    model.get(&id),
                    "get differed for id {id} | Set: {:?}, Map: {:?}", set, model
                );
            }
        }
    }
}
