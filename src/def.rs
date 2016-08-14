//#[macro_use]
use mysql;

//use std::error::Error;
use std::vec::Vec;
use std::str;
use std::io::prelude::*;
use std::convert::Into;
use std::option::Option;

use mio::{EventLoop, EventSet, PollOpt, Sender};
use nom::IResult;

use reader;
use parser;
use serialize;
use deserialize;
use connection_pool::*;
use connection::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DbColumn {
    pub name: String,
    pub db_type: String
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DbTable {
    pub name: String,
    pub columns: Vec<DbColumn>
}

#[derive(Clone)]
pub struct TargetPool {
    pub pool: mysql::Pool,
    pub database: Vec<DbTable>,
    pub working_database_name: String
}

#[derive(Clone)]
pub struct GraphQLPool {
    pub pool: mysql::Pool,
    pub sender: Sender<GraphqlMsg>,
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

        let mut targetPool = TargetPool{
            pool: pool.clone(),
            database: db.clone(),
            working_database_name: db_name.to_string(),
        };

        GraphQLPool{
            sender: ConnectionPool::new(targetPool.clone()),
            pool: pool,
            database: db,
            working_database_name: db_name.to_string(),
        }
    }

    pub fn get (&self, query: &str) -> String {
        /*
        self.sender.send(GraphqlMsg::Request{
            operation: "get".to_string(),
            body: query.to_string()
        }).unwrap();
        */
        let select_query_data = parser::parse_select_query(query.as_bytes());
        match select_query_data{
            IResult::Done(_, select_structure) => {
                let mut mysql_select: String = serialize::perform_get((&self.working_database_name).to_string(), &select_structure);
                println!("DEF:RS: Graph_QL_Pool::get:\n{}", mysql_select);
                deserialize::perform_get(&self.pool, mysql_select, &select_structure)
            },
            IResult::Error (cause) => panic!("Graph_QL_Pool::get::Error: {}", cause),
            //IResult::Incomplete (size) => unimplemented!()
            IResult::Incomplete (_) => unimplemented!()
        }
    }


    pub fn add (&mut self, query: &str) /*-> Result<T,E>*/ {
        self.sender.send(GraphqlMsg::Request{
            operation: "add".to_string(),
            body: query.to_string()
        }).unwrap();
    }

    pub fn update (&mut self, query: &str) /*-> Result<T,E>*/ {
        self.sender.send(GraphqlMsg::Request{
            operation: "update".to_string(),
            body: query.to_string()
        }).unwrap();
    }

    pub fn delete (&mut self, query: &str) /*-> Result<T,E>*/ {
        self.sender.send(GraphqlMsg::Request{
            operation: "delete".to_string(),
            body: query.to_string()
        }).unwrap();
    }

    pub fn destroy_database (&mut self){
        let mut conn = self.pool.get_conn().unwrap();
        conn.query(serialize::destroy_database((&self.working_database_name).to_string())).unwrap();
    }
}

// TESTING AREA
