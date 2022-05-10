use sled::IVec;

use crate::storage::{Table, Val, WhereExpr};

pub trait DbScanner {
    fn next(&mut self) -> Option<Vec<Val>>;
    fn table(&self) -> &Table;

    fn filter_by(&mut self, expr: WhereExpr) -> QueryFilter<Self>
    where
        Self: Sized,
    {
        QueryFilter(self, expr)
    }

    fn map_col(&mut self, to_modify: IVec, expr: WhereExpr) -> QueryMap<Self>
    where
        Self: Sized,
    {
        QueryMap(self, to_modify, expr)
    }
}
impl<'a, S: DbScanner> DbScanner for &mut S {
    fn next(&mut self) -> Option<Vec<Val>> {
        (**self).next()
    }

    fn table(&self) -> &Table {
        (**self).table()
    }
}

pub struct Scan<'a>(pub &'a Table, pub sled::Iter);
impl<'a> DbScanner for Scan<'a> {
    fn next(&mut self) -> Option<Vec<Val>> {
        let (_key, row) = self.1.next()?.ok()?;
        let mut row = row.as_ref();

        let mut res = vec![];
        while let Ok(val) = Val::deserialize(&mut row) {
            res.push(val);
        }
        Some(res)
    }

    fn table(&self) -> &Table {
        self.0
    }
}

pub struct QueryFilter<'a, S: DbScanner>(&'a mut S, WhereExpr);
impl<'a, S: DbScanner> DbScanner for QueryFilter<'a, S> {
    fn next(&mut self) -> Option<Vec<Val>> {
        let next = self.0.next()?;
        if self.1.eval(&next, self.table()).to_bool() {
            Some(next)
        } else {
            None
        }
    }

    fn table(&self) -> &Table {
        self.0.table()
    }
}

pub struct QueryMap<'a, S: DbScanner>(&'a mut S, IVec, WhereExpr);
impl<'a, S: DbScanner> DbScanner for QueryMap<'a, S> {
    fn next(&mut self) -> Option<Vec<Val>> {
        let mut next = self.0.next()?;
        let pos = self
            .table()
            .col_list
            .iter()
            .position(|(col_name, _)| *col_name == self.1)
            .unwrap();

        next[pos] = self.2.eval(&next, self.table());

        Some(next)
    }

    fn table(&self) -> &Table {
        self.0.table()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        query::{DbScanner, Scan},
        storage::{Table, Typ, Val, WhereExpr},
    };

    #[test]
    fn map_filter() {
        let mut db = sled::Config::new().temporary(true).open().unwrap();

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
        )
        .unwrap();

        table
            .insert(vec![
                Val::String("Antonio".into()),
                Val::String("Giunta".into()),
                Val::Number(60),
            ])
            .unwrap();

        assert_eq!(
            vec![
                Val::String("Antonio".into()),
                Val::String("Giunta".into()),
                Val::Number(120)
            ],
            Scan(&table, table.tree.iter())
                .filter_by(WhereExpr::Equal(
                    Box::new(WhereExpr::Column("eta".into())),
                    Box::new(WhereExpr::Literal(Val::Number(60))),
                ))
                .map_col(
                    "eta".into(),
                    WhereExpr::Mul(
                        Box::new(WhereExpr::Column("eta".into())),
                        Box::new(WhereExpr::Literal(Val::Number(2)))
                    )
                )
                .next()
                .unwrap()
        );
    }
}
