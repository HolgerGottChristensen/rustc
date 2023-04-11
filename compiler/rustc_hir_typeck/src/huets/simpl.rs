use crate::huets::datatype::{Constraint, Context, Problem};

pub fn simpl(context: Context, problem: Problem) -> Option<Problem> {
    let mut queue = problem.0.clone();
    let mut simplified = vec![];

    while let Some(constraint) = queue.pop() {
        if constraint.is_rigid_rigid() {
            queue.append(&mut simplify_constraint(context.clone(), constraint)?);
        } else {
            simplified.push(constraint);
        }
    }

    Some(Problem(simplified))
}

fn simplify_constraint(context: Context, constraint: Constraint) -> Option<Vec<Constraint>> {
    let (l_lambda, l_head, l_tail) = constraint.left.split();
    let (r_lambda, r_head, r_tail) = constraint.right.split();

    if l_head.equal_in_context(&r_head, &context.typing_context) ||
        l_head.binding_index(&l_lambda).and_then(|u| {
            r_head.binding_index(&r_lambda).map(|u1| u1 == u)
        }).unwrap_or(false) {
        let mut builder = vec![];

        for (l_elem, r_elem) in l_tail.into_iter().zip(r_tail.into_iter()) {
            builder.push(Constraint {
                left: l_elem.combine(l_lambda.clone()),
                right: r_elem.combine(r_lambda.clone())
            })
        }

        Some(builder)
    } else {
        None
    }
}
