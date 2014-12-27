use std::{ u16 };
use std::slice::BinarySearchResult::{ Found, NotFound };

use container::Container;

mod store;
mod container;

pub struct RoaringBitmap {
    containers: Vec<Container>,
}

impl RoaringBitmap {
    pub fn new() -> RoaringBitmap {
        RoaringBitmap { containers: Vec::new(), }
    }
}

impl RoaringBitmap {
    pub fn set(&mut self, index: u32, value: bool) {
        let (key, index) = calc_loc(index);
        let container = match self.containers.as_slice().binary_search(|container| key.cmp(&container.key())) {
            Found(loc) => &mut self.containers[loc],
            NotFound(loc) => {
                self.containers.insert(loc, Container::new(key));
                &mut self.containers[loc]
            },
        };
        container.set(index, value);
    }

    pub fn get(&self, index: u32) -> bool {
        let (key, index) = calc_loc(index);
        match self.containers.as_slice().binary_search(|container| key.cmp(&container.key())) {
            Found(loc) => self.containers[loc].get(index),
            NotFound(_) => false,
        }
    }
}

impl RoaringBitmap {
    pub fn none(&self) -> bool {
        self.cardinality() == 0u32
    }

    pub fn any(&self) -> bool {
        self.cardinality() != 0u32
    }

    pub fn cardinality(&self) -> u32 {
        self.containers
            .iter()
            .map(|container| container.cardinality() as u32)
            .fold(0, |sum, cardinality| sum + cardinality)
    }
}

#[inline]
fn calc_loc(index: u32) -> (u16, u16) { ((index >> u16::BITS) as u16, index as u16) }

#[cfg(test)]
mod test {
    use std::{ u16, u32 };
    use super::{ calc_loc };

    #[test]
    fn test_calc_location() {
        assert_eq!((0, 0), calc_loc(0));
        assert_eq!((0, 1), calc_loc(1));
        assert_eq!((0, u16::MAX - 1), calc_loc(u16::MAX as u32 - 1));
        assert_eq!((0, u16::MAX), calc_loc(u16::MAX as u32));
        assert_eq!((1, 0), calc_loc(u16::MAX as u32 + 1));
        assert_eq!((1, 1), calc_loc(u16::MAX as u32 + 2));
        assert_eq!((u16::MAX, u16::MAX - 1), calc_loc(u32::MAX - 1));
        assert_eq!((u16::MAX, u16::MAX), calc_loc(u32::MAX));
    }
}
