use std::{
    collections::{HashMap, HashSet},
    error::Error,
    ops::Deref,
};

use sled::{Db, IVec};

fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}

struct Database(Db);

impl Deref for Database {
    type Target = Db;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(dead_code)]
impl Database {
    fn create_table(&self, name: &str, attributes: &[&str]) -> Result<(), Box<dyn Error>> {
        let tables = self.open_tree("tables")?;
        tables.insert(name, attributes.join(",").as_str())?;

        self.open_tree(name)?;

        Ok(())
    }

    fn insert_into(
        &self,
        table_name: &str,
        row: &HashMap<IVec, IVec>,
    ) -> Result<(), Box<dyn Error>> {
        let table = self.open_tree(table_name)?;
        let cols = self.table_columns(table_name)?;

        let key = &row[&cols[0]];

        for col_name in cols {
            table.insert(
                &format!(
                    "{}_{}",
                    String::from_utf8_lossy(key),
                    String::from_utf8_lossy(&col_name)
                ),
                &row[&col_name],
            )?;

            println!(
                "inserito '{}_{}' = '{}'",
                String::from_utf8_lossy(key),
                String::from_utf8_lossy(&col_name),
                String::from_utf8_lossy(&row[&col_name]),
            )
        }

        Ok(())
    }

    fn select_from(
        &self,
        table_name: &str,
        where_clause: &HashMap<IVec, IVec>,
    ) -> Result<HashMap<IVec, IVec>, Box<dyn Error>> {
        let table = self.open_tree(table_name)?;
        let columns = self.table_columns(table_name)?;

        let key_name = &columns[0];

        let key_value = if let Some(key_value) = where_clause.get(key_name) {
            key_value.clone()
        } else {
            let mut result_set = HashSet::<IVec>::new();

            for entry in &table {
                let (k, v) = entry?;

                let mut iter = k.split(|ch| *ch == b'_');
                let key_val = iter.next().unwrap().into();
                let col_name = iter.next().unwrap();

                if let Some(where_val) = where_clause.get(col_name) {
                    if *where_val == v {
                        result_set.insert(key_val);
                    } else {
                        result_set.remove(&key_val);
                    }
                }
            }

            assert_eq!(
                result_set.len(),
                1,
                "more than one row matched `where_clause`"
            );

            result_set.into_iter().next().unwrap()
        };
        // If where_clause has all columns of an index, use that

        let mut res = HashMap::new();
        for col in columns {
            res.insert(
                col.clone(),
                table
                    .get(format!(
                        "{}_{}",
                        String::from_utf8_lossy(&key_value),
                        String::from_utf8_lossy(&col)
                    ))?
                    .unwrap(),
            );
        }
        Ok(res)
    }

    fn table_columns(&self, table_name: &str) -> Result<Vec<IVec>, Box<dyn Error>> {
        let cols = self.open_tree("tables")?.get(table_name)?.unwrap();
        let split = cols.split(|ch| *ch == ',' as u8);
        let mut vecs = vec![];

        for col in split {
            vecs.push(IVec::from(col));
        }

        Ok(vecs)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Database;
    use sled::{Config, IVec};

    fn common() -> (Database, HashMap<IVec, IVec>) {
        let db = Database(
            Config::new()
                .temporary(true)
                .open()
                .expect("Failed to create database"),
        );

        db.create_table("docenti", &["nome", "cognome", "sesso"])
            .unwrap();

        let mut docente = HashMap::new();
        docente.insert("nome".into(), "Antonio".into());
        docente.insert("cognome".into(), "Giunta".into());
        docente.insert("sesso".into(), "M".into());
        db.insert_into("docenti", &docente)
            .expect("Failed to insert");

        (db, docente)
    }

    #[test]
    fn insert_select() {
        let (db, docente) = common();

        let res = db
            .select_from(
                "docenti",
                &[("nome".into(), "Antonio".into())]
                    .iter()
                    .cloned()
                    .collect(),
            )
            .expect("Failed to select");

        assert_eq!(docente, res);
    }

    #[test]
    fn select_no_key() {
        let (db, docente) = common();

        let res = db
            .select_from(
                "docenti",
                &[("cognome".into(), "Giunta".into())]
                    .iter()
                    .cloned()
                    .collect(),
            )
            .expect("Failed to select");

        assert_eq!(docente, res);
    }
}
