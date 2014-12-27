use store::Store;

pub struct Container {
    key: u16,
    cardinality: u16,
    store: Store,
}

impl Container {
    pub fn new(key: u16) -> Container {
        Container {
            key: key,
            cardinality: 0,
            store: Store::Array(Vec::new()),
        }
    }
}

impl Container {
    #[inline]
    pub fn key(&self) -> u16 { self.key }

    #[inline]
    pub fn cardinality(&self) -> u16 { self.cardinality }

    pub fn set(&mut self, index: u16, value: bool) {
        match (self.store.set(index, value), value, self.cardinality) {
            (true, true, 4096) => { self.cardinality += 1; self.store = self.store.to_bitmap() },
            (true, true, _) => self.cardinality += 1,
            (true, false, 4097) => { self.cardinality -= 1; self.store = self.store.to_array() },
            (true, false, _) => self.cardinality -= 1,
            (false, _, _) => (),
        }
    }

    #[inline]
    pub fn get(&self, index: u16) -> bool {
        self.store.get(index)
    }
}

