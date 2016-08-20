use eventual::{Future, Async, Complete};
use mio;
use mio::util::Slab;
use mio::tcp::*;
use mio::{Token,EventLoop, Sender, TryRead, TryWrite, EventSet, PollOpt, Handler};
use std::{mem, str};
use std::net::{SocketAddr,IpAddr,Ipv4Addr};
use std::io::Error;
use std::collections::{VecDeque,BTreeMap, HashMap};
use std::io;
use std::io::ErrorKind;
use bytes::buf::ByteBuf;
use log::LogLevel;

use nom::IResult;
use serialize;
use deserialize;

use def::TargetPool;
use parser;
use connection_pool::*;


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
    target: TargetPool
}

impl Connection {
    pub fn new(socket:TcpStream, token: Token, targetPool: TargetPool) -> Connection{
        Connection {
            socket: socket,
            token: token,
            request_messages: Vec::new(),
            response_messages: Vec::new(),
            target: targetPool
        }
    }

    pub fn pushRequest(&mut self, msg: GraphqlMsg) {
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
                        _ => panic!("Wrong operation type")
                    };
                    self.response_messages.push(GraphqlMsg::Response{body: response_body.clone()});
                    tx.complete(response_body);
                },
                _ => panic!("Type of message not expected")
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

    pub fn get (&self, query: &str) -> String {
        //println!("Graph_QL_Pool::get:\n{}\n---------------------------", query);
        let select_query_data = parser::parse_select_query(query.as_bytes());
        match select_query_data{
            IResult::Done(_, select_structure) => {
                let mut mysql_select: String = serialize::perform_get((&self.target.working_database_name).to_string(), &select_structure);
                println!("CONNECTION::GET:\n{}", mysql_select);
                deserialize::perform_get(&self.target.pool, mysql_select, &select_structure)
            },
            IResult::Error (cause) => panic!("Graph_QL_Pool::get::Error: {}", cause),
            //IResult::Incomplete (size) => unimplemented!()
            IResult::Incomplete (_) => unimplemented!()
        }
    }

    pub fn add (&mut self, query: &str) -> String {
        //println!("Graph_QL_Pool::add:\n{}\n---------------------------", query);
        let insert_query_data = parser::parse_insert_query(query.as_bytes());
        match insert_query_data{
            //IResult::Done(input, insert_structure) => {
            IResult::Done(_, insert_structure) => {

                let mut mysql_insert: String = serialize::perform_add_mutation((&self.target.working_database_name).to_string(), &insert_structure);
                //println!("CONNECTION::ADD:\n{}", mysql_insert);
                let mut conn = self.target.pool.get_conn().unwrap();
                conn.query(&mysql_insert).unwrap();
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
        let update_query_data = parser::parse_update_query(query.as_bytes());
        match update_query_data{
            //IResult::Done(input, update_structure) => {
            IResult::Done(_, update_structure) => {
                let mut mysql_update: String = serialize::perform_update_mutation((&self.target.working_database_name).to_string(), &update_structure);

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
        let delete_query_data = parser::parse_delete_query(query.as_bytes());
        match delete_query_data{
            //IResult::Done(input, delete_structure) => {
            IResult::Done(_, delete_structure) => {
                let mut mysql_delete: String = serialize::perform_delete_mutation((&self.target.working_database_name).to_string(), &delete_structure);
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
}