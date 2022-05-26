#![allow(dead_code)]

use std::collections::HashMap;

use chumsky::prelude::*;

use self::lexer::Token;

pub mod lexer;

pub fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    let expr = recursive(|expr| {
        let int = text::int(10)
            .map(|s: String| Expr::Num(s.parse().unwrap()))
            .padded();

        let call = text::ident()
            .padded()
            .then(
                expr.clone()
                    .separated_by(just(','))
                    .allow_leading()
                    .delimited_by(just('('), just(')')),
            )
            .map(|(f, args)| Expr::Call(f, args));

        let atom = int
            .or(call)
            .or(text::ident().padded().map(Expr::Ident))
            .or(expr.delimited_by(just('('), just(')')));

        let op = |c| just(c).padded();

        let unary = op('-')
            .repeated()
            .then(atom)
            .foldr(|_op, rhs| Expr::Neg(Box::new(rhs)));

        let product = unary
            .clone()
            .then(
                op('*')
                    .to(Expr::Mul as fn(_, _) -> _)
                    .or(op('/').to(Expr::Div as fn(_, _) -> _))
                    .then(unary)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        let sum = product
            .clone()
            .then(
                op('+')
                    .to(Expr::Add as fn(_, _) -> _)
                    .or(op('-').to(Expr::Sub as fn(_, _) -> _))
                    .then(product)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        sum
    });

    expr
}

pub fn parser2() -> impl Parser<Token, Expr, Error = Simple<Token>> {
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
