use itertools::Itertools;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{cmp::Ordering, fs::File, io::Write, ops::Div, time, time::Duration};

fn main() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut res = String::new();

    let arr_sizes = [100, 1000, 10000, 100000, 1000000, 10000000];
    let group_sizes = [5, 7, 9, 11, 13];

    res.push_str("  Elements");
    for group_size in group_sizes {
        res.push_str(&format!("\t{group_size}"))
    }
    res.push('\n');

    for arr_size in arr_sizes {
        res.push_str(&format!("{arr_size:10}"));

        let base_arrays = (0..100)
            .map(|_| {
                let mut arr = vec![0_u32; arr_size];
                rng.fill(&mut arr[..]);
                arr
            })
            .collect_vec();

        let ks = base_arrays
            .iter()
            .map(|arr| rng.gen_range(0..arr.len()))
            .collect_vec();

        for group_size in group_sizes {
            let elapsed_nanos = elapsed_for_group_size(base_arrays.clone(), &ks, group_size).as_nanos() as f64;
            let normalized_elapsed_nanos = elapsed_nanos.div((arr_size * base_arrays.len()) as f64);
            res.push_str(&format!("\t{normalized_elapsed_nanos:.2}"));
            println!("Groups of {group_size} took {normalized_elapsed_nanos:.2} ns per element for {arr_size} elements");
        }

        res.push('\n');
    }

    let mut file = File::create("out.txt").unwrap();
    file.write_all(res.as_bytes()).unwrap();
}

fn elapsed_for_group_size(mut arrays: Vec<Vec<u32>>, ks: &[usize], group_size: usize) -> Duration {
    let arr_len = arrays[0].len();
    let start = time::Instant::now();

    for (arr, k) in arrays.iter_mut().zip(ks) {
        quick_select(&mut arr[..], 0, arr_len - 1, *k, group_size);
    }

    start.elapsed()
}

/// https://en.wikipedia.org/wiki/Quickselect
fn quick_select(list: &mut [u32], left: usize, right: usize, k: usize, group_size: usize) -> usize {
    let mut left = left;
    let mut right = right;
    loop {
        if left == right {
            return left;
        }

        let pivot_index = median_of_medians(list, left, right, group_size);
        let pivot_index = partition(list, left, right, pivot_index);

        match k.cmp(&pivot_index) {
            Ordering::Equal => return k,
            Ordering::Less => right = pivot_index - 1,
            Ordering::Greater => left = pivot_index + 1,
        }
    }
}

/// https://en.wikipedia.org/wiki/Median_of_medians
fn median_of_medians(list: &mut [u32], left: usize, right: usize, group_size: usize) -> usize {
    if right - left < group_size {
        return median_by_sort(list, left, right);
    }

    for i in (left..=right).step_by(group_size) {
        let mut sub_right = i + group_size - 1;
        if sub_right > right {
            sub_right = right
        }
        let median = median_by_sort(list, i, sub_right);
        list.swap(median, left + (i - left) / group_size);
    }

    let mid = (right - left) / group_size / 2 + left + 1;
    quick_select(
        list,
        left,
        left + (right - left) / group_size,
        mid,
        group_size,
    )
}

fn median_by_sort(list: &mut [u32], left: usize, right: usize) -> usize {
    list[left..=right].sort(); // is O(n * log(n)) in the worst case

    (left + right) / 2
}

/// https://en.wikipedia.org/wiki/Quickselect
fn partition(list: &mut [u32], left: usize, right: usize, pivot_index: usize) -> usize {
    let pivot_value = list[pivot_index];
    list.swap(pivot_index, right);
    let mut store_index = left;

    for i in left..right {
        if list[i] < pivot_value {
            list.swap(store_index, i);
            store_index += 1;
        }
    }
    list.swap(right, store_index);

    store_index
}

#[cfg(test)]
mod tests {
    use crate::quick_select;
    use itertools::Itertools;
    use rand::{seq::SliceRandom, thread_rng, Rng};

    const ARR_LEN: usize = 1024;

    #[test]
    fn fuzz_correct_median() {
        let mut rng = thread_rng();
        let mut arr = [0; ARR_LEN];

        for _ in 0..1000 {
            rng.fill(&mut arr);

            let k = rng.gen_range(1_usize..ARR_LEN / 2);
            let group_size = *[5_usize, 7, 9, 11].choose(&mut rng).unwrap();

            let mut quick_select_arr = arr;
            let quick_select_k_smallest_idx =
                quick_select(&mut quick_select_arr, 0, ARR_LEN - 1, k, group_size);
            let quick_select_k_smallest = quick_select_arr[quick_select_k_smallest_idx];

            let mut sort_select_arr = arr.into_iter().unique().collect_vec();
            sort_select_arr.sort();
            let sort_select_k_smallest = sort_select_arr[k];

            assert_eq!(sort_select_k_smallest, quick_select_k_smallest);
        }
    }
}
