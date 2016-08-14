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
                GraphqlMsg::Request{operation, body} => {
                    //println!("Operation {}", operation);
                    //println!("{}", body);
                    match operation.as_str(){
                        "add" => {
                            self.add(&body);
                        },
                        "get" => {
                            self.get(&body);
                        },
                        "update" => {
                            self.update(&body);
                        },
                        "delete" => {
                            self.delete(&body);
                        },
                        _ => panic!("Wrong operation type")
                    }
                },
                _ => ()
            }
        }



    }
/*
    pub fn write(&mut self, event_loop: &mut EventLoop<ConnectionPool>) {
        let mut buf = ByteBuf::mut_with_capacity(2048);

        match self.socket.try_read_buf(&mut buf) {
            Ok(Some(0)) => {
                //println!("read 0 bytes");
            }
            Ok(Some(n)) => {
                self.response.mut_read_buf().extend_from_slice(&buf.bytes());
                //println!("read {} bytes", n);
                //println!("Read: {:?}",buf.bytes());
                self.read(event_loop);  //Recursion here, care

            }
            Ok(None) => {
                //println!("Reading buf = None");
                if !self.are_pendings_send(){
                    self.reregister(event_loop,EventSet::readable());
                }
                    else{
                    self.reregister(event_loop,EventSet::writable());
                }
            }
            Err(e) => {
                panic!("got an error trying to read; err={:?}", e);
            }
        }
    }
*/

/*
    pub fn write(&mut self, event_loop: &mut EventLoop<ConnectionPool>) {
        let mut buf = ByteBuf::mut_with_capacity(2048);
        //println!("self.pendings.len = {:?}",self.pendings_send.len());
        match self.pendings_send
            .pop_back()
            .unwrap()
            {
                CqlMsg::Request{request,tx,address} => {
                    //println!("Sending a request.");
                    request.serialize(&mut buf,self.version);
                    //println!("Sending: {:?}",request);
                    self.pendings_complete.insert(request.stream,CqlMsg::Request{request:request,tx:tx,address:address});
                },
                CqlMsg::Connect{request,tx,address} =>{
                    //println!("Sending a connect request.");
                    request.serialize(&mut buf,self.version);
                    self.pendings_complete.insert(request.stream,CqlMsg::Connect{request:request,tx:tx,address:address});
                },
                CqlMsg::Shutdown => {
                    panic!("Shutdown messages shouldn't be at pendings");
                },
            }
        match self.socket.try_write_buf(&mut buf.flip())
            {
                Ok(Some(n)) => {
                    //println!("Written {} bytes",n);
                    self.reregister(event_loop,EventSet::readable());
                }
                Ok(None) => {
                    // The socket wasn't actually ready, re-register the socket
                    // with the event loop
                    self.reregister(event_loop,EventSet::writable());
                }
                Err(e) => {
                    panic!("got an error trying to read; err={:?}", e);
                }
            }

        //println!("Ended write");
    }
*/

    pub fn get (&self, query: &str) -> String {
        println!("Graph_QL_Pool::get:\n{}\n---------------------------", query);
        let select_query_data = parser::parse_select_query(query.as_bytes());
        match select_query_data{
            IResult::Done(_, select_structure) => {
                let mut mysql_select: String = serialize::perform_get((&self.target.working_database_name).to_string(), &select_structure);
                println!("parsed");
                deserialize::perform_get(&self.target.pool, mysql_select, &select_structure)
            },
            IResult::Error (cause) => panic!("Graph_QL_Pool::get::Error: {}", cause),
            //IResult::Incomplete (size) => unimplemented!()
            IResult::Incomplete (_) => unimplemented!()
        }

    }

    pub fn add (&mut self, query: &str) /*-> Result<T,E>*/ {
        println!("Graph_QL_Pool::add:\n{}\n---------------------------", query);
        let insert_query_data = parser::parse_insert_query(query.as_bytes());
        match insert_query_data{
            //IResult::Done(input, insert_structure) => {
            IResult::Done(_, insert_structure) => {

                let mut mysql_insert: String = serialize::perform_add_mutation((&self.target.working_database_name).to_string(), &insert_structure);
                println!("parsed");
                let mut conn = self.target.pool.get_conn().unwrap();
                conn.query(&mysql_insert).unwrap();
            },
            //IResult::Error (cause) => unimplemented!(),
            IResult::Error (_) => unimplemented!(),
            //IResult::Incomplete (size) => unimplemented!()
            IResult::Incomplete (_) => unimplemented!()
        }
    }

    pub fn update (&mut self, query: &str) /*-> Result<T,E>*/ {
        println!("Graph_QL_Pool::update:\n{}\n---------------------------", query);
        let update_query_data = parser::parse_update_query(query.as_bytes());
        match update_query_data{
            //IResult::Done(input, update_structure) => {
            IResult::Done(_, update_structure) => {
                let mut mysql_update: String = serialize::perform_update_mutation((&self.target.working_database_name).to_string(), &update_structure);

                println!("parsed");
                let mut conn = self.target.pool.get_conn().unwrap();
                conn.query(&mysql_update).unwrap();
            },
            //IResult::Error (cause) => unimplemented!(),
            IResult::Error (_) => unimplemented!(),
            //IResult::Incomplete (size) => unimplemented!()
            IResult::Incomplete (_) => unimplemented!()
        }
    }

    pub fn delete (&mut self, query: &str) /*-> Result<T,E>*/ {
        println!("Graph_QL_Pool::delete:\n{}\n---------------------------", query);
        let delete_query_data = parser::parse_delete_query(query.as_bytes());
        match delete_query_data{
            //IResult::Done(input, delete_structure) => {
            IResult::Done(_, delete_structure) => {
                let mut mysql_delete: String = serialize::perform_delete_mutation((&self.target.working_database_name).to_string(), &delete_structure);
                println!("parsed");
                let mut conn = self.target.pool.get_conn().unwrap();
                conn.query(&mysql_delete).unwrap();
            },
            //IResult::Error (cause) => unimplemented!(),
            IResult::Error (_) => unimplemented!(),
            //IResult::Incomplete (size) => unimplemented!()
            IResult::Incomplete (_) => unimplemented!()
        }
    }
}




