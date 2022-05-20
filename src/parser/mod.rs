use chumsky::prelude::*;

fn parser() -> impl Parser<char, WhereExpr, Error = Simple<char>> {

}

#[test]
#[cfg(test)]
fn test() {
    parser()
}