use chumsky::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Num(i64),
    Ident(String),

    Add,
    Sub,
    Mul,
    Div,

    ParensOpen,
    ParensClose,
    Comma,
}

pub fn lexer() -> impl Parser<char, Vec<Token>, Error = Simple<char>> {
    choice((
        just('+').to(Token::Add),
        just('-').to(Token::Sub),
        just('*').to(Token::Mul),
        just('/').to(Token::Div),
        just('(').to(Token::ParensOpen),
        just(')').to(Token::ParensClose),
        just(',').to(Token::Comma),
        text::int(10).map(|s: String| Token::Num(s.parse().unwrap())),
        text::ident().map(Token::Ident),
    ))
    .padded()
    .repeated()
}
