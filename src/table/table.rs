use std::io::{self, Write};

use sled::{Db, IVec, Tree};

use crate::{
    table::{ColumnList, ColumnName, Val},
    util::{display_ivec, serialize_ivec},
};

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

    pub fn select(&self, primary_key: Vec<Val>) -> Result<Vec<Val>, sled::Error> {
        let mut pk = vec![];
        for pk_col in primary_key {
            pk_col.serialize(&mut pk)?;
        }

        let row = self.tree.get(pk)?.expect("No such row");
        let mut reader = row.as_ref();
        let mut deserialized = vec![];

        while let Ok(val) = Val::deserialize(&mut reader) {
            deserialized.push(val);
        }

        Ok(deserialized)
    }
}
