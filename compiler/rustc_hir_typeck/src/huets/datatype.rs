use std::cell::RefCell;
use std::fmt::{Debug};
use std::rc::Rc;
use std::str::FromStr;
use std::string::ToString;
use std::sync::atomic::{AtomicU32, Ordering};
use rustc_data_structures::fx::FxHashMap;
use crate::huets::datatype::Term::{Abs, App, Var};
use crate::huets::datatype::Type::Star;
use crate::huets::substs::{beta_reduce, term_substitution};

const PLACEHOLDER: &'static str = "placeholder";

#[derive(Clone, PartialEq, Debug)]
pub enum Term {
    Meta(String),
    Var(String),
    Abs(String, Type, Box<Term>),
    App(Box<Term>, Box<Term>)
}

#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Star,
    Arrow(Box<Type>, Box<Type>)
}

#[derive(Clone, PartialEq, Debug)]
pub struct Context {
    pub typing_context: FxHashMap<String, Type>,
    pub substitutions: Vec<Substitution>,
    pub solutions: Rc<RefCell<Vec<Solution>>>,
    pub name_map: FxHashMap<String, Vec<String>>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Constraint {
    pub left: Term,
    pub right: Term
}

#[derive(Clone, PartialEq, Debug)]
pub struct Substitution {
    pub name: String,
    pub with: Term
}

#[derive(Clone, PartialEq, Debug)]
pub struct Problem(pub Vec<Constraint>);

#[derive(Clone, PartialEq, Debug)]
pub struct Solution(pub Vec<Substitution>);

#[derive(Clone, PartialEq, Debug)]
pub struct SolutionSet(pub Vec<Solution>);


impl Term {
    pub fn is_rigid(&self) -> bool {
        !matches!(self, Term::Meta(_))
    }

    pub fn split(&self) -> (Term, Term, Vec<Term>) {
        let mut current = self.clone();
        let mut external_abstractors_builder = Term::Var(PLACEHOLDER.to_string());
        let mut reverse_abstractors_builder = Term::Var(PLACEHOLDER.to_string());
        let mut arguments_builder = vec![];

        // Extract external abstractors
        while let Term::Abs(s, typ, term) = current {
            current = *term;
            external_abstractors_builder = Term::Abs(s, typ, Box::new(external_abstractors_builder));
        }

        while let Term::Abs(s, typ, term) = external_abstractors_builder {
            external_abstractors_builder = *term;
            reverse_abstractors_builder = Term::Abs(s, typ, Box::new(reverse_abstractors_builder));
        }


        // Extract arguments
        while let Term::App(t1, t2) = current {
            arguments_builder.push(*t2);
            current = *t1;
        }
        arguments_builder.reverse();

        // Extract head
        match current {
            Term::Meta(_) | Term::Var(_) => (reverse_abstractors_builder, current, arguments_builder),
            Term::Abs(_, _, _) => panic!("The term is not in eta-normal form"),
            Term::App(_, _) => unreachable!()
        }
    }

    pub fn combine(&self, bindings: Term) -> Term {
        let substs = Substitution { name: PLACEHOLDER.to_string(), with: self.clone() };
        term_substitution(bindings, substs)
    }

    pub fn get_name(&self) -> String {
        match self {
            Term::Meta(s) | Term::Var(s) => s.clone(),
            _ => panic!("Can not get name for Term::Abs or Term::App")
        }
    }

    pub fn equal_in_context(&self, other: &Term, context: &FxHashMap<String, Type>) -> bool {
        match (self, other) {
            (Term::Var(s1), Term::Var(s2)) if s1 == s2 => {
                context.get(s1).is_some()
            }
            (_, _) => false
        }
    }

    pub fn binding_index(&self, bindings: &Term) -> Option<usize> {
        let mut current = bindings;
        let mut depth = 0;
        let mut last_seen_index = None;
        while let Term::Abs(s, _, term) = current {
            current = term;
            if s == &self.get_name() {
                last_seen_index = Some(depth);
            }
            depth += 1;
        }
        last_seen_index.map(|index| depth - index)
    }
}

impl Constraint {
    pub fn is_rigid_rigid(&self) -> bool {
        let (_, l_head, _) = self.left.split();
        let (_, r_head, _) = self.right.split();

        l_head.is_rigid() && r_head.is_rigid()
    }
}

pub fn generate_fresh_var() -> String {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    format!("{:?}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

impl Solution {
    pub fn minimize(self, name_map: &FxHashMap<String, Vec<String>>) -> Solution {
        let mut originals =
            self.0.iter()
                .filter(|substitution| u32::from_str(&substitution.name).is_err())
                .cloned()
                .collect::<Vec<_>>();

        for sub in self.0 {
            for original in &mut originals {
                original.with = term_substitution(original.with.clone(), sub.clone())
            }
        }

        for original in &mut originals {
            if let Some(list) = name_map.get(&original.name) {
                let mut builder = original.with.clone();
                for element in list {
                    builder = beta_reduce(App(Box::new(builder), Box::new(Var(element.to_string()))))
                }

                for element in list.iter().rev() {
                    builder = Abs(element.to_string(), Star, Box::new(builder))
                }

                original.with = builder;
            }
        }

        Solution(originals)
    }
}

impl Context {
    pub fn minimal_solutions(&self) -> SolutionSet {
        SolutionSet(self.solutions.borrow().iter().cloned().map(|solution| solution.minimize(&self.name_map)).collect())
    }
}
