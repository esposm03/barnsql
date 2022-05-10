#![feature(box_patterns)]

mod query;
mod storage;
mod util;
use storage::{Table, Typ, Val};

use std::error::Error;

use crate::storage::WhereExpr;

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = sled::Config::new().temporary(true).open()?;

    let columns = vec![
        ("nome".into(), Typ::String),
        ("cognome".into(), Typ::String),
        ("eta".into(), Typ::Number),
    ];
    let table = Table::new(
        &mut db,
        "sus".into(),
        columns,
        vec!["nome".into(), "cognome".into()],
    )?;

    table.insert(vec![
        Val::String("Antonio".into()),
        Val::String("Giunta".into()),
        Val::Number(60),
    ])?;
    println!("Insert");

    let res = table.select(WhereExpr::And(
        Box::new(WhereExpr::Equal(
            Box::new(WhereExpr::Column("nome".into())),
            Box::new(WhereExpr::Literal(Val::String("Antonio".into()))),
        )),
        Box::new(WhereExpr::Equal(
            Box::new(WhereExpr::Column("cognome".into())),
            Box::new(WhereExpr::Literal(Val::String("Giunta".into()))),
        )),
    ))?;

    println!("Res: {res:?}");

    table.insert(vec![
        Val::String("Antonio".into()),
        Val::String("Giunta".into()),
        Val::Number(60),
    ])?;
    println!("Insert");

    Ok(())
}
