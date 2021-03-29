use box_intersect_ze::boxes::Box3Df32;
use box_intersect_ze::set::BBoxSet;
use box_intersect_ze::{intersect_brute_force, intersect_scan, intersect_ze_custom};
use rand::{Rng as OtherRng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::time::Instant;

fn random_boxes(n: usize, start: usize, seed: u64) -> BBoxSet<Box3Df32, usize> {
    let mut r = ChaCha8Rng::seed_from_u64(seed);
    let mut set = BBoxSet::with_capacity(n);
    let nf = n as f32;
    let len_max = nf.powf(2.0 / 3.0).floor() as usize;
    let lo_max = n - len_max;
    for i in start..n {
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

fn bench_intersect<const C: usize>(boxes: &BBoxSet<Box3Df32, usize>) {
    let mut r = ChaCha8Rng::seed_from_u64(12345);

    let now = Instant::now();
    let mut result = Vec::with_capacity(boxes.len());
    intersect_ze_custom::<_, _, _, C>(&boxes, &boxes, &mut result, &mut r);
    print!("{},", (now.elapsed()).as_micros());
    assert!(result.len() < boxes.len()); //want to benchmark the algorithm, not vector resizing
}

fn bench_scan(boxes: &BBoxSet<Box3Df32, usize>) {
    let mut result = Vec::with_capacity(boxes.len());

    let now = Instant::now();
    intersect_scan(boxes, boxes, &mut result);
    print!("{},", (now.elapsed()).as_micros());
}

fn bench_bruteforce(boxes: &BBoxSet<Box3Df32, usize>) {
    let mut result = Vec::with_capacity(boxes.len());

    let now = Instant::now();
    intersect_brute_force(boxes, boxes, &mut result);
    print!("{}", (now.elapsed()).as_micros());
}

/// Print some benchmarking results for the different algorithms
/// Times are in microseconds and don't include sorting, which is measured separately
fn main() {
    let sizes: [usize; 17] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 20, 30, 40, 50, 100, 500, 1000,
    ];
    println!("count,sort,ze10,ze100,ze1000,ze2000,scan,bruteforce");
    for &size in sizes.iter() {
        let size = size * 100;
        print!("{}, ", size);

        let mut boxes = random_boxes(size, 0, 12345);

        let now = Instant::now();
        boxes.sort();
        let sorttime = now.elapsed();
        print!("{},", sorttime.as_micros());

        bench_intersect::<10>(&boxes);
        bench_intersect::<100>(&boxes);
        bench_intersect::<1000>(&boxes);
        bench_intersect::<2000>(&boxes);
        bench_scan(&boxes);
        if size < 100000 {
            bench_bruteforce(&boxes);
        }
        println!();
    }
}
