use std::collections::btree_map::Entry;
use std::mem;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

use crate::RoaringTreemap;

impl BitOr<RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `union` between two sets.
    fn bitor(mut self, rhs: RoaringTreemap) -> RoaringTreemap {
        BitOrAssign::bitor_assign(&mut self, rhs);
        self
    }
}

impl BitOr<&RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `union` between two sets.
    fn bitor(mut self, rhs: &RoaringTreemap) -> RoaringTreemap {
        BitOrAssign::bitor_assign(&mut self, rhs);
        self
    }
}

impl BitOr<RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `union` between two sets.
    fn bitor(self, rhs: RoaringTreemap) -> RoaringTreemap {
        BitOr::bitor(rhs, self)
    }
}

impl BitOr<&RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `union` between two sets.
    fn bitor(self, rhs: &RoaringTreemap) -> RoaringTreemap {
        if self.len() <= rhs.len() {
            BitOr::bitor(rhs.clone(), self)
        } else {
            BitOr::bitor(self.clone(), rhs)
        }
    }
}

impl BitOrAssign<RoaringTreemap> for RoaringTreemap {
    /// An `union` between two sets.
    fn bitor_assign(&mut self, mut rhs: RoaringTreemap) {
        // We make sure that we apply the union operation on the biggest map.
        if self.len() < rhs.len() {
            mem::swap(self, &mut rhs);
        }

        for (key, other_rb) in rhs.map {
            match self.map.entry(key) {
                Entry::Vacant(ent) => {
                    ent.insert(other_rb);
                }
                Entry::Occupied(mut ent) => {
                    BitOrAssign::bitor_assign(ent.get_mut(), other_rb);
                }
            }
        }
    }
}

impl BitOrAssign<&RoaringTreemap> for RoaringTreemap {
    /// An `union` between two sets.
    fn bitor_assign(&mut self, rhs: &RoaringTreemap) {
        for (key, other_rb) in &rhs.map {
            match self.map.entry(*key) {
                Entry::Vacant(ent) => {
                    ent.insert(other_rb.clone());
                }
                Entry::Occupied(mut ent) => {
                    BitOrAssign::bitor_assign(ent.get_mut(), other_rb);
                }
            }
        }
    }
}

impl BitAnd<RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `intersection` between two sets.
    fn bitand(mut self, rhs: RoaringTreemap) -> RoaringTreemap {
        BitAndAssign::bitand_assign(&mut self, rhs);
        self
    }
}

impl BitAnd<&RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `intersection` between two sets.
    fn bitand(mut self, rhs: &RoaringTreemap) -> RoaringTreemap {
        BitAndAssign::bitand_assign(&mut self, rhs);
        self
    }
}

impl BitAnd<RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `intersection` between two sets.
    fn bitand(self, rhs: RoaringTreemap) -> RoaringTreemap {
        BitAnd::bitand(rhs, self)
    }
}

impl BitAnd<&RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `intersection` between two sets.
    fn bitand(self, rhs: &RoaringTreemap) -> RoaringTreemap {
        if rhs.len() < self.len() {
            BitAnd::bitand(self.clone(), rhs)
        } else {
            BitAnd::bitand(rhs.clone(), self)
        }
    }
}

impl BitAndAssign<RoaringTreemap> for RoaringTreemap {
    /// An `intersection` between two sets.
    fn bitand_assign(&mut self, mut rhs: RoaringTreemap) {
        // We make sure that we apply the intersection operation on the smallest map.
        if rhs.len() < self.len() {
            mem::swap(self, &mut rhs);
        }

        BitAndAssign::bitand_assign(self, &rhs)
    }
}

impl BitAndAssign<&RoaringTreemap> for RoaringTreemap {
    /// An `intersection` between two sets.
    fn bitand_assign(&mut self, rhs: &RoaringTreemap) {
        let mut keys_to_remove: Vec<u32> = Vec::new();
        for (key, self_rb) in &mut self.map {
            match rhs.map.get(key) {
                Some(other_rb) => {
                    BitAndAssign::bitand_assign(self_rb, other_rb);
                    if self_rb.is_empty() {
                        keys_to_remove.push(*key);
                    }
                }
                None => keys_to_remove.push(*key),
            }
        }

        for key in keys_to_remove {
            self.map.remove(&key);
        }
    }
}

