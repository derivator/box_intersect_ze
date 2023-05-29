use crate::boxes::Box3Df32;
use crate::intersect_brute_force;
use crate::set::BBoxSet;
use once_cell::sync::Lazy;
use rand::{Rng as OtherRng, SeedableRng};
use std::fmt::Debug;

fn same<ID: Eq + Copy + Debug>(correct: &Vec<(ID, ID)>, actual: &Vec<(ID, ID)>) -> bool {
    let mut missing = false;
    for el in correct {
        if !(actual.contains(el) || actual.contains(&(el.1, el.0))) {
            println!("Missing element: {:?}", el);
            missing = true;
        }
    }

    if correct.len() != actual.len() {
        for el in actual {
            if !(correct.contains(el) || correct.contains(&(el.1, el.0))) {
                println!("Incorrect element: {:?}", el);
                missing = true;
            }
        }
        println!("Correct: {}, observed: {}", correct.len(), actual.len());
    }

    correct.len() == actual.len() && !missing
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
struct TestData {
    boxes1: BBoxSet<Box3Df32, usize>,
    boxes2: BBoxSet<Box3Df32, usize>,
    complete: Vec<(usize, usize)>,
    bipartite: Vec<(usize, usize)>,
}

static TEST_DATA: Lazy<TestData> = Lazy::new(|| test_data());
/// Generates some random boxes and finds their intersections using brute force, as a reference to validate against
fn test_data() -> TestData {
    let mut boxes1 = random_boxes(150, 0, 12345);
    let mut boxes2 = random_boxes(150, boxes1.len(), 54321);
    boxes1.sort();
    boxes2.sort();

    let mut complete = Vec::<(usize, usize)>::with_capacity(80);
    intersect_brute_force(&boxes1, &boxes1, &mut complete);

    let mut bipartite = Vec::<(usize, usize)>::with_capacity(80);
    intersect_brute_force(&boxes1, &boxes2, &mut bipartite);

    assert_ne!(complete.len(), 0);
    assert_ne!(bipartite.len(), 0);

    TestData {
        boxes1,
        boxes2,
        complete,
        bipartite,
    }
}

// Validate the different algorithms against the brute force solution

#[test]
fn one_way_scan() {
    let mut res = Vec::<(usize, usize)>::with_capacity(80);
    crate::internals::one_way_scan(&TEST_DATA.boxes1, &TEST_DATA.boxes1, 2, crate::AnswerFormat::Ident(&mut res));
    assert!(same(&TEST_DATA.complete, &res));
}

#[test]
fn simulated_one_way_scan() {
    let mut res = Vec::<(usize, usize)>::with_capacity(80);
    crate::internals::simulated_one_way_scan(&TEST_DATA.boxes1, &TEST_DATA.boxes1, 2, crate::AnswerFormat::Ident(&mut res));

    assert!(same(&TEST_DATA.complete, &res));
}

#[test]
fn two_way_scan() {
    let mut res = Vec::<(usize, usize)>::with_capacity(80);
    crate::internals::two_way_scan(&TEST_DATA.boxes1, &TEST_DATA.boxes2, crate::AnswerFormat::Ident(&mut res));

    assert!(same(&TEST_DATA.bipartite, &res));
}

#[test]
fn box_intersect() {
    let mut res = Vec::<(usize, usize)>::with_capacity(TEST_DATA.complete.len());
    let mut r = rand_chacha::ChaCha8Rng::seed_from_u64(12345);

    crate::intersect_ze_custom::<_, _, _, 5>(
        &TEST_DATA.boxes1,
        &TEST_DATA.boxes1,
        &mut res,
        &mut r,
    );

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

    assert!(same(&TEST_DATA.complete, &res));

    let mut res = Vec::<(usize, usize)>::with_capacity(TEST_DATA.bipartite.len());
    crate::intersect_ze_custom::<_, _, _, 5>(
        &TEST_DATA.boxes1,
        &TEST_DATA.boxes2,
        &mut res,
        &mut r,
    );

    assert!(same(&TEST_DATA.bipartite, &res));
}
