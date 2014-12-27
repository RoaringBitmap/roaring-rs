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

    pub fn insert(&mut self, index: u16) -> bool {
        if self.store.insert(index) {
            self.cardinality += 1;
            if self.cardinality == 4097 {
                self.store = self.store.to_bitmap();
            }
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, index: u16) -> bool {
        if self.store.remove(index) {
            self.cardinality -= 1;
            if self.cardinality == 4096 {
                self.store = self.store.to_array();
            }
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn contains(&self, index: u16) -> bool {
        self.store.contains(index)
    }
}

