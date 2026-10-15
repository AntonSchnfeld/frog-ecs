#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Entity {
    pub(crate) id: u32,
    pub(crate) generation: u32
}

impl Entity {
    pub(crate) fn new(id: u32, generation: u32) -> Self {
        Entity {
            id,
            generation
        }
    }
}