#[cfg(test)]
#[allow(clippy::eq_op)] // Allow equal expressions as operands
mod test {
    use crate::RoaringBitmap;
    use proptest::prelude::*;

    //
    // Tests algebraic set properties in terms of RoaringBitmaps.
    // Follows wikipedia article regarding ordering and heading
    //
    // https://en.wikipedia.org/wiki/Algebra_of_sets
    //
    // Notes:
    //
    //  * Although a universe set exists, we leave properties involving it it out of these tests.
    //    It would be ~512 MiB and operations on it would be relatively slow
    //
    //  * Likewise, there is no compliment operator
    //
    //
    //
    //
    // The fundamental properties of set algebra
    // =========================================
    //
    // Commutative property:
    // --------------------

    proptest! {
        #[test]
        fn unions_are_commutative(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary()
        ) {
            prop_assert_eq!(&a | &b, &b | &a);
        }

        #[test]
        fn intersections_are_commutative(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary()
        ) {
            prop_assert_eq!(&a & &b, &b & &a);
        }
    }

    //
    // Associative property:
    // ---------------------

    proptest! {
        #[test]
        fn unions_are_associative(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary(),
            c in RoaringBitmap::arbitrary()
        ) {
            prop_assert_eq!(
                &a | ( &b | &c ),
                ( &a | &b ) | &c
            );
        }

        #[test]
        fn intersections_are_associative(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary(),
            c in RoaringBitmap::arbitrary()
        ) {
            prop_assert_eq!(
                &a & ( &b & &c ),
                ( &a & &b ) & &c
            );
        }
    }

    //
    // Distributive property:
    // ---------------------

    proptest! {
        #[test]
        fn union_distributes_over_intersection(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary(),
            c in RoaringBitmap::arbitrary()
        ) {
            prop_assert_eq!(
                &a | ( &b & &c),
                ( &a | &b ) & ( &a | &c )
            );
        }

        #[test]
        fn intersection_distributes_over_union(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary(),
            c in RoaringBitmap::arbitrary()
        ) {
            prop_assert_eq!(
                &a & ( &b | &c),
                ( &a & &b ) | ( &a & &c )
            );
        }
    }

    // Identity:
    // --------

    proptest! {
        #[test]
        fn the_empty_set_is_the_identity_for_union(a in RoaringBitmap::arbitrary()) {
            prop_assert_eq!(&a | &empty_set(), a);
        }
    }

    // Some additional laws for unions and intersections
    // =================================================
    //
    // PROPOSITION 3: For any subsets A and B of a universe set U, the following identities hold:
    //
    // Idempotent laws
    // ---------------

    proptest! {
        #[test]
        fn unions_are_idempotent(a in RoaringBitmap::arbitrary()) {
            prop_assert_eq!(&a | &a, a);
        }

        #[test]
        fn intersections_are_idempotent(a in RoaringBitmap::arbitrary()) {
            prop_assert_eq!(&a & &a, a);
        }
    }

    //
    // Domination laws
    // ---------------

    proptest! {
        #[test]
        fn empty_set_domination(a in RoaringBitmap::arbitrary()) {
            prop_assert_eq!(&a & &empty_set(), empty_set());
        }
    }

    // The algebra of inclusion
    // ========================
    // PROPOSITION 6: If A, B and C are sets then the following hold:

    proptest! {
        #[test]
        fn reflexivity(a in RoaringBitmap::arbitrary()) {
            prop_assert!(a.is_subset(&a));
        }

        #[test]
        fn antisymmetry(a in RoaringBitmap::arbitrary()) {
            let mut b = a.clone();
            prop_assert_eq!(&a, &b);
            prop_assert!(a.is_subset(&b) && b.is_subset(&a));

            b.insert(a.max().unwrap_or(0) + 1);
            prop_assert_ne!(&a, &b);
            prop_assert!(!(a.is_subset(&b) && b.is_subset(&a)));
        }

        #[test]
        fn transitivity(
            a in RoaringBitmap::arbitrary(),
            mut b in RoaringBitmap::arbitrary(),
            mut c in RoaringBitmap::arbitrary()
        ) {
            b |= &a;
            c |= &b;
            // If
            prop_assert!(a.is_subset(&b));
            prop_assert!(b.is_subset(&c));
            // Then
            prop_assert!(a.is_subset(&c));

        }
    }

    // PROPOSITION 7: If A, B and C are subsets of a set S then the following hold:

    proptest! {
        #[test]
        fn existence_of_joins(a in RoaringBitmap::arbitrary(), b in RoaringBitmap::arbitrary()) {
            prop_assert!(a.is_subset(&(&a | &b)));

        }

        #[test]
        fn existence_of_meets(a in RoaringBitmap::arbitrary(), b in RoaringBitmap::arbitrary()) {
            prop_assert!(&(&a & &b).is_subset(&a));

        }
    }

    // PROPOSITION 8: For any two sets A and B, the following are equivalent:

    proptest! {
        #[test]
        fn inclusion_can_be_characterized_by_union_or_inersection(
            b in RoaringBitmap::arbitrary(),
            c in RoaringBitmap::arbitrary()
        ) {
            let a = &b - &c;

            prop_assert!(a.is_subset(&b));
            prop_assert_eq!(&(&a & &b), &a);
            prop_assert_eq!(&(&a | &b), &b);
            prop_assert_eq!(&(&a - &b), &empty_set());
        }
    }

    // The algebra of relative complements
    // ===================================
    //
    // PROPOSITION 9: For any universe U and subsets A, B, and C of U,
    // the following identities hold:

    proptest! {
        #[test]
        fn relative_compliments(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary(),
            c in RoaringBitmap::arbitrary()
        ) {
            let u = &a | &b | &c;

            prop_assert_eq!(
                &c - (&a & &b),
                (&c - &a) | (&c - &b)
            );

            prop_assert_eq!(
                &c - (&a | &b),
                (&c - &a) & (&c - &b)
            );

            prop_assert_eq!(
                &c - (&b - &a),
                (&a & &c) | (&c - &b)
            );

            {
                let x = (&b - &a) & &c;
                let y = (&b & &c) - &a;
                let z = &b & (&c - &a);

                prop_assert_eq!(&x, &y);
                prop_assert_eq!(&y, &z);
                prop_assert_eq!(&z, &x);
            }

            prop_assert_eq!(
                (&b - &a) | &c,
                (&b | &c) - (&a - &c)
            );

            prop_assert_eq!(
                (&b - &a) - &c,
                &b - (&a | &c)
            );

            prop_assert_eq!(
                &a - &a,
                empty_set()
            );

             prop_assert_eq!(
                empty_set() - &a,
                empty_set()
            );

            prop_assert_eq!(
                &a - &u,
                empty_set()
            );
        }
    }

    fn empty_set() -> RoaringBitmap {
        RoaringBitmap::new()
    }
}
