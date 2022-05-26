use chumsky::prelude::*;

pub fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    sum().then_ignore(end())
}

fn int() -> impl Parser<char, Expr, Error = Simple<char>> {
    text::int(10)
        .map(|s: String| Expr::Num(s.parse().unwrap()))
        .padded()
}

fn op(op: char) -> impl Parser<char, char, Error = Simple<char>> {
    just(op).padded()
}

fn unary() -> impl Parser<char, Expr, Error = Simple<char>> {
    op('-')
        .repeated()
        .then(int())
        .foldr(|_op: char, int: Expr| Expr::Neg(Box::new(int)))
}

fn prod() -> impl Parser<char, Expr, Error = Simple<char>> {
    let mul = op('*').to(Expr::Mul as fn(_, _) -> _);
    let div = op('/').to(Expr::Div as fn(_, _) -> _);

    unary()
        .then(mul.or(div).then(unary()).repeated())
        .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)))
}

fn sum() -> impl Parser<char, Expr, Error = Simple<char>> {
    let add = op('+').to(Expr::Add as fn(_, _) -> _);
    let sub = op('-').to(Expr::Sub as fn(_, _) -> _);

    prod()
        .then(add.or(sub).then(unary()).repeated())
        .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)))
}

pub enum Expr {
    Num(f64),
    Neg(Box<Expr>),

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

pub fn eval(expr: &Expr) -> Result<f64, String> {
    use Expr::*;
    match expr {
        Num(n) => Ok(*n),
        Neg(e) => Ok(-eval(e)?),

        Add(a, b) => Ok(eval(a)? + eval(b)?),
        Sub(a, b) => Ok(eval(a)? - eval(b)?),
        Mul(a, b) => Ok(eval(a)? * eval(b)?),
        Div(a, b) => Ok(eval(a)? / eval(b)?),
    }
}
