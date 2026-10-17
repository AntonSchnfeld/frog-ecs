use crate::util::AsAny;

// TODO: Think of better names than id_mapper and mapped_id, horrible for rubber ducking
// TODO: Think of if SparseSet should be pub or pub(crate) or in a util library
#[derive(Debug)]
pub struct SparseSet<T> {
    // index = id, output = idx of component storage
    id_mapper: Vec<u32>,
    components: Vec<T>,
}

impl<T> SparseSet<T> {
    pub fn new() -> Self {
        SparseSet {
            id_mapper: Vec::new(),
            components: Vec::new(),
        }
    }

    pub fn get(&self, id: u32) -> Option<&T> {
        if id >= self.id_mapper.len() as u32 {
            return None;
        }
        // safe since we bounds checked
        let id = self.id_mapper[id as usize];
        self.components.get(id as usize)
    }

    pub fn set(&mut self, id: u32, component: T) {
        if let Some(mapped_id) = self.id_mapper.get(id as usize) {
            self.components[*mapped_id as usize] = component;
            return;
        }
        self.id_mapper.resize(id.saturating_sub(self.id_mapper.len() as u32) as usize + 1, u32::MAX);
        self.components.push(component);
        self.id_mapper[id as usize] = (self.components.len() - 1) as u32;
    }
}

pub trait SparseSetCD: AsAny {
    fn has(&self, id: u32) -> bool;
    fn remove (&mut self, id: u32);
}

// TODO: Rename in the future, not intuitive at all
impl<T: 'static> SparseSetCD for SparseSet<T> {
    fn has(&self, id: u32) -> bool {
        if let Some(idx) = self.id_mapper.get(id as usize) && *idx != u32::MAX {
            return true;
        }
        false
    }

    fn remove(&mut self, id: u32) {
        if id >= self.id_mapper.len() as u32 {
            return;
        }
        self.id_mapper[id as usize] = u32::MAX;
    }
}

#[cfg(test)]
mod test {
    use crate::sparseset::{SparseSet, SparseSetCD};

    #[derive(Debug, Eq, PartialEq)]
    struct EmptyComponent {}

    #[test]
    fn given_contains_component_when_has_then_return_true() {
        let mut set = SparseSet::new();
        let component = EmptyComponent {};
        set.set(0, component);
        assert!(set.has(0));
    }

    #[test]
    fn given_no_data_when_has_then_return_false() {
        let set: SparseSet<EmptyComponent> = SparseSet::new();
        assert!(!set.has(0));
    }

    #[test]
    fn given_no_data_at_index_when_has_then_return_false() {
        let mut set = SparseSet::new();
        set.set(1, EmptyComponent {});
        assert!(!set.has(0));
        assert!(set.has(1));
    }

    #[test]
    fn given_removed_data_at_index_when_has_then_return_false() {
        let mut set = SparseSet::new();
        set.set(0, EmptyComponent {});
        assert!(set.has(0));
        set.remove(0);
        assert!(!set.has(0));
    }

    #[test]
    fn given_data_at_index_when_has_then_return_true() {
        let mut set = SparseSet::new();
        set.set(0, EmptyComponent {});
        assert!(set.has(0));
    }

    #[test]
    fn given_data_at_index_when_get_then_return_data() {
        let mut set = SparseSet::new();
        set.set(0, 420);
        assert!(set.has(0));
        assert_eq!(*set.get(0).unwrap(), 420);
    }

    #[test]
    fn given_out_of_bounds_index_when_get_then_return_none() {
        let mut set = SparseSet::new();
        set.set(0, EmptyComponent {});
        assert!(set.has(0));
        assert_eq!(set.get(1), None);
    }

    #[test]
    fn given_in_bounds_index_with_no_data_when_get_then_return_none() {
        let mut set = SparseSet::new();
        set.set(1, EmptyComponent {});
        assert!(set.has(1));
        assert_eq!(set.get(0), None);
    }

    #[test]
    fn given_previously_used_index_when_get_then_return_none() {
        let mut set = SparseSet::new();
        set.set(0, EmptyComponent {});
        assert!(set.has(0));
        assert_eq!(*set.get(0).unwrap(), EmptyComponent {});
        set.remove(0);
        assert!(!set.has(0));
        assert_eq!(set.get(0), None);
    }
}
