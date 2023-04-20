use crate::huets::datatype::{Solution, SolutionSet};

pub fn get_solution_from_solution_set(solutions: SolutionSet) -> Result<Solution, SolutionSet> {
    let existence_filtered = existence(solutions);
    let generality_filtered = generality(existence_filtered);
    let exhaustiveness_filtered = exhaustiveness(generality_filtered);
    let ordering_filtered = ordering(exhaustiveness_filtered);
    let simplicity_filterered = simplicity(ordering_filtered);

    if simplicity_filterered.0.len() == 1 {
        Ok(simplicity_filterered.0[0].clone())
    } else {
        Err(simplicity_filterered)
    }
}

fn existence(mut solutions: SolutionSet) -> SolutionSet {
    solutions.0.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    let mut new_list = Vec::new();
    if let Some(first) = solutions.0.first() {
        let max = first.0.len();

        while let Some(elem) = solutions.0.pop() {
            if elem.0.len() == max {
                new_list.push(elem);
            }
        }
    }
    SolutionSet(new_list)
}

fn generality(mut solutions: SolutionSet) -> SolutionSet {
    solutions.0.sort_by(|a, b| {
        a.number_of_constants().cmp(&b.number_of_constants())
    });

    let mut new_list = Vec::new();
    if let Some(first) = solutions.0.first() {
        let max = first.number_of_constants();

        while let Some(elem) = solutions.0.pop() {
            if elem.number_of_constants() == max {
                new_list.push(elem);
            }
        }
    }

    SolutionSet(new_list)
}

fn exhaustiveness(mut solutions: SolutionSet) -> SolutionSet {
    solutions.0.sort_by(|a, b| {
        b.number_of_unique_params().cmp(&a.number_of_unique_params())
    });

    let mut new_list = Vec::new();
    if let Some(first) = solutions.0.first() {
        let max = first.number_of_unique_params();

        while let Some(elem) = solutions.0.pop() {
            if elem.number_of_unique_params() == max {
                new_list.push(elem);
            }
        }
    }

    SolutionSet(new_list)
}

fn ordering(mut solutions: SolutionSet) -> SolutionSet {
    solutions.0.sort_by(|a, b| {
        // FIXMIG : what to we sort by?
        a.number_of_swaps().cmp(&b.number_of_swaps())
    });

    let mut new_list = Vec::new();
    if let Some(first) = solutions.0.first() {
        let max = first.number_of_swaps();

        while let Some(elem) = solutions.0.pop() {
            if elem.number_of_swaps() == max {
                new_list.push(elem);
            }
        }
    }

    SolutionSet(new_list)
}

fn simplicity(mut solutions: SolutionSet) -> SolutionSet {
    solutions.0.sort_by(|a, b| {
        // FIXMIG : what to we sort by?
        a.number_of_params().cmp(&b.number_of_params())
    });

    let mut new_list = Vec::new();
    if let Some(first) = solutions.0.first() {
        let max = first.number_of_params();

        while let Some(elem) = solutions.0.pop() {
            if elem.number_of_params() == max {
                new_list.push(elem);
            }
        }
    }

    SolutionSet(new_list)
}
