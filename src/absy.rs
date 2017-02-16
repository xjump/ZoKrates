//
// @file absy.rs
// @author Dennis Kuhnert <dennis.kuhnert@campus.tu-berlin.de>
// @date 2017

use std::fmt;
use std::collections::HashMap;

pub struct Prog {
    pub id: String,
    pub arguments: Vec<Parameter>,
    pub statements: Vec<Statement>,
}
impl Prog {
    pub fn get_witness(&self, inputs: Vec<i32>) -> HashMap<String, i32> {
        assert!(self.arguments.len() == inputs.len());
        let mut witness = HashMap::new();
        witness.insert("~one".to_string(), 1);
        for i in 0..self.arguments.len() {
            witness.insert(self.arguments[i].id.to_string(), inputs[i]);
        }
        for statement in &self.statements {
            match *statement {
                Statement::Return(ref expr) => {
                    let s = expr.solve(&mut witness);
                    witness.insert("~out".to_string(), s);
                },
                Statement::Definition(ref id, ref expr) => {
                    let s = expr.solve(&mut witness);
                    witness.insert(id.to_string(), s);
                },
                Statement::Condition(..) => unimplemented!(),
            }
        }
        witness
    }
}
impl fmt::Display for Prog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "def {}({}):\n{}", self.id, self.arguments.iter().map(|x| format!("{}", x)).collect::<Vec<_>>().join(","), self.statements.iter().map(|x| format!("\t{}", x)).collect::<Vec<_>>().join("\n"))
    }
}
impl fmt::Debug for Prog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Prog(id: {:?}, arguments: {:?}, ...):\n{}", self.id, self.arguments, self.statements.iter().map(|x| format!("\t{:?}", x)).collect::<Vec<_>>().join("\n"))
    }
}

pub enum Statement {
    Return(Expression),
    Definition(String, Expression),
    Condition(Expression, Expression),
}
impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Statement::Return(ref expr) => write!(f, "return {}", expr),
            Statement::Definition(ref lhs, ref rhs) => write!(f, "{} = {}", lhs, rhs),
            Statement::Condition(ref lhs, ref rhs) => write!(f, "{} == {}", lhs, rhs),
        }
    }
}
impl fmt::Debug for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Statement::Return(ref expr) => write!(f, "Return({:?})", expr),
            Statement::Definition(ref lhs, ref rhs) => write!(f, "Definition({:?}, {:?})", lhs, rhs),
            Statement::Condition(ref lhs, ref rhs) => write!(f, "Condition({:?}, {:?})", lhs, rhs),
        }
    }
}

pub struct Parameter { pub id: String }
impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}
impl fmt::Debug for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parameter(id: {:?})", self.id)
    }
}

#[derive(Clone,PartialEq)]
pub enum Expression {
    NumberLiteral(i32),
    VariableReference(String),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mult(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Pow(Box<Expression>, Box<Expression>),
    IfElse(Box<Condition>, Box<Expression>, Box<Expression>),
}
impl Expression {
    pub fn apply_substitution(&self, substitution: &HashMap<String, String>) -> Expression {
        match *self {
            ref e @ Expression::NumberLiteral(_) => e.clone(),
            Expression::VariableReference(ref v) => {
                let mut new_name = v.to_string();
                loop {
                    match substitution.get(&new_name) {
                        Some(x) => new_name = x.to_string(),
                        None => return Expression::VariableReference(new_name),
                    }
                }
            },
            Expression::Add(ref e1, ref e2) => Expression::Add(box e1.apply_substitution(substitution), box e2.apply_substitution(substitution)),
            Expression::Sub(ref e1, ref e2) => Expression::Sub(box e1.apply_substitution(substitution), box e2.apply_substitution(substitution)),
            Expression::Mult(ref e1, ref e2) => Expression::Mult(box e1.apply_substitution(substitution), box e2.apply_substitution(substitution)),
            Expression::Div(ref e1, ref e2) => Expression::Div(box e1.apply_substitution(substitution), box e2.apply_substitution(substitution)),
            Expression::Pow(ref e1, ref e2) => Expression::Pow(box e1.apply_substitution(substitution), box e2.apply_substitution(substitution)),
            Expression::IfElse(ref c, ref e1, ref e2) => Expression::IfElse(box c.apply_substitution(substitution), box e1.apply_substitution(substitution), box e2.apply_substitution(substitution)),
        }
    }

    fn solve(&self, inputs: &mut HashMap<String, i32>) -> i32 {
        match *self {
            Expression::NumberLiteral(x) => x,
            Expression::VariableReference(ref var) => {
                if let None = inputs.get(var) {
                    if var.contains("_b") {
                        let var_name = var.split("_b").collect::<Vec<_>>()[0];
                        let mut num = inputs[var_name];
                        let bits = 8;
                        if num < 0 {
                            num += 2i32.pow(bits - 1);
                            inputs.insert(format!("{}_b{}", &var_name, bits - 1), 1);
                        } else {
                            inputs.insert(format!("{}_b{}", &var_name, bits - 1), 0);
                        }
                        for i in (0..bits - 1).rev() {
                            if 2i32.pow(i) <= num {
                                num -= 2i32.pow(i);
                                inputs.insert(format!("{}_b{}", &var_name, i), 1);
                            } else {
                                inputs.insert(format!("{}_b{}", &var_name, i), 0);
                            }
                        }
                        assert_eq!(num, 0);
                    } else {
                        panic!("Variable not found in inputs: {}", var);
                    }
                }
                inputs[var]
            },
            Expression::Add(ref x, ref y) => x.solve(inputs) + y.solve(inputs),
            Expression::Sub(ref x, ref y) => x.solve(inputs) - y.solve(inputs),
            Expression::Mult(ref x, ref y) => x.solve(inputs) * y.solve(inputs),
            Expression::Div(ref x, ref y) => x.solve(inputs) / y.solve(inputs),
            Expression::Pow(ref x, ref y) => x.solve(inputs).pow(y.solve(inputs) as u32),
            Expression::IfElse(ref condition, ref consequent, ref alternative)
                => if condition.solve(inputs) { consequent.solve(inputs) } else { alternative.solve(inputs) },
        }
    }

