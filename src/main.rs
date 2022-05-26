#![feature(box_patterns)]

mod parsing;
mod query;
#[allow(dead_code)]
mod storage;
mod util;

use std::{collections::HashMap, error::Error};

use chumsky::Parser;
use parsing::{eval, lexer, parser};

fn main() -> Result<(), Box<dyn Error>> {
    let src = "-hello(42, sus)";
    let mut vars = HashMap::new();
    vars.insert("hello".into(), 42);
    vars.insert("sus".into(), 69);

    match parser().parse(lexer().parse(src).unwrap()) {
        Ok(expr) => match eval(&expr, &mut vars) {
            Ok(val) => println!("Output: {val}"),
            Err(e) => println!("Eval err: {e:?}"),
        },
        Err(e) => e.iter().for_each(|e| println!("Parse err: {e:?}")),
    };

    Ok(())
}
