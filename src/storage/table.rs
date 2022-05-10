use std::{
    collections::HashMap,
    io::{self, Write},
};

use sled::{Db, IVec, Tree};

use crate::{
    storage::{ColumnList, ColumnName, Val},
    util::{display_ivec, serialize_ivec},
};

pub struct Table {
    pub name: IVec,
    pub col_list: ColumnList,
    pub primary_key: Vec<ColumnName>,

    pub tree: Tree,
}

impl Table {
    pub fn new(
        db: &mut Db,
        name: IVec,
        columns: ColumnList,
        primary_key: Vec<ColumnName>,
    ) -> Result<Self, sled::Error> {
        if db.tree_names().contains(&name) {
            panic!("Table {} already exists", display_ivec(&name));
        }

        let tree = db.open_tree(&name)?;
        let table = Table {
            name,
            tree,
            primary_key,
            col_list: columns,
        };

        let mut serialized_table = vec![];
        table.serialize(&mut serialized_table)?;
        db.open_tree("tables")?;
        db.insert(table.name.clone(), serialized_table)?;

        Ok(table)
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        for (name, typ) in &self.col_list {
            writer.write_all(&[typ.serialize()])?;
            serialize_ivec(name, writer)?;
        }

        write!(writer, ";")?;

        for name in &self.primary_key {
            serialize_ivec(name, writer)?;
        }

        Ok(())
    }

    pub fn insert(&self, row: Vec<Val>) -> Result<(), sled::Error> {
        let mut pk = vec![];

        for pk_col in &self.primary_key {
            let col_index = self
                .col_list
                .iter()
                .position(|(col_name, _)| col_name == pk_col)
                .unwrap();
            row[col_index].serialize(&mut pk)?;
        }

        if self.tree.contains_key(&pk)? {
            panic!("Row already inserted");
        }

        let mut serialized_row = vec![];

        for val in &row {
            val.serialize(&mut serialized_row)?;
        }

        self.tree.insert(pk, serialized_row)?;

        Ok(())
    }

    pub fn select(&self, expr: WhereExpr) -> Result<Option<Vec<Val>>, sled::Error> {
        if let Some(serialized_pk) = self.select_by_pk(
            expr.lookup_by_pk(self)
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
        ) {
            let row = self.tree.get(serialized_pk)?.expect("No such row");
            if let Some(row) = self.satisfies_expr(row, &expr) {
                return Ok(Some(row));
            }
        } else {
            for val in self.tree.iter() {
                let (_, row) = val?;
                if let Some(row) = self.satisfies_expr(row, &expr) {
                    return Ok(Some(row));
                }
            }
        };

        Ok(None)
    }

    fn select_by_pk(&self, pk: HashMap<IVec, Val>) -> Option<IVec> {
        let mut serialized_pk = vec![];

        for pk_col_name in &self.primary_key {
            if let Some(val) = pk.get(pk_col_name) {
                val.serialize(&mut serialized_pk)
                    .expect("Serialization failed");
            } else {
                return None;
            }
        }

        Some(serialized_pk.into())
    }

    fn satisfies_expr(&self, row: IVec, expr: &WhereExpr) -> Option<Vec<Val>> {
        let mut reader = row.as_ref();
        let mut deserialized = vec![];

        while let Ok(val) = Val::deserialize(&mut reader) {
            deserialized.push(val);
        }

        if expr.eval(&deserialized, self).to_bool() {
            Some(deserialized)
        } else {
            None
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum WhereExpr {
    Column(IVec),
    Literal(Val),

    // Arithmetic
    Sum(Box<WhereExpr>, Box<WhereExpr>),
    Sub(Box<WhereExpr>, Box<WhereExpr>),
    Mul(Box<WhereExpr>, Box<WhereExpr>),
    Div(Box<WhereExpr>, Box<WhereExpr>),

    // Logical
    And(Box<WhereExpr>, Box<WhereExpr>),
    Or(Box<WhereExpr>, Box<WhereExpr>),
    Not(Box<WhereExpr>),

    // Comparison
    Equal(Box<WhereExpr>, Box<WhereExpr>),
    Gt(Box<WhereExpr>, Box<WhereExpr>),
    Gte(Box<WhereExpr>, Box<WhereExpr>),
    Lt(Box<WhereExpr>, Box<WhereExpr>),
    Lte(Box<WhereExpr>, Box<WhereExpr>),
}

impl WhereExpr {
    pub fn eval(&self, row: &[Val], schema: &Table) -> Val {
        match self {
            Self::Literal(val) => val.clone(),
            Self::Column(name) => {
                let col_index = schema
                    .col_list
                    .iter()
                    .position(|(col_name, _)| col_name == name)
                    .unwrap();

                row[col_index].clone()
            }

            // Arithmethic
            WhereExpr::Sum(a, b) => a.eval(row, schema) + b.eval(row, schema),
            WhereExpr::Sub(a, b) => a.eval(row, schema) - b.eval(row, schema),
            WhereExpr::Mul(a, b) => a.eval(row, schema) * b.eval(row, schema),
            WhereExpr::Div(a, b) => a.eval(row, schema) / b.eval(row, schema),

            // Logical
            WhereExpr::And(a, b) => {
                Val::Boolean(a.eval(row, schema).to_bool() && b.eval(row, schema).to_bool())
            }
            WhereExpr::Or(_, _) => todo!(),
            WhereExpr::Not(_) => todo!(),

            // Comparison
            WhereExpr::Equal(a, b) => Val::Boolean(a.eval(row, schema) == b.eval(row, schema)),
            WhereExpr::Gt(a, b) => Val::Boolean(a.eval(row, schema) > b.eval(row, schema)),
            WhereExpr::Gte(a, b) => Val::Boolean(a.eval(row, schema) >= b.eval(row, schema)),
            WhereExpr::Lt(a, b) => Val::Boolean(a.eval(row, schema) < b.eval(row, schema)),
            WhereExpr::Lte(a, b) => Val::Boolean(a.eval(row, schema) <= b.eval(row, schema)),
        }
    }

    pub fn lookup_by_pk(&self, schema: &Table) -> Vec<(IVec, Val)> {
        let mut exprs = vec![];

        match self {
            WhereExpr::Literal(_) => {}
            WhereExpr::Column(_) => {}

            // Comparison
            WhereExpr::Equal(box WhereExpr::Column(col), box WhereExpr::Literal(lit)) => {
                if schema.primary_key.contains(col) {
                    exprs.push((col.clone(), lit.clone()));
                }
            }

            WhereExpr::And(a, b) => {
                exprs.append(&mut a.lookup_by_pk(schema));
                exprs.append(&mut b.lookup_by_pk(schema));
            }

            _ => {}
        };

        exprs
    }
}
