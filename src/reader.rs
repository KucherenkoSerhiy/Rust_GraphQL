use def::*;
use parser;

use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::vec::Vec;
use std::str;
use std::io::prelude::*;

use nom::IResult;

fn graphql_to_mysql_type (attr_type: String) -> String {
    match attr_type.as_str(){
        "Number" => "INT".to_string(),
        "String" => "TEXT(2048)".to_string(),
        "Boolean" => "BOOLEAN".to_string(),
        _ => attr_type.to_string()
    }
}

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
        IResult::Done(_, tables) => {
            let mut db: Vec<DbTable> = Vec::new();
            for table in tables {
                let mut columns: Vec<DbColumn> = Vec::new();
                for column in table.1 {
                    columns.push(DbColumn { name: column.0.to_string(), db_type: graphql_to_mysql_type(column.1.to_string()), is_mandatory: column.2});
                }
                db.push(DbTable{ name: table.0.to_string(), columns:columns })
            }
            db
        },
        IResult::Error (_) => unimplemented!(),
        IResult::Incomplete (_) => unimplemented!()
    }


}