/*
struct WebSocketServer {
    socket: TcpListener,
    clients: HashMap<Token, TcpStream>,
    token_counter: usize
}

const SERVER_TOKEN: Token = Token(0);

impl Handler for WebSocketServer {
    type Timeout = usize;
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<WebSocketServer>,
             token: Token, events: EventSet)
    {
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

                self.token_counter += 1;
                let new_token = Token(self.token_counter);

                self.clients.insert(new_token, client_socket);
                event_loop.register(&self.clients[&new_token],
                                    new_token, EventSet::readable(),
                                    PollOpt::edge() | PollOpt::oneshot()).unwrap();
            }
            token => {
                let mut client = self.clients.get_mut(&token).unwrap();
                client.read();
                event_loop.reregister(&client.socket, token, EventSet::readable(),
                                      PollOpt::edge() | PollOpt::oneshot()).unwrap();
            }
        }
    }
}
    #[test]
    fn new ()/* -> WebSocketServer*/{
        let mut event_loop = EventLoop::new().unwrap();
        let address = "0.0.0.0:10000".parse::<SocketAddr>().unwrap();
        let server_socket = TcpListener::bind(&address).unwrap();

        let mut server = WebSocketServer {
            token_counter: 1,        // Starting the token counter from 1
            clients: HashMap::new(), // Creating an empty HashMap
            socket: server_socket    // Handling the ownership of the socket to the struct
        };

        event_loop.register(&server.socket,
                            SERVER_TOKEN,
                            EventSet::readable(),
                            PollOpt::edge()).unwrap();
        event_loop.run(&mut server).unwrap();
    }