impl Sub<RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `difference` between two sets.
    fn sub(mut self, rhs: RoaringTreemap) -> RoaringTreemap {
        SubAssign::sub_assign(&mut self, rhs);
        self
    }
}

impl Sub<&RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `difference` between two sets.
    fn sub(mut self, rhs: &RoaringTreemap) -> RoaringTreemap {
        SubAssign::sub_assign(&mut self, rhs);
        self
    }
}

impl Sub<RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `difference` between two sets.
    fn sub(self, rhs: RoaringTreemap) -> RoaringTreemap {
        Sub::sub(self.clone(), rhs)
    }
}

impl Sub<&RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `difference` between two sets.
    fn sub(self, rhs: &RoaringTreemap) -> RoaringTreemap {
        Sub::sub(self.clone(), rhs)
    }
}

impl SubAssign<RoaringTreemap> for RoaringTreemap {
    /// A `difference` between two sets.
    fn sub_assign(&mut self, rhs: RoaringTreemap) {
        SubAssign::sub_assign(self, &rhs)
    }
}

impl SubAssign<&RoaringTreemap> for RoaringTreemap {
    /// A `difference` between two sets.
    fn sub_assign(&mut self, rhs: &RoaringTreemap) {
        for (key, rhs_rb) in &rhs.map {
            match self.map.entry(*key) {
                Entry::Vacant(_entry) => (),
                Entry::Occupied(mut entry) => {
                    SubAssign::sub_assign(entry.get_mut(), rhs_rb);
                    if entry.get().is_empty() {
                        entry.remove_entry();
                    }
                }
            }
        }
    }
}

impl BitXor<RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `symmetric difference` between two sets.
    fn bitxor(mut self, rhs: RoaringTreemap) -> RoaringTreemap {
        BitXorAssign::bitxor_assign(&mut self, rhs);
        self
    }
}

impl BitXor<&RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `symmetric difference` between two sets.
    fn bitxor(mut self, rhs: &RoaringTreemap) -> RoaringTreemap {
        BitXorAssign::bitxor_assign(&mut self, rhs);
        self
    }
}

impl BitXor<RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `symmetric difference` between two sets.
    fn bitxor(self, rhs: RoaringTreemap) -> RoaringTreemap {
        BitXor::bitxor(rhs, self)
    }
}

impl BitXor<&RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `symmetric difference` between two sets.
    fn bitxor(self, rhs: &RoaringTreemap) -> RoaringTreemap {
        if self.len() < rhs.len() {
            BitXor::bitxor(self, rhs.clone())
        } else {
            BitXor::bitxor(self.clone(), rhs)
        }
    }
}

impl BitXorAssign<RoaringTreemap> for RoaringTreemap {
    /// A `symmetric difference` between two sets.
    fn bitxor_assign(&mut self, rhs: RoaringTreemap) {
        for (key, other_rb) in rhs.map {
            match self.map.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(other_rb);
                }
                Entry::Occupied(mut entry) => {
                    BitXorAssign::bitxor_assign(entry.get_mut(), other_rb);
                    if entry.get().is_empty() {
                        entry.remove_entry();
                    }
                }
            }
        }
    }
}

impl BitXorAssign<&RoaringTreemap> for RoaringTreemap {
    /// A `symmetric difference` between two sets.
    fn bitxor_assign(&mut self, rhs: &RoaringTreemap) {
        for (key, other_rb) in &rhs.map {
            match self.map.entry(*key) {
                Entry::Vacant(entry) => {
                    entry.insert(other_rb.clone());
                }
                Entry::Occupied(mut entry) => {
                    BitXorAssign::bitxor_assign(entry.get_mut(), other_rb);
                    if entry.get().is_empty() {
                        entry.remove_entry();
                    }
                }
            }
        }
    }
}
