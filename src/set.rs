//! Sets of boxes that can be passed to the intersection finding algorithms

use std::fmt;
use std::fmt::{Debug, Formatter};

use crate::boxes::BBox;
use crate::{median, Rng};

#[derive(Clone)]
/// A generic set of [`BBox`]es of type `B` with identifiers of type `ID`
pub struct BBoxSet<B: BBox, ID> {
    pub boxes: Vec<(B, ID)>,
}

impl<B, ID> Debug for BBoxSet<B, ID>
where
    B: BBox + Debug,
    ID: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.boxes.iter().map(|(_r, id)| id))
            .finish()
    }
}

impl<B, ID> BBoxSet<B, ID>
where
    B: BBox,
    ID: Copy + PartialEq,
    B::Num: PartialOrd,
{
    /// Creates a new, empty set. Prefer [`BBoxSet::with_capacity`].
    pub fn new() -> Self {
        Self { boxes: Vec::new() }
    }

    /// Creates a new, empty set with the specified capacity. See [`Vec::with_capacity`].
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            boxes: Vec::with_capacity(capacity),
        }
    }

    /// Adds a box with an identifier to the set.
    /// * `id` must be a unique identifier for that box.
    /// If you want to use algorithms other than [brute force](`crate::intersect_brute_force`)
    /// to find intersections, `ID` must be [`PartialOrd`]
    pub fn push(&mut self, id: ID, bbox: B) {
        self.boxes.push((bbox, id));
    }

    /// Removes all boxes from the set.
    pub fn clear(&mut self) {
        self.boxes.clear();
    }

    /// Sorts the boxes in the set by their low boundaries in dimension 0.
    /// Needed for the intersection finding algorithms.
    pub fn sort(&mut self) {
        self.boxes
            .sort_by(|(a, _), (b, _)| a.lo(0).partial_cmp(&b.lo(0)).unwrap());
    }

    /// Returns the number of boxes in the set.
    pub fn len(&self) -> usize {
        self.boxes.len()
    }

    /// Returns the box at the given index and its identifier.
    pub fn get(&self, idx: usize) -> (B, ID) {
        self.boxes[idx]
    }

    /// Performs a linear search for the box with the given identifier.
    /// Returns [`Some`] if found, [`None`] otherwise.
    pub fn find(&self, id: ID) -> Option<B> {
        self.boxes
            .iter()
            .find(|x| x.1 == id)
            .and_then(|x| Some(x.0))
    }

    /// Returns `true` if the set is empty.
    pub fn empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a subset of the set, containing only those boxes that match the given predicate.
    /// If the set is sorted, the sorting is preserved in the subset.
    pub fn filter<P>(&self, pred: P) -> Self
    where
        P: FnMut(&&(B, ID)) -> bool,
    {
        Self {
            boxes: self.boxes.iter().filter(pred).cloned().collect(),
        }
    }

    /// Returns a pair of subsets of the set, containing:
    /// * those boxes for which the given predicate returns `true`
    /// * those for which it returns `false`
    ///
    /// If the set is sorted, the sorting is preserved in the subsets.
    pub fn partition<P>(&self, pred: P) -> (Self, Self)
    where
        P: FnMut(&&(B, ID)) -> bool,
    {
        let (tr, fls) = self.boxes.iter().partition(pred);
        (Self { boxes: tr }, Self { boxes: fls })
    }

    /// Returns an approximate median of the low boundaries in dimension `dim` of the boxes,
    /// obtained by recursively calculating medians of three (medians of ...) random elements
    /// * `rand` must be a random number generator implementing the [`Rng`] trait.
    pub fn approx_median<R: Rng>(&self, dim: usize, rand: &mut R) -> B::Num {
        // magic formula for the number of levels from CGAL: https://github.com/CGAL/cgal/blob/f513a791e2f474b002564e2e9300293877d6d91e/Box_intersection_d/include/CGAL/Box_intersection_d/segment_tree.h#L263
        let mut levels = (0.91 * ((self.len() as f64) / 137.0 + 1.0).ln().floor()) as u32;
        if levels == 0 {
            levels = 1;
        }
        let cap = 3usize.pow(levels);
        let mut random_indices = Vec::<usize>::with_capacity(cap);

        let points: Vec<B::Num> = self.boxes.iter().map(|&(bbox, _id)| bbox.lo(dim)).collect();
        for _ in 0..cap {
            random_indices.push(rand.rand_usize(points.len()));
        }
        median::approx_median(&points, levels as u8, &mut random_indices)
    }
}
