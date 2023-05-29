//! Implementations of the algorithms provided by this crate. You probably want to call
//! the wrappers at the [top level of the crate](`crate`) instead.

use crate::boxes::BBox;
use crate::set::BBoxSet;
use crate::{HasInfinity, Rng};
use crate::AnswerFormat;
/// Reports intersections between `intervals` and `points` by scanning in dimension 0,
/// treating boxes in `points` as points: intersections are only reported when the low
/// endpoint in dimension 0 of a box in `points` is inside the projection of a box in `intervals`.
/// * `intervals` and `points` must be sorted before calling
/// * `max_dim_check`: highest dimension that should be checked for intersection
/// * `out` will contain pairs of `ID`s of intersecting boxes.
pub fn one_way_scan<'a, B, ID>(
    intervals: &BBoxSet<B, ID>,
    points: &BBoxSet<B, ID>,
    max_dim_check: usize,
    mut out:  AnswerFormat<'a, ID>,
) where
    B: BBox,
    ID: Copy + PartialOrd,
    B::Num: PartialOrd,
{
    let p_len = points.len();
    let mut p_min_idx = 0;

    // iterate through (sorted) intervals
    for (i_idx, i) in intervals.boxes.iter().enumerate() {
        let &(i, i_id) = i;
        let i_min = i.lo(0);
        let i_max = i.hi(0);

        //skip all points that don't have a chance to be in `i`
        while p_min_idx < p_len && points.boxes[p_min_idx].0.lo(0) < i_min {
            p_min_idx += 1;
        }
        // if no point has a chance to be in the current interval,
        // they can't be in any of the remaining ones either (because they are sorted)
        if p_min_idx == p_len {
            return;
        }

        'points: for p_idx in p_min_idx..p_len {
            let (p, p_id) = points.boxes[p_idx];
            let p_min = p.lo(0);
            if p_min >= i_max {
                break 'points;
            }

            if p_id == i_id {
                continue 'points;
            }

            for dim in 1..max_dim_check + 1 {
                if !p.intersects_in(dim, i.lo(dim), i.hi(dim)) {
                    continue 'points;
                }
            }

            //if low endpoints are not pairwise different, this is needed to avoid duplicates
            if p_min == i_min && p_id > i_id {
                continue 'points;
            }

            match out{
                    AnswerFormat::Index(ref mut  out) => {out.push((i_idx, p_idx));}
                    AnswerFormat::Ident( ref mut  out) => {out.push((i_id, p_id));}
                    AnswerFormat::Both( ref mut out) => {out.push(((p_idx, p_min_idx),(i_id, p_id)));}
                }
        }
    }
}

/// Reports intersections between `intervals` and `points` by scanning in dimension 0 (because that's where boxes are sorted),
/// but pretends it was scanning in dimension `max_dim_check` by treating `points` as points there, as in [`one_way_scan`]
pub fn simulated_one_way_scan<'a, B, ID>(
    intervals: &BBoxSet<B, ID>,
    points: &BBoxSet<B, ID>,
    max_dim_check: usize,
    out: AnswerFormat<'a, ID>,
) where
    B: BBox,
    ID: Copy + PartialOrd,
    B::Num: PartialOrd,
{
    _two_way_scan::<B, ID, true>(intervals, points, max_dim_check, out);
}

/// Reports intersections between boxes in `a` and `b` by scanning in dimension 0, treating each
/// as intervals and points in turn, as if [`one_way_scan`] was called twice, once with intervals and points switched
/// * `a` and `b` must be distinct [`BBoxSet`]s and must be sorted before calling.
/// * `out` will contain pairs of `ID`s of intersecting boxes.
pub fn two_way_scan<'a, B, ID>(a: &BBoxSet<B, ID>, b: &BBoxSet<B, ID>,  out: AnswerFormat<'a, ID>)
where
    B: BBox,
    ID: Copy,
    B::Num: PartialOrd,
    ID: PartialOrd,
{
    _two_way_scan::<B, ID, false>(a, b, B::DIM - 1, out);
}

