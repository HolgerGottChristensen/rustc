use std::fmt::{Debug, Display, Formatter};
use paris::formatter::colorize_string;
use crate::huets::datatype::{Constraint, Problem, Solution, SolutionSet, Substitution, Term, Type};

impl Term {
    pub fn print(&self) -> String {
        match self {
            Term::Abs(s, t1, t2) => colorize_string(format!("<bright-green><b>λ</>{}<bright-green>:</>{} {}", s, t1.print(), t2.print())),
            _ => self.print_middle()
        }
    }

    fn print_middle(&self) -> String {
        match self {
            Term::App(t1, t2) => format!("{} {}", t1.print_middle(), t2.print_atomic()),
            _ => self.print_atomic()
        }
    }

    fn print_atomic(&self) -> String {
        match self {
            Term::Meta(s) => colorize_string(format!("<red><i>{}</>", s)),
            Term::Var(s) => s.to_string(),
            _ => format!("({})", self.print())
        }
    }
}

impl Type {
    pub fn print(&self) -> String {
        match self {
            Type::Arrow(t1, t2) => {
                colorize_string(format!("<bright-blue><b>{} -> {}</>", t1.print_atomic(), t2.print()))
            }
            _ => self.print_atomic()
        }
    }

    fn print_atomic(&self) -> String {
        match self {
            Type::Star => colorize_string(String::from("<bright-blue><b>*</>")),
            _ => format!("({})", self.print())
        }
    }
}

impl Constraint {
    pub fn print(&self) -> String {
        colorize_string(format!("{} <yellow><b>=?</> {}", self.left.print(), self.right.print()))
    }
}

impl Problem {
    pub fn print(&self) -> String {
        colorize_string(self.0.iter().map(|constraint| constraint.print()).collect::<Vec<_>>().join(" <bright-yellow><b>∧</> "))
    }
}

impl Substitution {
    pub fn print(&self) -> String {
        colorize_string(format!("<red><i>{}</> <yellow>=></> {}", self.name, self.with.print()))
    }
}




impl Display for Term {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.print())
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.print())
    }
}

impl Display for Constraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.print())
    }
}

impl Display for Problem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.print())
    }
}

impl Display for Substitution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.print())
    }
}

struct PrintHelper(String);

impl Debug for PrintHelper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|a| PrintHelper(format!("{}", a))))
            .finish()
    }
}

impl Display for SolutionSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SolutionSet [")?;
        for e in &self.0 {
            writeln!(f, "\t{},", e)?;
        }
        writeln!(f, "]")
    }
}
