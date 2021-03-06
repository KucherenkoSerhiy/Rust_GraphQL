use std::vec::Vec;
use std::str;

use mysql;
use mio::{Sender};
use eventual::Future;

use reader;
use serialize;
use connection_pool::*;
use connection::*;
use def::*;

#[derive(Clone)]
pub struct GraphQLPool {
    pub sender: Sender<GraphqlMsg>,
}

impl GraphQLPool {
    pub fn new (db_conn: &str, db_name: &str, path_name: &str) -> GraphQLPool{

        let db = reader::extract_database_from_file(path_name);

        let pool = mysql::Pool::new(db_conn).unwrap();
        let mut conn = pool.get_conn().unwrap();
        let mut serializer = serialize::Serializer::new();

        conn.query(serializer.create_database(db_name.to_string())).unwrap();
        conn.query(serializer.use_database(db_name.to_string())).unwrap();

        let mut relations : Vec<Relation> = Vec::new();
        for table in & db {
            let (query, mut rels) = serializer.create_table(db_name.to_string(), &table);
            relations.append(&mut rels);
            conn.query(query).unwrap();
        }

        serializer.store_relations(&mut relations);
        for rel in &serializer.relations {
            let query = serializer.create_relation_table(db_name.to_string(), &rel);
            conn.query(query).unwrap();
        }

        let target_pool = TargetPool{
            pool: pool.clone(),
            database: db.clone(),
            working_database_name: db_name.to_string(),
        };

        GraphQLPool{
            sender: ConnectionPool::new(target_pool.clone(), serializer),
        }
    }

    pub fn get (&self, query: &str) -> Future<String, ()> {
        let (tx, future) = Future::<String, ()>::pair();
        self.sender.send(GraphqlMsg::Request{
            operation: "get".to_string(),
            body: query.to_string(),
            tx: tx
        }).unwrap();
        future
    }


    pub fn add (&mut self, query: &str) -> Future<String, ()> {
        let (tx, future) = Future::<String, ()>::pair();
        self.sender.send(GraphqlMsg::Request{
            operation: "add".to_string(),
            body: query.to_string(),
            tx: tx
        }).unwrap();
        future
    }

    pub fn update (&mut self, query: &str) -> Future<String, ()> {
        let (tx, future) = Future::<String, ()>::pair();
        self.sender.send(GraphqlMsg::Request{
            operation: "update".to_string(),
            body: query.to_string(),
            tx: tx
        }).unwrap();
        future
    }

    pub fn delete (&mut self, query: &str) {
        let (tx, future) = Future::<String, ()>::pair();
        self.sender.send(GraphqlMsg::Request{
            operation: "delete".to_string(),
            body: query.to_string(),
            tx: tx
        }).unwrap();
    }

    pub fn mysql_query (&mut self, query: &str) {
        let (tx, future) = Future::<String, ()>::pair();
        self.sender.send(GraphqlMsg::Request{
            operation: "query".to_string(),
            body: query.to_string(),
            tx: tx
        }).unwrap();
    }

    pub fn destroy_database (&mut self){
        let (tx, future) = Future::<String, ()>::pair();
        self.sender.send(GraphqlMsg::Request{
            operation: "destroy_db".to_string(),
            body: "".to_string(),
            tx: tx
        }).unwrap();
    }
}