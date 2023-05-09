//! Provides algorithms for broad phase collision detection. Specifically, implements
//! Zomorodian and Edelsbrunner's hybrid algorithm using streamed segment trees, pruning and scanning,
//! described in [Fast software for box intersections](https://dl.acm.org/doi/10.1145/336154.336192).
//! Takes much inspiration from [the implementation in CGAL](https://github.com/CGAL/cgal/tree/master/Box_intersection_d/include/CGAL).
//!
//! # Examples
//! ```
//! use box_intersect_ze::set::BBoxSet;
//! use box_intersect_ze::boxes::Box3Df32;
//! use rand_chacha::ChaCha8Rng;
//! use rand::SeedableRng;
//!
//! // create some boxes
//! let box0 = Box3Df32::new([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
//! let box1 = Box3Df32::new([5.0, 5.0, 5.0], [15.0, 15.0, 15.0]);
//! let box2 = Box3Df32::new([10.0, 10.0, 10.0], [20.0, 20.0, 20.0]);
//!
//! // add them to a BBoxSet
//! let mut boxes = BBoxSet::with_capacity(3);
//! boxes.push(0, box0);
//! boxes.push(1, box1);
//! boxes.push(2, box2);
//! boxes.sort(); // sort it in dimension 0
//!
//! let mut result = Vec::with_capacity(2); // set capacity according to expected number of intersections to avoid resizing
//! box_intersect_ze::intersect_ze(&boxes, &boxes, &mut result, &mut ChaCha8Rng::seed_from_u64(1234)); // get the intersections
//!
//! assert!(result.contains(&(1,0)));
//! assert!(result.contains(&(2,1)));
//! assert!(!result.contains(&(2,0)));
//! assert!(!result.contains(&(0,2)));
//! ```

use boxes::BBox;
use set::BBoxSet;

use crate::internals::{hybrid, one_way_scan, two_way_scan};

pub mod boxes;
pub mod internals;
mod median;
pub mod set;

/// Trait for box boundary types
pub trait HasInfinity {
    /// Value representing negative infinity
    const NINFTY: Self;
    /// Value representing positive infinity
    const INFTY: Self;
}

/// Trait for random number generator used in [`intersect_ze`] for approximate median calculation
pub trait Rng {
    /// Returns a random `usize` between 0 (inclusive) and `high` (exclusive)
    fn rand_usize(&mut self, high: usize) -> usize;
}

#[cfg(feature = "rand-crate")]
impl<R> Rng for R
where
    R: rand::Rng,
{
    fn rand_usize(&mut self, max: usize) -> usize {
        self.gen_range(0..max)
    }
}

/// Finds all intersections between boxes in `a` and `b` using Zomorodian and Edelsbrunner's
/// hybrid algorithm (streamed segment trees pruned with a cutoff).
/// * `a` and `b` may be either the same or distinct [`BBoxSet`]s and must be sorted before calling.
/// * `out` will contain pairs of `ID`s of intersecting boxes.
/// Choose capacity according to the number of intersections you expect to avoid resizing.
/// * `rand` must be a random number generator implementing the [`Rng`] trait. (used for approximate median selection)
pub fn intersect_ze<B, ID, R>(
    a: &BBoxSet<B, ID>,
    b: &BBoxSet<B, ID>,
    out: &mut Vec<(ID, ID)>,
    rand: &mut R,
) where
    B: BBox,
    B::Num: PartialOrd + HasInfinity,
    ID: PartialOrd + Copy,
    R: Rng,
{
    const CUTOFF: usize = 1000; // should give reasonable performance for up to 100,000 boxes
    intersect_ze_custom::<B, ID, R, CUTOFF>(a, b, out, rand);
}

/// Like `intersect_ze` but with a customizable cutoff.
pub fn intersect_ze_custom<B, ID, R, const CUTOFF: usize>(
    a: &BBoxSet<B, ID>,
    b: &BBoxSet<B, ID>,
    out: &mut Vec<(ID, ID)>,
    rand: &mut R,
) where
    B: BBox,
    ID: PartialOrd + Copy,
    B::Num: PartialOrd + HasInfinity,
    ID: PartialEq,
    R: Rng,
{
    let same = a as *const _ == b as *const _;
    if same {
        // one tree is enough to have every box represented as both an interval and a point
        hybrid::<B, ID, R, CUTOFF>(a, a, B::Num::NINFTY, B::Num::INFTY, B::DIM - 1, out, rand);
    } else {
        // need two trees so that every box is represented as both an interval and a point
        hybrid::<B, ID, R, CUTOFF>(a, b, B::Num::NINFTY, B::Num::INFTY, B::DIM - 1, out, rand);
        hybrid::<B, ID, R, CUTOFF>(b, a, B::Num::NINFTY, B::Num::INFTY, B::DIM - 1, out, rand);
    }
}

/// Finds all intersections between boxes in `a` and `b` using a scanning algorithm.
/// Should perform reasonably up to approximately 1,000 boxes
/// * `a` and `b` may be either the same or distinct [`BBoxSet`]s and must be sorted before calling.
/// * `out` will contain pairs of `ID`s of intersecting boxes.
pub fn intersect_scan<B, ID>(a: &BBoxSet<B, ID>, b: &BBoxSet<B, ID>, out: &mut Vec<(usize, usize)>)
where
    B: BBox,
    ID: Copy + PartialOrd,
{
    let same = a as *const _ == b as *const _; // check if a and b refer to the same BBoxSet
    if same {
        one_way_scan(a, b, B::DIM - 1, out);
    } else {
        two_way_scan(a, b, out);
    }
}

/// Finds box intersections by checking every box in `a` against every box in `b`.
/// Performs well for on the order of 100 boxes. *O*(*n^2*)
/// * `a` and `b` may be either the same or distinct [`BBoxSet`]s
/// * `out` will contain pairs of `ID`s of intersecting boxes.
pub fn intersect_brute_force<B, ID>(a: &BBoxSet<B, ID>, b: &BBoxSet<B, ID>, out: &mut Vec<(ID, ID)>)
where
    B: BBox,
    ID: Copy,
{
    let same = a as *const _ == b as *const _; // check if a and b refer to the same BBoxSet
    if same {
        // avoid duplicate intersections
        let mut start = 1;
        for &(bbox, id) in &a.boxes {
            for idx in start..a.boxes.len() {
                let (bbox2, id2) = a.boxes[idx];
                if bbox.intersects(&bbox2) {
                    out.push((id, id2));
                }
            }
            start += 1;
        }
    } else {
        for &(bbox, id) in &a.boxes {
            for &(bbox2, id2) in &b.boxes {
                if bbox.intersects(&bbox2) {
                    out.push((id, id2));
                }
            }
        }
    }
}

impl HasInfinity for f32 {
    const NINFTY: Self = f32::NEG_INFINITY;
    const INFTY: Self = f32::INFINITY;
}

impl HasInfinity for f64 {
    const NINFTY: Self = f64::NEG_INFINITY;
    const INFTY: Self = f64::INFINITY;
}

#[cfg(test)]
mod tests;
