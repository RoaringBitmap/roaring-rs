use store::Store;
use store::Store::{ Array, Bitmap };

pub struct Container {
    key: u16,
    len: u16,
    store: Store,
}

impl Container {
    pub fn new(key: u16) -> Container {
        Container {
            key: key,
            len: 0,
            store: Array(Vec::new()),
        }
    }
}

impl Container {
    #[inline]
    pub fn key(&self) -> u16 { self.key }

    #[inline]
    pub fn len(&self) -> u16 { self.len }

    pub fn insert(&mut self, index: u16) -> bool {
        if self.store.insert(index) {
            self.len += 1;
            if self.len == 4097 {
                self.store = self.store.to_bitmap();
            }
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, index: u16) -> bool {
        if self.store.remove(index) {
            self.len -= 1;
            if self.len == 4096 {
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

