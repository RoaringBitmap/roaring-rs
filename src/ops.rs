use std::ops::{ BitXor, BitAnd, BitOr, Sub };

use num::traits::Zero;

use RoaringBitmap;
use util::{ Halveable, ExtInt };

impl<Size: ExtInt + Halveable> RoaringBitmap<Size> {
    /// Unions in-place with the specified other bitmap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (1..5u32).collect();
    ///
    /// rb1.union_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn union_with(&mut self, other: &Self) {
        for container in &other.containers {
            let key = container.key;
            match self.containers.binary_search_by(|container| container.key.cmp(&key)) {
                Err(loc) => self.containers.insert(loc, container.clone()),
                Ok(loc) => self.containers[loc].union_with(container),
            };
        }
    }

    /// Intersects in-place with the specified other bitmap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (3..4u32).collect();
    ///
    /// rb1.intersect_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn intersect_with(&mut self, other: &Self) {
        let mut index = 0;
        while index < self.containers.len() {
            let key = self.containers[index].key;
            match other.containers.binary_search_by(|container| container.key.cmp(&key)) {
                Err(_) => {
                    self.containers.remove(index);
                },
                Ok(loc) => {
                    self.containers[index].intersect_with(&other.containers[loc]);
                    if self.containers[index].len == Zero::zero() {
                        self.containers.remove(index);
                    } else {
                        index += 1;
                    }
                },
            };
        }
    }

    /// Removes all values in the specified other bitmap from self, in-place.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (1..3u32).collect();
    ///
    /// rb1.difference_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn difference_with(&mut self, other: &Self) {
        let mut index = 0;
        while index < self.containers.len() {
            let key = self.containers[index].key;
            match other.containers.binary_search_by(|container| container.key.cmp(&key)) {
                Ok(loc) => {
                    self.containers[index].difference_with(&other.containers[loc]);
                    if self.containers[index].len == Zero::zero() {
                        self.containers.remove(index);
                    } else {
                        index += 1;
                    }
                },
                _ => {
                    index += 1;
                }
            };
        }
    }

    /// Replaces this bitmap with one that is equivalent to `self XOR other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..6u32).collect();
    /// let rb3: RoaringBitmap<u32> = ((1..3u32).chain(4..6u32)).collect();
    ///
    /// rb1.symmetric_difference_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn symmetric_difference_with(&mut self, other: &Self) {
        for container in &other.containers {
            let key = container.key;
            match self.containers.binary_search_by(|container| container.key.cmp(&key)) {
                Err(loc) => self.containers.insert(loc, (*container).clone()),
                Ok(loc) => {
                    self.containers[loc].symmetric_difference_with(container);
                    if self.containers[loc].len == Zero::zero() {
                        self.containers.remove(loc);
                    }
                }
            };
        }
    }
}

impl<Size: ExtInt + Halveable> BitOr<RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = Self;

    /// Unions the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (2..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (1..5u32).collect();
    ///
    /// let rb4 = rb1 | rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// ```
    fn bitor(mut self, rhs: Self) -> Self {
        self.union_with(&rhs);
        self
    }
}

impl<'a, Size: ExtInt + Halveable> BitOr<RoaringBitmap<Size>> for &'a RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    /// Unions`rhs` and `self`, writes result in place to `rhs`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (2..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (1..5u32).collect();
    ///
    /// let rb4 = &rb1 | rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// ```
    fn bitor(self, mut rhs: RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        rhs.union_with(self);
        rhs
    }
}

impl<'a, 'b, Size: ExtInt + Halveable> BitOr<&'a RoaringBitmap<Size>> for &'b RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    /// Unions`rhs` and `self`, allocates new bitmap for result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (2..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (1..5u32).collect();
    ///
    /// let rb4 = rb1 | &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// ```
    fn bitor(self, rhs: &'a RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        let mut result = self.clone();
        result.union_with(rhs);
        result
    }
}

impl<'a, Size: ExtInt + Halveable> BitOr<&'a RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = Self;

    /// Unions the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (2..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (1..5u32).collect();
    ///
    /// let rb4 = rb1 | &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// ```
    fn bitor(mut self, rhs: &'a Self) -> Self {
        self.union_with(rhs);
        self
    }
}

