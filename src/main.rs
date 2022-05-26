#![feature(box_patterns)]

mod parser;
mod query;
#[allow(dead_code)]
mod storage;
mod util;
use chumsky::Parser;
use parser::{eval, lexer::lexer, parser2};

use std::{collections::HashMap, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let src = "-hello(42, sus)";
    let mut vars = HashMap::new();
    vars.insert("hello".into(), 42);
    vars.insert("sus".into(), 69);

    match parser2().parse(lexer().parse(src).unwrap()) {
        Ok(expr) => match eval(&expr, &mut vars) {
            Ok(val) => println!("Output: {val}"),
            Err(e) => println!("Eval err: {e:?}"),
        },
        Err(e) => e.iter().for_each(|e| println!("Parse err: {e:?}")),
    };

    Ok(())
}
