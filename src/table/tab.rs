use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io::{self, Write},
};

use sled::{Db, IVec, Tree};

use crate::util::{display_ivec, serialize_ivec};

use super::{ColumnList, ColumnName, Val};

pub struct Table {
    name: IVec,
    col_list: ColumnList,
    primary_key: Vec<ColumnName>,

    tree: Tree,
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
        let mut serialized_row = vec![];
        let mut pk_hasher = DefaultHasher::new();

        for pk_col in &self.primary_key {
            let col_index = self
                .col_list
                .iter()
                .position(|(col_name, _)| col_name == pk_col)
                .unwrap();
            row[col_index].hash(&mut pk_hasher);
        }

        if self.tree.contains_key(pk_hasher.finish().to_le_bytes())? {
            panic!("Row already inserted");
        }

        for val in &row {
            val.serialize(&mut serialized_row)?;
        }

        self.tree
            .insert(pk_hasher.finish().to_le_bytes(), serialized_row)?;

        Ok(())
    }

    pub fn select(&self, primary_key: Vec<Val>) -> Result<Vec<Val>, sled::Error> {
        let mut pk_hasher = DefaultHasher::new();
        for pk_col in primary_key {
            pk_col.hash(&mut pk_hasher);
        }
        let pk = pk_hasher.finish();

        let row = self.tree.get(pk.to_le_bytes())?.expect("No such row");
        let mut reader = row.as_ref();
        let mut deserialized = vec![];

        while let Ok(val) = Val::deserialize(&mut reader) {
            deserialized.push(val);
        }

        Ok(deserialized)
    }
}
