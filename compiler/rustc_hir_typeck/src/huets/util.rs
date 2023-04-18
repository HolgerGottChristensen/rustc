
pub fn amount_of_swaps_to_sort(list: Vec<usize>) -> usize {
    let mut indexed_list = Vec::new();
    for i in 0..list.len() {
        indexed_list.push((list[i], i));
    }
    indexed_list.sort_by(|(a, _), (b, _)| a.cmp(b));
    let mut marked = vec![false; indexed_list.len()];
    let mut result = 0;

    for i in 0..indexed_list.len() {
        if marked[i] || indexed_list[i].1 == i {
            continue;
        }

        let mut cycle_size = 0;
        let mut j = i;
        while !marked[j] {
            marked[j] = true;
            j = indexed_list[j].1;
            cycle_size += 1;
        }

        if cycle_size > 0 {
            result += cycle_size - 1;
        }
    }

    result
}
