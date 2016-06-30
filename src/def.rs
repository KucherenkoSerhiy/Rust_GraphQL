#[macro_use]
use mysql;

use std::fmt::Debug;
use std::any::Any;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

use parser::*;

#[derive(Debug, PartialEq, Eq)]
pub struct db_column <'T> {
    pub name: &'T [u8],
    pub db_type: &'T [u8]
}

pub struct GraphQLPool {
    #[derive(Debug, PartialEq, Eq)]
    pub pool: mysql::Pool,
    pub init_db_file: File
}

impl GraphQLPool {
    ///db_params has the structure 'username:password@host:port'
    /*
        EXAMPLE:
        mysql://root:root@localhost:3306,


        type Human : Character {
            id: String
            name: String
            friends: [Character]
            appearsIn: [Episode]
            homePlanet: String
        }
    */
    pub fn new (db_conn: &str, path_name: &str) -> GraphQLPool{
        let p = mysql::Pool::new(db_conn).unwrap();
        let path = Path::new(path_name);
        GraphQLPool{
            pool: p,
            init_db_file: match File::open(path){
                // The `description` method of `io::Error` returns a string that
                // describes the error
                Err(why) => panic!("couldn't open {}: {}", path_name,
                why.description()),
                Ok(file) => file,
            }
        }
    }

    /*
    pub fn create_table <T: Any + Debug, E: Any + Debug> (query: &str) -> Result<T,E> {

    }
    */
    /*
    pub fn create (query: &str) -> Result<T,E> {

    }
    */

    /*
    pub fn get (query: &str) -> Result<T,E> {

    }
    */
/*
    pub fn update (query: &str) -> Result<T,E> {

    }

    pub fn delete (query: &str) -> Result<T,E> {

    }
*/
}