impl<Size: ExtInt + Halveable> BitAnd<RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = Self;

    /// Intersects the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (2..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (2..4u32).collect();
    ///
    /// let rb4 = rb1 & rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// ```
    fn bitand(mut self, rhs: Self) -> Self {
        self.intersect_with(&rhs);
        self
    }
}

impl<'a, Size: ExtInt + Halveable> BitAnd<&'a RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = Self;

    /// Intersects the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (2..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (2..4u32).collect();
    ///
    /// let rb4 = rb1 & &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// ```
    fn bitand(mut self, rhs: &'a Self) -> Self {
        self.intersect_with(rhs);
        self
    }
}

impl<'a, Size: ExtInt + Halveable> BitAnd<RoaringBitmap<Size>> for &'a RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    /// Intersects `self` into the `rhs` `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (2..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (2..4u32).collect();
    ///
    /// let rb4 = &rb1 & rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// ```
    fn bitand(self, mut rhs: RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        rhs.intersect_with(self);
        rhs
    }
}

impl<'a, 'b, Size: ExtInt + Halveable> BitAnd<&'a RoaringBitmap<Size>> for &'b RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    /// Intersects `self` and `rhs` into a new `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (2..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (2..4u32).collect();
    ///
    /// let rb4 = &rb1 & &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// ```
    fn bitand(self, rhs: &'a RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        let mut result = self.clone();
        result.intersect_with(rhs);
        result
    }
}

impl<Size: ExtInt + Halveable> Sub<RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = Self;

    /// Subtracts the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (1..3u32).collect();
    ///
    /// let rb4 = rb1 - rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// ```
    fn sub(mut self, rhs: Self) -> Self {
        self.difference_with(&rhs);
        self
    }
}

impl<'a, Size: ExtInt + Halveable> Sub<&'a RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = Self;

    /// Subtracts the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (1..3u32).collect();
    ///
    /// let rb4 = rb1 - &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// ```
    fn sub(mut self, rhs: &'a Self) -> Self {
        self.difference_with(rhs);
        self
    }
}

impl<'a, 'b, Size: ExtInt + Halveable> Sub<&'a RoaringBitmap<Size>> for &'b RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    /// Subtracts `rhs` from `self` and allocates a new `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (1..3u32).collect();
    ///
    /// let rb4 = &rb1 - &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// ```
    fn sub(self, rhs: &'a RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        let mut result = self.clone();
        result.difference_with(rhs);
        result
    }
}

impl<Size: ExtInt + Halveable> BitXor<RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = Self;

    /// Subtracts the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..6u32).collect();
    /// let rb3: RoaringBitmap<u32> = ((1..3u32).chain(4..6u32)).collect();
    ///
    /// let rb4 = rb1 ^ rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// ```
    fn bitxor(mut self, rhs: Self) -> Self {
        self.symmetric_difference_with(&rhs);
        self
    }
}

impl<'a, Size: ExtInt + Halveable> BitXor<&'a RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    /// Exclusive ors the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..6u32).collect();
    /// let rb3: RoaringBitmap<u32> = ((1..3u32).chain(4..6u32)).collect();
    ///
    /// let rb4 = rb1 ^ &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// ```
    fn bitxor(mut self, rhs: &'a Self) -> Self {
        self.symmetric_difference_with(rhs);
        self
    }
}

impl<'a, Size: ExtInt + Halveable> BitXor<RoaringBitmap<Size>> for &'a RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    /// Exclusive ors `rhs` and `self`, writes result in place to `rhs`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..6u32).collect();
    /// let rb3: RoaringBitmap<u32> = ((1..3u32).chain(4..6u32)).collect();
    ///
    /// let rb4 = &rb1 ^ rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// ```
    fn bitxor(self, mut rhs: RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        rhs.symmetric_difference_with(self);
        rhs
    }
}

impl<'a, 'b, Size: ExtInt + Halveable> BitXor<&'a RoaringBitmap<Size>> for &'b RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    /// Exclusive ors `rhs` and `self`, allocates a new bitmap for the result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..6u32).collect();
    /// let rb3: RoaringBitmap<u32> = ((1..3u32).chain(4..6u32)).collect();
    ///
    /// let rb4 = &rb1 ^ &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// ```
    fn bitxor(self, rhs: &'a RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        let mut result = self.clone();
        result.symmetric_difference_with(rhs);
        result
    }
}

