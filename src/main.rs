#![feature(iter_intersperse)]

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
    fn create_table(&self, name: &str, columns: &[&str]) -> Result<(), Box<dyn Error>> {
        let tables = self.open_tree("tables")?;
        tables.insert(name, columns.join(",").as_str())?;

        self.open_tree(name)?;
        self.flush()?;

        Ok(())
    }

    fn create_index(&self, table_name: &str, columns: &[&str]) -> Result<(), Box<dyn Error>> {
        let col_list = columns.join(",");
        let indexes = self.open_tree("indexes")?;
        indexes.insert(format!("{table_name}_{col_list}"), &[42])?;

        self.open_tree(format!("index_{table_name}_{col_list}"))?;
        self.flush()?;

        Ok(())
    }

    fn insert_into(
        &self,
        table_name: &str,
        row: &HashMap<IVec, IVec>,
    ) -> Result<(), Box<dyn Error>> {
        let table = self.open_tree(table_name)?;
        let indexes = self.open_tree("indexes")?;
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

        for index in indexes.scan_prefix(table_name) {
            let (orig_index, _) = index?;
            let index = orig_index.subslice(
                table_name.len() + 1,
                orig_index.len() - (table_name.len() + 1),
            );
            let index = index.as_ref();
            println!("index: {}", String::from_utf8(index.to_vec()).unwrap());

            #[rustfmt::skip]
            let index_key: IVec = index
                .split(|ch| *ch == b',')
                .map(|name| &row[name])
                .intersperse(&IVec::from(&[b',']))
                .map(|i| i.into_iter())
                .flatten()
                .copied()
                .collect();

            let index_table = self.open_tree(format!(
                "index_{}",
                String::from_utf8(orig_index.to_vec()).unwrap(),
            ))?;

            let res = index_table.insert(index_key, key)?;
            assert!(res.is_none(), "Index mapping isn't unique");
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
        } else if let Some(index_name) = self.matches_index(table_name, where_clause)? {
            let index = self.open_tree(format!("index_{table_name}_{index_name}"))?;

            let index_key: IVec = index_name
                .split(|ch: char| ch == ',')
                .map(|name| &where_clause[name.as_bytes()])
                .intersperse(&IVec::from(&[b',']))
                .map(|i| i.into_iter())
                .flatten()
                .copied()
                .collect();

            index.get(index_key)?.unwrap()
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

    fn matches_index(
        &self,
        table_name: &str,
        row: &HashMap<IVec, IVec>,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let indexes = self.open_tree("indexes")?;

        for index in indexes.scan_prefix(table_name) {
            let (orig_index, _) = index?;
            let index = orig_index.subslice(
                table_name.len() + 1,
                orig_index.len() - (table_name.len() + 1),
            );
            let index = index.as_ref();
            println!("index: {}", String::from_utf8(index.to_vec()).unwrap());

            if index
                .split(|ch| *ch == b',')
                .all(|col_name| row.contains_key(col_name))
            {
                return Ok(Some(String::from_utf8(index.to_vec()).unwrap()));
            }
        }

        Ok(None)
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

        db.create_index("docenti", &["cognome"]).unwrap();

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
    fn select_index() {
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

    #[test]
    fn select_no_key() {
        let (db, docente) = common();

        let res = db
            .select_from(
                "docenti",
                &[("sesso".into(), "M".into())].iter().cloned().collect(),
            )
            .expect("Failed to select");

        assert_eq!(docente, res);
    }
}