fn _two_way_scan<'a, B, ID, const SIMULATE_ONE_WAY: bool>(
    intervals: &BBoxSet<B, ID>,
    points: &BBoxSet<B, ID>,
    max_dim_check: usize,
    mut out: AnswerFormat<'a, ID>,
) where
    B: BBox,
    ID: Copy + PartialOrd,
    B::Num: PartialOrd,
{
    let mut i_min_idx = 0;
    let i_len = intervals.len();
    let mut p_min_idx = 0;
    let p_len = points.len();

    // exclusive upper bound of dimensions to be checked for intersection
    let dim_range_upper = if SIMULATE_ONE_WAY {
        max_dim_check // simulated one way scan employs a stricter check than just intersection for the highest dimension
    } else {
        max_dim_check + 1
    };

    while i_min_idx < i_len && p_min_idx < p_len {
        let (i_min, i_min_id) = intervals.get(i_min_idx);
        let (p_min, p_min_id) = points.get(p_min_idx);
        if i_min.lo(0) < p_min.lo(0) {
            'points: for p_idx in p_min_idx..p_len {
                let (p, p_id) = points.get(p_idx);
                if p.lo(0) >= i_min.hi(0) {
                    break 'points;
                }

                if p_id == i_min_id {
                    continue 'points;
                }

                for dim in 1..dim_range_upper {
                    if !p.intersects_in(dim, i_min.lo(dim), i_min.hi(dim)) {
                        continue 'points;
                    }
                }

                if SIMULATE_ONE_WAY
                    && (!i_min.contains_in(max_dim_check, p.lo(max_dim_check))
                        || (i_min.lo(max_dim_check) == p.lo(max_dim_check) && i_min_id > p_id))
                {
                    continue 'points;
                }
                match out{
                    AnswerFormat::Index(ref mut out) => {out.push((p_idx, i_min_idx));}
                    AnswerFormat::Ident(ref mut out) => {out.push((i_min_id, p_id));}
                    AnswerFormat::Both(ref mut out) => {out.push(((p_idx, i_min_idx),(i_min_id, p_id)));}
                }
                //out.push((p_id, i_min_id));

            }

            i_min_idx += 1;
        } else {
            //p_min.lo(0) <= i_min.lo(0), so switch the roles of intervals and points
            'intervals: for i_idx in i_min_idx..i_len {
                let (i, i_id) = intervals.get(i_idx);
                if i.lo(0) >= p_min.hi(0) {
                    break 'intervals;
                }

                if i_id == p_min_id {
                    continue 'intervals;
                }

                for dim in 1..dim_range_upper {
                    if !i.intersects_in(dim, p_min.lo(dim), p_min.hi(dim)) {
                        continue 'intervals;
                    }
                }

                if SIMULATE_ONE_WAY
                    && (!i.contains_in(max_dim_check, p_min.lo(max_dim_check))
                        || (i.lo(max_dim_check) == p_min.lo(max_dim_check) && i_id > p_min_id))
                {
                    continue 'intervals;
                }

              match out{
                AnswerFormat::Index(ref mut out) => {out.push((p_min_idx, i_idx));}
                AnswerFormat::Ident(ref mut out) => {out.push((p_min_id, i_id));}
                AnswerFormat::Both(ref mut out) => {out.push(((p_min_idx, i_idx),(p_min_id,  i_id)));}
            }


            }

            p_min_idx += 1;
        }
    }
}

/// Streams a segment tree to check if the boxes in `intervals` intersect those in `points`,
/// treating the latter as points in dimension `dim`: intersections are only reported when the low
/// endpoint in dimension `dim` of a box in `points` is inside the projection of a box in `intervals`.
/// If `dim > 0`, will recursively stream two segment trees in dimension `dim - 1`, so that
/// each box will be treated both as an `interval` and as a `point`.
/// * [`lo`, `hi`) is the segment belonging to this node of the streamed segment tree
/// * `out` will contain pairs of `ID`s of intersecting boxes.
pub fn hybrid<B, ID, R, const CUTOFF: usize>(
    intervals: &BBoxSet<B, ID>,
    points: &BBoxSet<B, ID>,
    lo: B::Num,
    hi: B::Num,
    dim: usize,
    out: &mut Vec<(ID, ID)>,
    rand: &mut R,
) where
    B: BBox,
    ID: PartialOrd + Copy,
    B::Num: PartialOrd + HasInfinity,
    R: Rng,
{
    hybrid_flex::<B,ID,R,CUTOFF>(intervals,points,lo, hi,dim, AnswerFormat::Ident(out), rand);
}

