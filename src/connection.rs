use eventual::Complete;
use mio::tcp::*;
use mio::Token;
use std::str;

use nom::IResult;
use serialize::*;
use deserialize::*;

use def::TargetPool;
use parser;


pub enum GraphqlMsg{
    Connect,
    Request{
        operation: String,
        body: String,
        tx: Complete<String,()>,
    },
    Response{
        body: String,
    },
    Shutdown
}

pub struct Connection {
    // handle to the accepted socket
    pub socket: TcpStream,

    // token used to register with the event loop
    pub token: Token,

    pub request_messages: Vec<GraphqlMsg>,
    pub response_messages: Vec<GraphqlMsg>,
    target: TargetPool,
    serializer: Serializer,
    deserializer: Deserializer
}

impl Connection {
    pub fn new(socket:TcpStream, token: Token, target_pool: TargetPool, serializer: Serializer) -> Connection{
        Connection {
            socket: socket,
            token: token,
            request_messages: Vec::new(),
            response_messages: Vec::new(),
            target: target_pool,
            serializer: serializer,
            deserializer: Deserializer::new()
        }
    }

    pub fn push_request(&mut self, msg: GraphqlMsg) {
        self.request_messages.push(msg);
    }

    pub fn process(&mut self){
        while !self.request_messages.is_empty(){
            let msg = self.request_messages.remove(0);
            match msg {
                GraphqlMsg::Request{operation, body, tx} => {
                    //println!("Operation {}", operation);
                    //println!("{}", body);
                    let response_body = match operation.as_str(){
                        "add" => {
                            self.add(&body)
                        },
                        "get" => {
                            self.get(&body)
                        },
                        "update" => {
                            self.update(&body)
                        },
                        "delete" => {
                            self.delete(&body)
                        },
                        "query" => {
                            self.process_mysql_query(&body)
                        },
                        "destroy_db" => {
                            self.destroy_database()
                        },
                        _ => panic!("Error: Wrong operation type")
                    };
                    self.response_messages.push(GraphqlMsg::Response{body: response_body.clone()});
                    tx.complete(response_body);
                },
                _ => panic!("Error: Unexpected type of message")
            }
        }
    }

    pub fn get_responses (&mut self) -> Vec<GraphqlMsg>{
        let mut result: Vec<GraphqlMsg> = Vec::new();
        while !self.response_messages.is_empty() {
            result.push(self.response_messages.remove(0));
        }
        result
    }

    pub fn get (&mut self, query: &str) -> String {
        //println!("Graph_QL_Pool::get:\n{}\n---------------------------", query);
        let select_query_data = parser::parse_query(query.as_bytes());
        match select_query_data{
            IResult::Done(_, select_structure) => {
                //println!("get structure: {:?}", select_structure);
                let mysql_select_origin_ids = self.serializer.perform_get_ids((&self.target.working_database_name).to_string(), &select_structure);
                let origin_ids : Vec<i32> = self.deserializer.perform_get_ids(&self.target.pool, mysql_select_origin_ids);

                let mysql_select: String = self.serializer.perform_get((&self.target.working_database_name).to_string(), &select_structure);
                let mysql_select_rels: String = self.serializer.perform_get_rels((&self.target.working_database_name).to_string(), &select_structure, origin_ids);

                self.deserializer.perform_get(&self.target.pool, mysql_select, mysql_select_rels, &select_structure)
            },
            IResult::Error (cause) => panic!("Graph_QL_Pool::get::Error: {}", cause),
            //IResult::Incomplete (size) => unimplemented!()
            IResult::Incomplete (_) => unimplemented!()
        }
    }

    pub fn add (&mut self, query: &str) -> String {
        //println!("Graph_QL_Pool::add:\n{}\n---------------------------", query);
        let insert_query_data = parser::parse_mutation_query(query.as_bytes());
        match insert_query_data{
            //IResult::Done(input, insert_structure) => {
            IResult::Done(_, insert_structure) => {

                let mut conn = self.target.pool.get_conn().unwrap();
                self.serializer.perform_add_mutation(&mut conn, (&self.target.working_database_name).to_string(), &insert_structure);
            },
            //IResult::Error (cause) => unimplemented!(),
            IResult::Error (_) => unimplemented!(),
            //IResult::Incomplete (size) => unimplemented!()
            IResult::Incomplete (_) => unimplemented!()
        }
        "add response completed".to_string()
    }

    pub fn update (&mut self, query: &str) -> String {
        //println!("Graph_QL_Pool::update:\n{}\n---------------------------", query);
        let update_query_data = parser::parse_mutation_query(query.as_bytes());
        match update_query_data{
            //IResult::Done(input, update_structure) => {
            IResult::Done(_, update_structure) => {
                let mysql_update: String = self.serializer.perform_update_mutation((&self.target.working_database_name).to_string(), &update_structure);

                //println!("parsed");
                let mut conn = self.target.pool.get_conn().unwrap();
                conn.query(&mysql_update).unwrap();
            },
            //IResult::Error (cause) => unimplemented!(),
            IResult::Error (_) => unimplemented!(),
            //IResult::Incomplete (size) => unimplemented!()
            IResult::Incomplete (_) => unimplemented!()
        }
        "update response completed".to_string()
    }

    pub fn delete (&mut self, query: &str) -> String {
        //println!("Graph_QL_Pool::delete:\n{}\n---------------------------", query);
        let delete_query_data = parser::parse_mutation_query(query.as_bytes());
        match delete_query_data{
            //IResult::Done(input, delete_structure) => {
            IResult::Done(_, delete_structure) => {
                let mysql_delete: String = self.serializer.perform_delete_mutation((&self.target.working_database_name).to_string(), &delete_structure);
                //println!("parsed");
                let mut conn = self.target.pool.get_conn().unwrap();
                conn.query(&mysql_delete).unwrap();
            },
            //IResult::Error (cause) => unimplemented!(),
            IResult::Error (_) => unimplemented!(),
            //IResult::Incomplete (size) => unimplemented!()
            IResult::Incomplete (_) => unimplemented!()
        }
        "delete response completed".to_string()
    }

    pub fn process_mysql_query (&mut self, query: &str) -> String {
        let mut conn = self.target.pool.get_conn().unwrap();
        conn.query(query).unwrap();
        "query executed".to_string()
    }

    pub fn destroy_database (&mut self) -> String {
        let mut conn = self.target.pool.get_conn().unwrap();
        conn.query("DROP DATABASE ".to_string() + (&self.target.working_database_name) + ";").unwrap();
        "database dropped".to_string()
    }
}