use crate::huets::datatype::{Constraint, Context, generate_fresh_var, Substitution, Term, Type};

pub fn match_(context: Context, constraint: Constraint) -> Vec<Substitution> {
    let mut res = vec![];
    res.push(imitation(&context, &constraint));
    res.append(&mut projection(&context, &constraint));
    res
}

pub fn imitation(_context: &Context, constraint: &Constraint) -> Substitution {
    let (_, x, l_tail) = constraint.left.split();
    let (_, h, r_tail) = constraint.right.split();

    let x_argument_count = l_tail.len();
    let h_argument_count = r_tail.len();

    let function_constructed = match x {
        Term::Meta(_) => construct_imitation_function(x_argument_count, h_argument_count, h.clone()),
        _ => construct_imitation_function(h_argument_count, x_argument_count, x.clone())
    };

    match x {
        Term::Meta(_) => Substitution { name: x.get_name(), with: function_constructed },
        _ => Substitution { name: h.get_name(), with: function_constructed }
    }

}

fn construct_imitation_function(x_argument_count: usize, h_argument_count: usize, h: Term) -> Term {
    let mut fresh_vars = vec![];
    let mut builder = h.clone();

    for _ in 0..x_argument_count {
        fresh_vars.push(generate_fresh_var())
    }

    for _ in 0..h_argument_count {
        let mut meta_builder = Term::Meta(generate_fresh_var());
        for fresh_var in &fresh_vars {
            meta_builder = Term::App(Box::new(meta_builder), Box::new(Term::Var(fresh_var.clone())))
        }
        builder = Term::App(Box::new(builder), Box::new(meta_builder))
    }

    for fresh_var in fresh_vars.iter().rev() {
        //TODO change Type::Star?
        builder = Term::Abs(fresh_var.clone(), Type::Star, Box::new(builder));
    }

    builder
}

pub fn projection(_context: &Context, constraint: &Constraint) -> Vec<Substitution> {
    let (_, x, l_head) = constraint.left.split();
    let (_, h, r_head) = constraint.right.split();
    let x_argument_count = l_head.len();
    let h_argument_count = r_head.len();

    let mut substitutions = vec![];

    for index in 0..x_argument_count {
        let substitution = match x {
            Term::Meta(_) => {
                let projected_function = construct_projection_function(index, x_argument_count);
                Substitution { name: x.get_name(), with: projected_function }
            }
            _ => {
                let projected_function = construct_projection_function(index, h_argument_count);
                Substitution { name: h.get_name(), with: projected_function }
            }
        };
        substitutions.push(substitution);
    }

    substitutions
}

fn construct_projection_function(index: usize, length: usize) -> Term {
    let mut fresh_vars = vec![];

    for _ in 0.. length{
        fresh_vars.push(generate_fresh_var())
    }

    let mut builder = Term::Var(fresh_vars[index].clone());

    for fresh_var in fresh_vars.iter().rev() {
        //TODO change Type::Star?
        builder = Term::Abs(fresh_var.clone(), Type::Star, Box::new(builder))
    }

    builder
}
