use crate::boxes::Box3Df32;
use crate::intersect_brute_force;
use crate::set::BBoxSet;
use rand::{Rng as OtherRng, SeedableRng};
use std::fmt::Debug;

fn same<ID: Eq + Copy + Debug>(a: &Vec<(ID, ID)>, b: &Vec<(ID, ID)>) -> bool {
    let (long, short, short_name) = if a.len() > b.len() {
        (a, b, "second")
    } else {
        (b, a, "first")
    };

    let mut missing = false;
    for el in long {
        if !(short.contains(el) || short.contains(&(el.1, el.0))) {
            println!("Missing in {}: {:?}", short_name, el);
            missing = true;
        }
    }

    if a.len() != b.len() {
        for el in short {
            if !(long.contains(el) || long.contains(&(el.1, el.0))) {
                println!("Missing in long: {:?}", el);
                missing = true;
            }
        }
        println!("First: {}, second: {}", a.len(), b.len());
    }

    a.len() == b.len() && !missing
}

fn random_boxes(n: usize, start: usize, seed: u64) -> BBoxSet<Box3Df32, usize> {
    let mut r = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
    let mut set = BBoxSet::with_capacity(n);
    let nf = n as f32;
    let len_max = nf.powf(2.0 / 3.0).floor() as usize;
    let lo_max = n - len_max;
    for i in start..start + n {
        let mut min = [0.0; 3];
        let mut max = [0.0; 3];
        for d in 0..min.len() {
            let lo = r.gen_range(1..lo_max as u32);
            let hi = lo + r.gen_range(1..len_max as u32);
            min[d] = lo as f32;
            max[d] = hi as f32;
        }

        set.push(i, Box3Df32::new(min, max));
    }
    set
}

/// Generates some random boxes and finds their intersections using brute force, as a reference to validate against
fn test_data() -> (
    BBoxSet<Box3Df32, usize>,
    BBoxSet<Box3Df32, usize>,
    Vec<(usize, usize)>,
    Vec<(usize, usize)>,
) {
    let boxes = random_boxes(150, 0, 12345);
    let boxes2 = random_boxes(150, boxes.len(), 54321);

    let mut res = Vec::<(usize, usize)>::with_capacity(80);
    intersect_brute_force(&boxes, &boxes, &mut res);

    let mut res2 = Vec::<(usize, usize)>::with_capacity(80);
    intersect_brute_force(&boxes, &boxes2, &mut res2);

    assert_ne!(res.len(), 0);
    assert_ne!(res2.len(), 0);

    (boxes, boxes2, res, res2)
}

// Validate the different algorithms against the brute force solution

#[test]
fn one_way_scan() {
    let (mut boxes, _boxes2, boxes_self, _boxes_boxes2) = test_data();

    let mut res = Vec::<(usize, usize)>::with_capacity(80);
    boxes.sort();
    crate::internals::one_way_scan(&boxes, &boxes, 2, &mut res);

    assert!(same(&boxes_self, &res));
}

#[test]
fn simulated_one_way_scan() {
    let (mut boxes, _boxes2, boxes_self, _boxes_boxes2) = test_data();

    let mut res = Vec::<(usize, usize)>::with_capacity(80);
    boxes.sort();
    crate::internals::simulated_one_way_scan(&boxes, &boxes, 2, &mut res);

    assert!(same(&boxes_self, &res));
}

#[test]
fn two_way_scan() {
    let (mut boxes, mut boxes2, _boxes_self, boxes_boxes2) = test_data();

    let mut res = Vec::<(usize, usize)>::with_capacity(80);
    boxes.sort();
    boxes2.sort();
    crate::internals::two_way_scan(&boxes, &boxes2, &mut res);

    assert!(same(&boxes_boxes2, &res));
}

#[test]
fn box_intersect() {
    let (mut boxes, mut boxes2, boxes_self, boxes_boxes2) = test_data();

    let mut res = Vec::<(usize, usize)>::with_capacity(boxes.len());
    let mut r = rand_chacha::ChaCha8Rng::seed_from_u64(12345);

    boxes.sort();
    boxes2.sort();
    crate::intersect_ze_custom::<_, _, _, 5>(&boxes, &boxes, &mut res, &mut r);

    for &(id1, id2) in &res {
        if res.contains(&(id2, id1)) {
            println!("duplicate: {:?}", (id1, id2))
        }
    }

    for (idx, &(id1, id2)) in res.iter().enumerate() {
        for i in idx + 1..res.len() {
            if res[i] == (id1, id2) {
                println!("duplicate: {:?}", (id1, id2))
            }
        }
    }

    assert!(same(&boxes_self, &res));

    let mut res = Vec::<(usize, usize)>::with_capacity(boxes.len());
    crate::intersect_ze_custom::<_, _, _, 5>(&boxes, &boxes2, &mut res, &mut r);

    assert!(same(&boxes_boxes2, &res));
}
