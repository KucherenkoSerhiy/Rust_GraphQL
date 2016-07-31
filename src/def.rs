//#[macro_use]
use mysql;

//use std::error::Error;
use std::vec::Vec;
use std::str;
use std::io::prelude::*;
use std::convert::Into;
use std::borrow::Cow;
use std::option::Option;

use reader;
use parser;
use nom::IResult;

#[derive(Debug, PartialEq, Eq)]
pub struct DbColumn {
    pub name: String,
    pub db_type: String
}

#[derive(Debug, PartialEq, Eq)]
pub struct DbTable {
    pub name: String,
    pub columns: Vec<DbColumn>
}

pub struct GraphQLPool {
    pub pool: mysql::Pool,
    pub database: Vec<DbTable>,
    pub working_database_name: String
}

impl GraphQLPool {
    pub fn new (db_conn: &str, db_name: &str, path_name: &str) -> GraphQLPool{

        let db = reader::extract_database_from_file(path_name);

        let mut load_table_query: String = "".to_string();
        for table in & db {
            //creates temporary table with auto-generated id
            //load_table_query = load_table_query + "DROP TABLE IF EXISTS " + db_name + "." + &table.name + ";\n";
            load_table_query = load_table_query + "CREATE TABLE IF NOT EXISTS " + db_name + "." + &table.name; load_table_query = load_table_query + "(
                         " + &table.name + "_id INT NOT NULL PRIMARY KEY AUTO_INCREMENT"; for column in &table.columns {load_table_query = load_table_query + ",
                         "+ &column.name + " "+ &column.db_type}; load_table_query = load_table_query +"
                     );\n";
        }
        println!("{}", load_table_query);

        let pool = mysql::Pool::new(db_conn).unwrap();

        let mut conn = pool.get_conn().unwrap();
        //conn.query("DROP DATABASE IF EXISTS ".to_string() + db_name).unwrap();
        conn.query("CREATE DATABASE IF NOT EXISTS ".to_string() + db_name).unwrap();
        conn.query("USE ".to_string() + db_name).unwrap();

        conn.query(load_table_query).unwrap();