    pub fn is_linear(&self) -> bool {
        match *self {
            Expression::NumberLiteral(_) |
            Expression::VariableReference(_) => true,
            Expression::Add(ref x, ref y) |
            Expression::Sub(ref x, ref y) => x.is_linear() && y.is_linear(),
            Expression::Mult(ref x, ref y) |
            Expression::Div(ref x, ref y) => match (x.clone(), y.clone()) {
                (box Expression::NumberLiteral(_), box Expression::NumberLiteral(_)) |
                (box Expression::NumberLiteral(_), box Expression::VariableReference(_)) |
                (box Expression::VariableReference(_), box Expression::NumberLiteral(_)) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_flattened(&self) -> bool {
        match *self {
            Expression::NumberLiteral(_) |
            Expression::VariableReference(_) => true,
            Expression::Add(ref x, ref y) |
            Expression::Sub(ref x, ref y) => x.is_linear() && y.is_linear(),
            Expression::Mult(ref x, ref y) |
            Expression::Div(ref x, ref y) => x.is_linear() && y.is_linear(),
            _ => false,
        }
    }
}
impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expression::NumberLiteral(ref i) => write!(f, "{}", i),
            Expression::VariableReference(ref var) => write!(f, "{}", var),
            Expression::Add(ref lhs, ref rhs) => write!(f, "({} + {})", lhs, rhs),
            Expression::Sub(ref lhs, ref rhs) => write!(f, "({} - {})", lhs, rhs),
            Expression::Mult(ref lhs, ref rhs) => write!(f, "({} * {})", lhs, rhs),
            Expression::Div(ref lhs, ref rhs) => write!(f, "({} / {})", lhs, rhs),
            Expression::Pow(ref lhs, ref rhs) => write!(f, "{}**{}", lhs, rhs),
            Expression::IfElse(ref condition, ref consequent, ref alternative) => write!(f, "if {} then {} else {} fi", condition, consequent, alternative),
        }
    }
}
impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expression::NumberLiteral(ref i) => write!(f, "Num({})", i),
            Expression::VariableReference(ref var) => write!(f, "Ide({})", var),
            Expression::Add(ref lhs, ref rhs) => write!(f, "Add({:?}, {:?})", lhs, rhs),
            Expression::Sub(ref lhs, ref rhs) => write!(f, "Sub({:?}, {:?})", lhs, rhs),
            Expression::Mult(ref lhs, ref rhs) => write!(f, "Mult({:?}, {:?})", lhs, rhs),
            Expression::Div(ref lhs, ref rhs) => write!(f, "Div({:?}, {:?})", lhs, rhs),
            Expression::Pow(ref lhs, ref rhs) => write!(f, "Pow({:?}, {:?})", lhs, rhs),
            Expression::IfElse(ref condition, ref consequent, ref alternative) => write!(f, "IfElse({:?}, {:?}, {:?})", condition, consequent, alternative),
        }
    }
}

#[derive(Clone,PartialEq)]
pub enum Condition {
    Lt(Expression, Expression),
    Le(Expression, Expression),
    Eq(Expression, Expression),
    Ge(Expression, Expression),
    Gt(Expression, Expression),
}
impl Condition {
    fn apply_substitution(&self, substitution: &HashMap<String, String>) -> Condition {
        match *self {
            Condition::Lt(ref lhs, ref rhs) => Condition::Lt(lhs.apply_substitution(substitution), rhs.apply_substitution(substitution)),
            Condition::Le(ref lhs, ref rhs) => Condition::Le(lhs.apply_substitution(substitution), rhs.apply_substitution(substitution)),
            Condition::Eq(ref lhs, ref rhs) => Condition::Eq(lhs.apply_substitution(substitution), rhs.apply_substitution(substitution)),
            Condition::Ge(ref lhs, ref rhs) => Condition::Ge(lhs.apply_substitution(substitution), rhs.apply_substitution(substitution)),
            Condition::Gt(ref lhs, ref rhs) => Condition::Gt(lhs.apply_substitution(substitution), rhs.apply_substitution(substitution)),
        }
    }

    fn solve(&self, inputs: &mut HashMap<String, i32>) -> bool {
        match *self {
            Condition::Lt(ref lhs, ref rhs) => lhs.solve(inputs) < rhs.solve(inputs),
            Condition::Le(ref lhs, ref rhs) => lhs.solve(inputs) <= rhs.solve(inputs),
            Condition::Eq(ref lhs, ref rhs) => lhs.solve(inputs) == rhs.solve(inputs),
            Condition::Ge(ref lhs, ref rhs) => lhs.solve(inputs) >= rhs.solve(inputs),
            Condition::Gt(ref lhs, ref rhs) => lhs.solve(inputs) > rhs.solve(inputs),
        }
    }
}
impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Condition::Lt(ref lhs, ref rhs) => write!(f, "{} < {}", lhs, rhs),
            Condition::Le(ref lhs, ref rhs) => write!(f, "{} <= {}", lhs, rhs),
            Condition::Eq(ref lhs, ref rhs) => write!(f, "{} == {}", lhs, rhs),
            Condition::Ge(ref lhs, ref rhs) => write!(f, "{} >= {}", lhs, rhs),
            Condition::Gt(ref lhs, ref rhs) => write!(f, "{} > {}", lhs, rhs),
        }
    }
}
impl fmt::Debug for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
