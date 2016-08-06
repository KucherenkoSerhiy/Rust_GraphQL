//#[macro_use]
use mysql;

//use std::error::Error;
use std::vec::Vec;
use std::str;
use std::io::prelude::*;
use std::convert::Into;
use std::option::Option;

use mio::{EventLoop, EventSet, PollOpt};

use reader;
use parser;
use serialize;
use deserialize;
use nom::IResult;
use connection_pool::*;

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

pub struct mio_server{
    connection_pool: ConnectionPool,
    event_loop: EventLoop<ConnectionPool>
}

pub struct GraphQLPool {
    pub pool: mysql::Pool,
    pub async: mio_server,
    pub database: Vec<DbTable>,
    pub working_database_name: String
}

impl GraphQLPool {
    pub fn new (db_conn: &str, db_name: &str, path_name: &str) -> GraphQLPool{

        let db = reader::extract_database_from_file(path_name);

        let pool = mysql::Pool::new(db_conn).unwrap();
        let mut conn = pool.get_conn().unwrap();

        conn.query(serialize::create_database(db_name.to_string())).unwrap();
        conn.query(serialize::use_database(db_name.to_string())).unwrap();
        for table in & db {
            conn.query((&(serialize::create_table(db_name.to_string(), &table))).to_string()).unwrap();
        }

        let mut server = ConnectionPool::new();
        let mut event_loop = EventLoop::new().ok().expect("Failed to create event loop");
        event_loop.register(&server.socket,
                            SERVER_TOKEN,
                            EventSet::readable(),
                            PollOpt::edge()).unwrap();

        GraphQLPool{
            pool: pool,
            database: db,
            working_database_name: db_name.to_string(),
            async: mio_server{
                connection_pool:server,
                event_loop: event_loop
            }
        }
    }

    pub fn get (&self, query: &str) -> String {
        let select_query_data = parser::parse_select_query(query.as_bytes());
        match select_query_data{
            IResult::Done(_, select_structure) => {
                let mut mysql_select: String = serialize::perform_get((&self.working_database_name).to_string(), &select_structure);
                println!("Graph_QL_Pool::get:\n{}", mysql_select);
                deserialize::perform_get(&self.pool, mysql_select, &select_structure)
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

                let mut mysql_insert: String = serialize::perform_post_mutation((&self.working_database_name).to_string(), &insert_structure);
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
                let mut mysql_update: String = serialize::perform_update_mutation((&self.working_database_name).to_string(), &update_structure);

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
                let mut mysql_delete: String = serialize::perform_delete_mutation((&self.working_database_name).to_string(), &delete_structure);

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
        conn.query(serialize::destroy_database((&self.working_database_name).to_string())).unwrap();
    }
}

// TESTING AREA
