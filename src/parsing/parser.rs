use super::Token;
use std::collections::HashMap;

use chumsky::prelude::*;

pub fn parser() -> impl Parser<Token, Expr, Error = Simple<Token>> {
    recursive(|expr| {
        let literal = select! {
            Token::Num(n) => Expr::Num(n),
            Token::Ident(i) => Expr::Ident(i),
        };
        let call = select! {Token::Ident(i) => i}
            .then(
                expr.clone()
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::ParensOpen), just(Token::ParensClose)),
            )
            .map(|(expr, args): (String, Vec<Expr>)| Expr::Call(expr, args));
        let expr = expr.delimited_by(just(Token::ParensOpen), just(Token::ParensClose));

        let atom = choice((call, literal, expr));

        let unary = just(Token::Sub)
            .repeated()
            .then(atom.clone())
            .foldr(|_op, rhs| Expr::Neg(Box::new(rhs)));

        atom.or(unary)
    })
    .then_ignore(end())
}

pub enum Expr {
    Num(i64),
    Ident(String),

    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),

    Call(String, Vec<Expr>),
    Let {
        name: String,
        rhs: Box<Expr>,
        then: Box<Expr>,
    },
}

pub fn eval(expr: &Expr, vars: &mut HashMap<String, i64>) -> Result<i64, String> {
    use Expr::*;
    match expr {
        Num(n) => Ok(*n),
        Ident(n) => vars.get(n).copied().ok_or("no such variable".to_string()),
        Neg(e) => Ok(-eval(e, vars)?),

        Add(a, b) => Ok(eval(a, vars)? + eval(b, vars)?),
        Sub(a, b) => Ok(eval(a, vars)? - eval(b, vars)?),
        Mul(a, b) => Ok(eval(a, vars)? * eval(b, vars)?),
        Div(a, b) => Ok(eval(a, vars)? / eval(b, vars)?),

        Call(function, args) => Err(format!(
            "TODO: {function}({:?})",
            args.iter()
                .map(|arg| eval(arg, vars).unwrap())
                .collect::<Vec<_>>()
        )),
        _ => Err("can't call eval_decimal on this expression".to_string()),
    }
}
