# box_intersect_ze
[![Latest version](https://img.shields.io/crates/v/box_intersect_ze.svg)](https://crates.io/crates/box_intersect_ze)
[![Documentation](https://docs.rs/box_intersect_ze/badge.svg)](https://docs.rs/box_intersect_ze)

box_intersect_ze is a broad phase collision detection library implementing the algorithm
described in the paper [Fast software for box intersections](https://dl.acm.org/doi/10.1145/336154.336192)
by Afra Zomorodian and Herbert Edelsbrunner. The algorithm uses streamed segment trees, pruning and scanning.  
It should be well suited for video games, where bounding boxes change frequently and any spatial data structure
kept in memory will be frequently invalidated.  
The algorithm requires a random number generator. With the `rand-crate` optional feature you can use
any RNG from the [rand](https://crates.io/crates/rand) crate, or you can implement the `Rng` trait for your own RNG if
you don't want the dependency.

## Example

```rust
use box_intersect_ze::set::BBoxSet;
use box_intersect_ze::boxes::Box3Df32;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

// create some boxes
let box0 = Box3Df32::new([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
let box1 = Box3Df32::new([5.0, 5.0, 5.0], [15.0, 15.0, 15.0]);
let box2 = Box3Df32::new([10.0, 10.0, 10.0], [20.0, 20.0, 20.0]);

// add them to a BBoxSet
let mut boxes = BBoxSet::with_capacity(3);
boxes.push(0, box0);
boxes.push(1, box1);
boxes.push(2, box2);
boxes.sort(); // sort it in dimension 0

// set capacity according to expected number of intersections to avoid resizing
let mut result = Vec::with_capacity(2);
// get the intersections
box_intersect_ze::intersect_ze(&boxes, &boxes, &mut result, &mut ChaCha8Rng::seed_from_u64(1234));

assert!(result.contains(&(1,0)));
assert!(result.contains(&(2,1)));
assert!(!result.contains(&(2,0)));
assert!(!result.contains(&(0,2)));
```

