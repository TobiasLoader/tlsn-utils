use std::ops::{BitAnd, BitAndAssign};

use crate::range::{Range, RangeSet};

pub trait Intersection<Rhs> {
    type Output;

    /// Returns the set intersection of `self` and `other`.
    #[must_use]
    fn intersection(&self, other: &Rhs) -> Self::Output;
}

impl<T: Copy + Ord> Intersection<Range<T>> for Range<T> {
    type Output = Option<Range<T>>;

    fn intersection(&self, other: &Range<T>) -> Self::Output {
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);

        if start < end {
            Some(Range { start, end })
        } else {
            None
        }
    }
}

impl<T: Copy + Ord> Intersection<RangeSet<T>> for Range<T> {
    type Output = RangeSet<T>;

    fn intersection(&self, other: &RangeSet<T>) -> Self::Output {
        let mut set = RangeSet::default();

        for other in &other.ranges {
            if self.end <= other.start {
                // `self` is leftward of `other`, so we can break early.
                break;
            } else if let Some(intersection) = self.intersection(other) {
                // Given that `other` contains sorted, non-adjacent, non-intersecting, and non-empty
                // ranges, the new set will also have these properties.
                set.ranges.push(intersection);
            }
        }

        set
    }
}

impl<T: Copy + Ord> Intersection<Range<T>> for RangeSet<T> {
    type Output = RangeSet<T>;

    fn intersection(&self, other: &Range<T>) -> Self::Output {
        other.intersection(self)
    }
}

impl<T: Copy + Ord> Intersection<RangeSet<T>> for RangeSet<T> {
    type Output = RangeSet<T>;

    fn intersection(&self, other: &RangeSet<T>) -> Self::Output {
        let mut set = RangeSet::default();

        let mut i = 0;
        let mut j = 0;

        while i < self.ranges.len() && j < other.ranges.len() {
            let a = &self.ranges[i];
            let b = &other.ranges[j];

            if a.end <= b.start {
                // `a` is leftward of `b`, so we can proceed to the next range in `self`.
                i += 1;
            } else if b.end <= a.start {
                // `b` is leftward of `a`, so we can proceed to the next range in `other`.
                j += 1;
            } else if let Some(intersection) = a.intersection(b) {
                // Given that `self` and `other` contain sorted, non-adjacent, non-intersecting, and
                // non-empty ranges, the new set will also have these properties.
                set.ranges.push(intersection);

                if a.end <= b.end {
                    i += 1;
                }

                if b.end <= a.end {
                    j += 1;
                }
            }
        }

        set
    }
}

impl<T: Copy + Ord> BitAndAssign<Range<T>> for RangeSet<T> {
    fn bitand_assign(&mut self, other: Range<T>) {
        *self = self.intersection(&other);
    }
}

impl<T: Copy + Ord> BitAndAssign<&Range<T>> for RangeSet<T> {
    fn bitand_assign(&mut self, other: &Range<T>) {
        *self = self.intersection(other);
    }
}

impl<T: Copy + Ord> BitAnd<RangeSet<T>> for Range<T> {
    type Output = RangeSet<T>;

    fn bitand(self, other: RangeSet<T>) -> Self::Output {
        self.intersection(&other)
    }
}

impl<T: Copy + Ord> BitAnd<&RangeSet<T>> for Range<T> {
    type Output = RangeSet<T>;

    fn bitand(self, other: &RangeSet<T>) -> Self::Output {
        self.intersection(other)
    }
}

impl<T: Copy + Ord> BitAnd<Range<T>> for RangeSet<T> {
    type Output = RangeSet<T>;

    fn bitand(self, other: Range<T>) -> Self::Output {
        other.intersection(&self)
    }
}

impl<T: Copy + Ord> BitAnd<&Range<T>> for RangeSet<T> {
    type Output = RangeSet<T>;

    fn bitand(self, other: &Range<T>) -> Self::Output {
        other.intersection(&self)
    }
}

impl<T: Copy + Ord> BitAndAssign<RangeSet<T>> for RangeSet<T> {
    fn bitand_assign(&mut self, other: RangeSet<T>) {
        *self = self.intersection(&other);
    }
}

impl<T: Copy + Ord> BitAndAssign<&RangeSet<T>> for RangeSet<T> {
    fn bitand_assign(&mut self, other: &RangeSet<T>) {
        *self = self.intersection(other);
    }
}

impl<T: Copy + Ord> BitAnd<RangeSet<T>> for RangeSet<T> {
    type Output = RangeSet<T>;

    fn bitand(self, other: RangeSet<T>) -> Self::Output {
        self.intersection(&other)
    }
}

