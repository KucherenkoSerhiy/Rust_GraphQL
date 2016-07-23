/*
pub trait GraphQLReader {
    fn read(&mut self, value: string);
}

impl<T: Read> GraphQLReader for T {
    fn read(&mut self, value: string) {
        println!("Read the line: {}", value);
    }
}
*/

use def::*;
use parser;

use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::vec::Vec;
use std::str;
use std::io::prelude::*;

use nom::IResult;

pub fn extract_database_from_file (path_name: &str) -> Vec<DbTable> {
    let path = Path::new(path_name);
    let mut file = match File::open(path){
        Err(why) => panic!("couldn't open {}: {}", path_name,
                why.description()),
        Ok(file) => file,
    };

    let mut db_data = String::new();
    file.read_to_string(&mut db_data);

    let result = parser::parse_all_objects(db_data.as_bytes());

    match result{
        //IResult::Done(input, tables) => {
        IResult::Done(_, tables) => {
            let mut db: Vec<DbTable> = Vec::new();
            for table in tables {
                let mut columns: Vec<DbColumn> = Vec::new();
                for column in table.1 {
                    columns.push(DbColumn { name: column.0.to_string(), db_type: column.1.to_string() });
                }
                db.push(DbTable{ name: table.0.to_string(), columns:columns })
            }
            db
        },
        //IResult::Error (cause) => unimplemented!(),
        IResult::Error (_) => unimplemented!(),
        //IResult::Incomplete (size) => unimplemented!()
        IResult::Incomplete (_) => unimplemented!()
    }


}