        GraphQLPool{
            pool: pool,
            database: db,
            working_database_name: db_name.to_string()
        }
    }

    pub fn get (&self, query: &str) -> String {
        let select_query_data = parser::parse_select_query(query.as_bytes());
        match select_query_data{

            //IResult::Done(input, select_structure) => {
            IResult::Done(_, select_structure) => {
                //select_structure : (&str, (&str, &str), Vec<&str>)

                let last_column = select_structure.2.last().unwrap();
                let mut mysql_select: String = "SELECT ".to_string();
                for col in &select_structure.2{
                    mysql_select = mysql_select + col;
                    if col != last_column {mysql_select = mysql_select + ","};
                    mysql_select = mysql_select + " "
                }
                mysql_select = mysql_select +

                    "FROM " + &(self.working_database_name) + "." + select_structure.0 + " " +

                    "WHERE " + (select_structure.1).0 + " = " + (select_structure.1).1 + ";";

                println!("Graph_QL_Pool::get:\n{}", mysql_select);

                let mut resulting_object : String = "".to_string();

                self.pool.prep_exec(mysql_select, ()).map(|mut result| {
                    let mut row = result.next().unwrap().unwrap();

                    resulting_object = "{\n  \"data\": {\n".to_string();
                    for col in &select_structure.2{
                        //let data : ColumnType = row.take(*col).unwrap();
                        let data : String = row.take(*col).unwrap();
                        match data {
                            _ => resulting_object = resulting_object + "    \"" + col + "\": \"" + &data + "\"\n"
                        };
                    }
                    resulting_object = resulting_object + "  }\n}";
                    println!("{}", resulting_object);
                    //let name: String = row.take("name").unwrap();
                    resulting_object

                }).unwrap()

            },
            IResult::Error (cause) => panic!("Graph_QL_Pool::get::Error: {}", cause),
            //IResult::Incomplete (size) => unimplemented!()
            IResult::Incomplete (_) => unimplemented!()
        }

    }

    pub fn post (&mut self, query: &str) /*-> Result<T,E>*/ {
        let insert_query_data = parser::parse_insert_query(query.as_bytes());
        match insert_query_data{
            //IResult::Done(input, insert_structure) => {
            IResult::Done(_, insert_structure) => {
                //insert_structure : (&str, Vec<(&str, &str)> )
                let last_column = &insert_structure.1.last().unwrap();
                let mut mysql_insert: String = "INSERT INTO ".to_string() + &(self.working_database_name) + "." + insert_structure.0 + "(";
                                        /*COLUMNS*/
                                        for col in &insert_structure.1{
                                            mysql_insert = mysql_insert + col.0;
                                            if col.0 != last_column.0 {mysql_insert = mysql_insert + ","};
                                            mysql_insert = mysql_insert + " ";
                                        }

                                        mysql_insert = mysql_insert + ")\n" +

                                        "VALUES (";
                                        for col in &insert_structure.1{
                                            mysql_insert = mysql_insert + "\"" + col.1 + "\"";;
                                            if col.1 != last_column.1 {mysql_insert = mysql_insert + ","};
                                            mysql_insert = mysql_insert + " ";
                                        }
                                        mysql_insert = mysql_insert + ");";
                println!("Graph_QL_Pool::post:\n{}", mysql_insert);
                let mut conn = self.pool.get_conn().unwrap();
                conn.query(&mysql_insert).unwrap();
            },
            //IResult::Error (cause) => unimplemented!(),
            IResult::Error (_) => unimplemented!(),
            //IResult::Incomplete (size) => unimplemented!()
            IResult::Incomplete (_) => unimplemented!()
        }
    }

    pub fn update (&mut self, query: &str) /*-> Result<T,E>*/ {
        let update_query_data = parser::parse_update_query(query.as_bytes());
        match update_query_data{
            //IResult::Done(input, update_structure) => {
            IResult::Done(_, update_structure) => {
                //update_structure : (&str, (&str, &str), Vec<(&str, &str)> )
                let last_column = &update_structure.2.last().unwrap();
                let mut mysql_update: String = "UPDATE ".to_string() + &(self.working_database_name) + "." + update_structure.0 + " SET ";
                /*COLUMNS*/
                for col in &update_structure.2{
                    mysql_update = mysql_update + col.0 + " = " + col.1;
                    if col.0 != last_column.0 {mysql_update = mysql_update + ","};
                    mysql_update = mysql_update + " ";
                }

                mysql_update = mysql_update + "WHERE " + (update_structure.1).0 + " = " + (update_structure.1).1 + ";";
                println!("Graph_QL_Pool::post:\n{}", mysql_update);
                let mut conn = self.pool.get_conn().unwrap();
                conn.query(&mysql_update).unwrap();
            },
            //IResult::Error (cause) => unimplemented!(),
            IResult::Error (_) => unimplemented!(),
            //IResult::Incomplete (size) => unimplemented!()
            IResult::Incomplete (_) => unimplemented!()
        }
    }

    pub fn delete (&mut self, query: &str) /*-> Result<T,E>*/ {
        let delete_query_data = parser::parse_delete_query(query.as_bytes());
        match delete_query_data{
            //IResult::Done(input, delete_structure) => {
            IResult::Done(_, delete_structure) => {
                //delete_structure : (&str, (&str, &str)
                let mut mysql_delete: String = "DELETE FROM ".to_string() + &(self.working_database_name) + "." + delete_structure.0 + " ";
                if let Some(id) = delete_structure.1 {
                    mysql_delete = mysql_delete + "WHERE " + id.0 + "=" + id.1;
                }
                mysql_delete = mysql_delete + ";";
                println!("Graph_QL_Pool::post:\n{}", mysql_delete);
                let mut conn = self.pool.get_conn().unwrap();
                conn.query(&mysql_delete).unwrap();
            },
            //IResult::Error (cause) => unimplemented!(),
            IResult::Error (_) => unimplemented!(),
            //IResult::Incomplete (size) => unimplemented!()
            IResult::Incomplete (_) => unimplemented!()
        }
    }

    pub fn destroy (&mut self){
        let mut conn = self.pool.get_conn().unwrap();
        conn.query("DROP DATABASE IF EXISTS ".to_string() + &(self.working_database_name)).unwrap();
    }
}

// TESTING AREA