impl<T: Copy + Ord> BitAnd<&RangeSet<T>> for RangeSet<T> {
    type Output = RangeSet<T>;

    fn bitand(self, other: &RangeSet<T>) -> Self::Output {
        self.intersection(other)
    }
}

#[cfg(test)]
#[allow(clippy::single_range_in_vec_init)]
mod tests {
    use std::collections::HashSet;

    use itertools::iproduct;

    use crate::range::assert_invariants;

    use super::*;

    #[test]
    fn test_range_intersection_range() {
        assert!((0..0).intersection(&(0..0)).is_none());
        assert!((0..1).intersection(&(0..0)).is_none());
        assert!((0..0).intersection(&(0..1)).is_none());
        assert_eq!((0..1).intersection(&(0..1)), Some(0..1));
        assert_eq!((0..2).intersection(&(0..1)), Some(0..1));
        assert_eq!((0..1).intersection(&(0..2)), Some(0..1));
        assert_eq!((0..2).intersection(&(0..2)), Some(0..2));
        assert_eq!((0..2).intersection(&(1..2)), Some(1..2));
        assert_eq!((1..2).intersection(&(0..2)), Some(1..2));
    }

    #[test]
    fn test_range_intersection_set() {
        let set = RangeSet::from(vec![0..1, 2..3, 4..5]);

        assert_eq!(set.intersection(&(0..0)), RangeSet::default());
        assert_eq!(set.intersection(&(0..1)), RangeSet::from(vec![0..1]));
        assert_eq!(set.intersection(&(0..2)), RangeSet::from(vec![0..1]));
        assert_eq!(set.intersection(&(0..3)), RangeSet::from(vec![0..1, 2..3]));
        assert_eq!(set.intersection(&(0..4)), RangeSet::from(vec![0..1, 2..3]));
        assert_eq!(set.intersection(&(1..3)), RangeSet::from(vec![2..3]));
        assert_eq!(
            set.intersection(&(0..6)),
            RangeSet::from(vec![0..1, 2..3, 4..5])
        );
        assert_eq!(
            set.intersection(&(0..6)),
            RangeSet::from(vec![0..1, 2..3, 4..5])
        );
    }

    #[test]
    fn test_set_intersection_set() {
        let set = RangeSet::from(vec![0..1, 2..3, 5..6]);

        assert_eq!(set.intersection(&RangeSet::default()), RangeSet::default());
        assert_eq!(
            set.intersection(&RangeSet::from(vec![1..2])),
            RangeSet::default()
        );
        assert_eq!(
            set.intersection(&RangeSet::from(vec![3..5])),
            RangeSet::default()
        );
        assert_eq!(
            set.intersection(&RangeSet::from(vec![7..8])),
            RangeSet::default()
        );
        assert_eq!(
            set.intersection(&RangeSet::from(vec![0..1])),
            RangeSet::from(vec![0..1])
        );
        assert_eq!(
            set.intersection(&RangeSet::from(vec![0..2])),
            RangeSet::from(vec![0..1])
        );
        assert_eq!(
            set.intersection(&RangeSet::from(vec![0..3])),
            RangeSet::from(vec![0..1, 2..3])
        );
        assert_eq!(set.intersection(&RangeSet::from(vec![0..6])), set);
        assert_eq!(
            set.intersection(&RangeSet::from(vec![1..6])),
            RangeSet::from(vec![2..3, 5..6])
        );
        assert_eq!(
            set.intersection(&RangeSet::from(vec![2..3, 5..6])),
            RangeSet::from(vec![2..3, 5..6])
        );
    }

    #[test]
    #[ignore = "expensive"]
    fn test_prove_set_intersection_set_8x2_8x2() {
        for (xs, xe, ys, ye, ws, we, zs, ze) in
            iproduct!(0..8, 0..8, 0..8, 0..8, 0..8, 0..8, 0..8, 0..8)
        {
            let s1 = RangeSet::new(&[(xs..xe), (ys..ye)]);
            let s2 = RangeSet::new(&[(ws..we), (zs..ze)]);

            let h1 = s1.iter().collect::<HashSet<_>>();
            let h2 = s2.iter().collect::<HashSet<_>>();

            let actual = s1.intersection(&s2);
            let h3 = HashSet::<usize>::from_iter(actual.iter());

            assert_invariants(&actual);

            assert_eq!(
                h3,
                h1.intersection(&h2).copied().collect::<HashSet<_>>(),
                "{:?} {:?} {:?} {:?} => {:?}",
                xs..xe,
                ys..ye,
                ws..we,
                zs..ze,
                h3
            );
        }
    }
}
