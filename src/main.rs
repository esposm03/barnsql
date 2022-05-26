#![feature(box_patterns)]

mod parser;
mod query;
mod storage;
mod util;
use chumsky::Parser;
use parser::{eval, parser};

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let src = "--42 * 2 - 20";

    match parser().parse(src) {
        Ok(expr) => match eval(&expr) {
            Ok(val) => println!("Output: {val}"),
            Err(e) => println!("Eval err: {e:?}"),
        },
        Err(e) => e.iter().for_each(|e| println!("Parse err: {e:?}")),
    };

    Ok(())
}
