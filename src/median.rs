pub fn median_of_3<N: PartialOrd + Copy>(a: N, b: N, c: N) -> N {
    if a > b {
        if b > c {
            // b is in the middle
            b
        } else {
            // b is the smallest
            if a > c {
                c
            } else {
                a
            }
        }
    } else {
        if b < c {
            //b is in the middle
            b
        } else {
            //b is the largest
            if a > c {
                a
            } else {
                c
            }
        }
    }
}

pub fn approx_median<N: PartialOrd + Copy>(
    items: &Vec<N>,
    levels: u8,
    random_indices: &mut Vec<usize>,
) -> N {
    if levels == 0 {
        items[random_indices.pop().unwrap()]
    } else {
        median_of_3(
            approx_median(items, levels - 1, random_indices),
            approx_median(items, levels - 1, random_indices),
            approx_median(items, levels - 1, random_indices),
        )
    }
}
