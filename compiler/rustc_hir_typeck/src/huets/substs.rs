use crate::huets::datatype::{Constraint, Problem, Substitution, Term};

pub fn term_substitution(term: Term, sub: Substitution) -> Term {
    match term {
        Term::Meta(s) | Term::Var(s) if s == sub.name => sub.with,
        Term::Abs(s, typ, t1) =>
            Term::Abs(s, typ, Box::new(term_substitution(*t1, sub))),
        Term::App(t1, t2) => {
            let app_term = Term::App(
                Box::new(term_substitution(*t1, sub.clone())),
                Box::new(term_substitution(*t2, sub))
            );
            beta_reduce(app_term)
        }
        _ => term
    }
}

pub fn beta_reduce(term: Term) -> Term {
    match term {
        Term::App(t1, t2) => {
            match *t1 {
                Term::Abs(s, _, t11) =>
                    term_substitution(*t11, Substitution { name: s, with: *t2 }),
                _ => Term::App(t1, t2)
            }
        }
        _ => term
    }
}


pub fn constraint_substitution(constraint: Constraint, sub: Substitution) -> Constraint {
    Constraint {
        left: term_substitution(constraint.left, sub.clone()),
        right: term_substitution(constraint.right, sub)
    }
}

pub fn problem_substitution(problem: Problem, sub: Substitution) -> Problem {
    Problem(problem.0.into_iter()
        .map(|constraint| constraint_substitution(constraint, sub.clone()))
        .collect())
}
