use crate::slaves::serializer::serialize_all;

use std::{fs::File, io::Write};

use super::{
    fetchers::FoundItem,
    serializer::SerType::{self, *},
};
use anyhow::{anyhow, Result};

pub enum SaverType {
    Stdout,
    File(String),
    Multiple(Vec<SaverType>),
    Telegram,
    Postgres,
}

use SaverType::*;

pub struct Saver {
    stype: SaverType,
    sertype: SerType,
}

impl Saver {
    fn new(stype: SaverType, sertype: SerType) -> Self {
        Saver { stype, sertype }
    }

    fn new_default() -> Self {
        Self::new(Stdout, Json)
    }

    fn push(&self, data: Vec<Vec<FoundItem>>) -> Result<()> {
        let ser_data = serialize_all(data, self.sertype);
        match &self.stype {
            Stdout => println!("{}", ser_data),
            File(path) => {
                let mut file = File::open("foo.txt")?;
                write!(file, "{}", ser_data)?
            }
            Multiple(_) => unimplemented!(),
            Telegram => unimplemented!(),
            Postgres => unimplemented!(),
        }
        Ok(())
    }
}