pub fn hybrid_flex<'a, B, ID, R, const CUTOFF: usize>(
    intervals: &BBoxSet<B, ID>,
    points: &BBoxSet<B, ID>,
    lo: B::Num,
    hi: B::Num,
    dim: usize,
    mut out: AnswerFormat<'a, ID>,
    rand: &mut R,
) where
B: BBox,
ID: PartialOrd + Copy,
B::Num: PartialOrd + HasInfinity,
R: Rng,
{
    //use reborrow::ReborrowMut;
    //impl <'a, ID> Copy for AnswerFormat<'a, ID>{}
    // The steps of the algorithm are numbered as in the paper "Fast software for box intersections":
    // https://dl.acm.org/doi/10.1145/336154.336192

    // Step 1: return if input is empty
    if intervals.empty() || points.empty() || hi <= lo {
        return;
    }

    // Step 2: first hybridization method: scan if only dimension 0 is left to check
    if dim == 0 {
        one_way_scan(intervals, points, 0, out);
        return;
    }

    // Step 3: second hybridization method: scan if size of input is smaller than cutoff
    if intervals.len() < CUTOFF || points.len() < CUTOFF {
        simulated_one_way_scan(intervals, points, dim, out);
        return;
    }

    // Step 4: let intervals_m contain the intervals that would be stored at this node of the segment tree
    // because they span the segment [lo, hi), meaning it is one of their canonical segments
    // let intervals_lr contain the intervals not stored at this node
    let (intervals_m, intervals_lr) =
        intervals.partition(|(i, _)| i.lo(dim) < lo && i.hi(dim) > hi);
    let (ninfty, infty) = (B::Num::NINFTY, B::Num::INFTY);

    // Step 4: stream two segment trees in the next dimension for the intervals stored at this node

    hybrid_flex::<B, ID, R, CUTOFF>(&intervals_m, points, ninfty, infty, dim - 1, out.reborrow(), rand);


    hybrid_flex::<B, ID, R, CUTOFF>(points, &intervals_m, ninfty, infty, dim - 1, out.reborrow(), rand);

    // Step 5: divide the segment [lo, hi) into segments [lo, mi) and [mi, hi) by computing an approximate median
    let mi = points.approx_median(dim, rand);

    // if we failed to divide the segment into subsegments, just scan instead
    if mi == hi || mi == lo {
        simulated_one_way_scan(&intervals_lr, points, dim, out.reborrow());
        return;
    }

    // let points_l contain the points in the left subsegment [lo, mi),
    // points_r those in the right subsegment [mi, hi)
    let (points_l, points_r) = points.partition(|(p, _)| p.lo(dim) < mi);
    let len = intervals_lr.len();
    let (mut intervals_l, mut intervals_r) =
        (BBoxSet::with_capacity(len), BBoxSet::with_capacity(len));

    // let intervals_l/r contain the intervals stored somewhere in the left/right subtree
    // because they intersect [lo, mi)/[mi, hi) but don't span [lo, hi)
    // intervals_l and intervals_r are not usually disjoint!
    for &(i, id) in &intervals_lr.boxes {
        if i.lo(dim) < mi {
            intervals_l.push(id, i);
        }

        if i.hi(dim) > mi {
            intervals_r.push(id, i);
        }
    }

    hybrid_flex::<B, ID, R, CUTOFF>(&intervals_l, &points_l, lo, mi, dim, out.reborrow(), rand); // Step 6: left subtree
    hybrid_flex::<B, ID, R, CUTOFF>(&intervals_r, &points_r, mi, hi, dim, out.reborrow(), rand); // Step 7: right subtree
}