/*
#[derive(Debug)]
pub struct Connection {
    // The connection's TCP socket
    socket: TcpStream,
    // The token used to register this connection with the EventLoop
    token: Token,
    // set of events we are interested in
    interest: EventSet,
    // messages waiting to be sent out
    send_queue: Vec<ByteBuf>,
}

impl Connection {
    fn new(sock: TcpStream, token: Token) -> Connection {
        Connection {
            socket: sock,
            token: token,

            // new connections are only listening for a hang up event when
            // they are first created. we always want to make sure we are
            // listening for the hang up event. we will additionally listen
            // for readable and writable events later on.
            interest: EventSet::hup(),

            send_queue: Vec::new(),
        }
    }

    /// Handle read event from event loop.
    ///
    /// Currently only reads a max of 2048 bytes. Excess bytes are dropped on the floor.
    ///
    /// The recieve buffer is sent back to `Server` so the message can be broadcast to all
    /// listening connections.
    fn readable(&mut self) -> io::Result<ByteBuf> {

        // ByteBuf is a heap allocated slice that mio supports internally. We use this as it does
        // the work of tracking how much of our slice has been used. I chose a capacity of 2048
        // after reading
        // https://github.com/carllerche/mio/blob/eed4855c627892b88f7ca68d3283cbc708a1c2b3/src/io.rs#L23-27
        // as that seems like a good size of streaming. If you are wondering what the difference
        // between messaged based and continuous streaming read
        // http://stackoverflow.com/questions/3017633/difference-between-message-oriented-protocols-and-stream-oriented-protocols
        // . TLDR: UDP vs TCP. We are using TCP.
        let mut recv_buf = ByteBuf::mut_with_capacity(2048);

        // we are PollOpt::edge() and PollOpt::oneshot(), so we _must_ drain
        // the entire socket receive buffer, otherwise the server will hang.
        loop {
            match self.socket.try_read_buf(&mut recv_buf) {
                // the socket receive buffer is empty, so let's move on
                // try_read_buf internally handles WouldBlock here too
                Ok(None) => {
                    debug!("CONN : we read 0 bytes");
                    break;
                },
                Ok(Some(n)) => {
                    debug!("CONN : we read {} bytes", n);

                    // if we read less than capacity, then we know the
                    // socket is empty and we should stop reading. if we
                    // read to full capacity, we need to keep reading so we
                    // can drain the socket. if the client sent exactly capacity,
                    // we will match the arm above. the recieve buffer will be
                    // full, so extra bytes are being dropped on the floor. to
                    // properly handle this, i would need to push the data into
                    // a growable Vec<u8>.
                    if n < recv_buf.capacity() {
                        break;
                    }
                },
                Err(e) => {
                    error!("Failed to read buffer for token {:?}, error: {}", self.token, e);
                    return Err(e);
                }
            }
        }

        // change our type from MutByteBuf to ByteBuf
        Ok(recv_buf.flip())
    }

    /// Handle a writable event from the event loop.
    ///
    /// Send one message from the send queue to the client. If the queue is empty, remove interest
    /// in write events.
    /// TODO: Figure out if sending more than one message is optimal. Maybe we should be trying to
    /// flush until the kernel sends back EAGAIN?


    /// Queue an outgoing message to the client.
    ///
    /// This will cause the connection to register interests in write events with the event loop.
    /// The connection can still safely have an interest in read events. The read and write buffers
    /// operate independently of each other.
    fn send_message(&mut self, message: ByteBuf) -> io::Result<()> {
        self.send_queue.push(message);
        self.interest.insert(EventSet::writable());
        Ok(())
    }

    /// Register interest in read events with the event_loop.
    ///
    /// This will let our connection accept reads starting next event loop tick.
    fn register(&mut self, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        self.interest.insert(EventSet::readable());

        event_loop.register(
            &self.socket,
            self.token,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            error!("Failed to register {:?}, {:?}", self.token, e);
            Err(e)
        })
    }

    /// Re-register interest in read events with the event_loop.
    fn reregister(&mut self, event_loop: &mut EventLoop<Server>) -> io::Result<()> {
        event_loop.reregister(
            &self.socket,
            self.token,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).or_else(|e| {
            error!("Failed to reregister {:?}, {:?}", self.token, e);
            Err(e)
        })
    }
}
*/
*/