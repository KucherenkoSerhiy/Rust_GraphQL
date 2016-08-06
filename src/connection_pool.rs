use eventual::{Future, Async, Complete};
use mio::{Token, EventLoop, Sender, TryRead, TryWrite, EventSet, PollOpt, Handler};
use mio::tcp::{TcpStream, TcpListener};
use mio::util::Slab;
use mio;

use std::net::{SocketAddr,IpAddr,Ipv4Addr};
use std::collections::BTreeMap;
use std::borrow::Cow;
use std::error::Error;
use std::io;

use env_logger;

use connection::Connection;


pub const SERVER_TOKEN: Token = Token(0);

pub struct ConnectionPool {
    pub socket: TcpListener,
    token_counter: usize,
    connections: Slab<Connection>
}

impl ConnectionPool {

    pub fn new() -> ConnectionPool {
        let addr: SocketAddr = "0.0.0.0:10000".parse::<SocketAddr>()
            .ok().expect("Failed to parse host:port string");
        let server_socket = TcpListener::bind(&addr).ok().expect("Failed to bind address");

        ConnectionPool {
            socket: server_socket,
            connections: Slab::new_starting_at(Token(1), 32768),
            token_counter: 1,
        }
    }

/*
    fn create_connection(&mut self,event_loop: &mut EventLoop<ConnectionPool>,address:&IpAddr) -> RCResult<Token>{
        //println!("[ConnectionPool::create_connection]");
        let mut conn = try_rc!(connect(SocketAddr::new(address.clone(),9042),
                                None,
                                event_loop,
                                self.event_handler.clone()),"Failed connecting");
        let token = try_rc!(self.add_connection(address.clone(),conn),"Failed adding a new connection");
        Ok(token)
    }

    fn add_connection(&mut self, address:IpAddr,connection: Connection)-> RCResult<Token>{
        //println!("[ConnectionPool::add_connection]");
        let result = self.connections.insert(connection);

        match result{
            Ok(token) => {
                {
                    let conn = self.find_connection_by_token(token).ok().expect("Couldn't unwrap the connection");
                    //println!("Setting token {:?}",token);
                    conn.set_token(token);
                }
                self.token_by_ip.insert(address,token);
                Ok(token)
            },
            Err(err) => {
                Err(RCError::new("Credential should be provided for authentication", ReadError))
            }
        }
    }


    fn exists_connection_by_token(&mut self,token: Token) -> bool{
        self.connections.contains(token)
    }
*/
    fn find_connection_by_token(&mut self, token: Token) -> Result<&mut Connection,&'static str>{
        println!("[ConnectionPool::find_connection_by_token]");
        if !self.connections.is_empty() {
            let conn = Ok(self.connections.get_mut(token).unwrap());
            //println!("Connection with {:?} -> {:?}",token,conn );
            return conn;
        }
        Err("There is no connection found")
    }

}

impl Handler for ConnectionPool {
    type Timeout = usize;
    type Message = ();

    fn ready(&mut self,
             event_loop: &mut EventLoop<ConnectionPool>,
             token: Token,
             events: EventSet)
    {
        // A read event for our `Server` token means we are establishing a new connection.
        // A read event for any other token should be handed off to that connection.
        if events.is_readable() {
            match token {
                SERVER_TOKEN => {
                    let client_socket = match self.socket.accept() {
                        Err(e) => {
                            println!("Accept error: {}", e);
                            return;
                        },
                        Ok(None) => unreachable!("Accept has returned 'None'"),
                        Ok(Some((sock, addr))) => sock
                    };

                    let new_token = Token(self.token_counter);
                    self.token_counter += 1;
                    let connection = Connection::new(client_socket, new_token);

                    self.connections.insert(connection);
                    event_loop.register(&self.connections[new_token].socket, new_token,
                        EventSet::readable(), PollOpt::edge() | PollOpt::oneshot()
                    ).or_else(|e| {
                        Err(e)
                    });
                }
                token => {
                    let mut connection = self.connections.get_mut(token).unwrap();
                    connection.read();
                    event_loop.reregister(&connection.socket, connection.token, EventSet::readable(),
                                          PollOpt::edge() | PollOpt::oneshot()).unwrap();
                }
            }
        }

    }
}

/*
#[test]
fn test_pool(){
    // Before doing anything, let us register a logger. The mio library has really good logging
    // at the _trace_ and _debug_ levels. Having a logger setup is invaluable when trying to
    // figure out why something is not working correctly.
    env_logger::init().ok().expect("Failed to init logger");

    let mut event_loop = EventLoop::new().ok().expect("Failed to create event loop");
    let mut server = ConnectionPool::new();

    event_loop.register(&server.socket,
                        SERVER_TOKEN,
                        EventSet::readable(),
                        PollOpt::edge()).unwrap();
    event_loop.run(&mut server).unwrap();
    event_loop.shutdown();
}
*/