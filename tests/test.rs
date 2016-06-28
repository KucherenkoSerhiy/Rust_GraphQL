#[macro_use]
extern crate rust_sql;

#[macro_use]
extern crate mysql;
extern crate mio;
extern crate eventual;
#[macro_use]
extern crate nom;

#[macro_use]
extern crate log;
extern crate env_logger;

use rust_sql::*;
use rust_sql::def::GraphQLPool;
use mio::{Token, EventLoop};
use eventual::*;
use mysql as my;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

#[test]
fn test_sql_connection(){
    #[derive(Debug, PartialEq, Eq)]
    struct Payment {
        customer_id: i32,
        amount: i32,
        account_name: Option<String>,
    }

    let pool = my::Pool::new("mysql://root:root@localhost:3306").unwrap(); //mysql://username:password@host:port

    // Let's create payment table.
    // It is temporary so we do not need `tmp` database to exist.
    // Unwap just to make sure no error happened.
    pool.prep_exec(r"CREATE TEMPORARY TABLE tmp.payment (
                         customer_id int not null,
                         amount int not null,
                         account_name text
                     )", ()).unwrap();

    let payments = vec![
        Payment { customer_id: 1, amount: 2, account_name: None },
        Payment { customer_id: 3, amount: 4, account_name: Some("foo".into()) },
        Payment { customer_id: 5, amount: 6, account_name: None },
        Payment { customer_id: 7, amount: 8, account_name: None },
        Payment { customer_id: 9, amount: 10, account_name: Some("bar".into()) },
    ];

    // Let's insert payments to the database
    // We will use into_iter() because we do not need to map Stmt to anything else.
    // Also we assume that no error happened in `prepare`.
    for mut stmt in pool.prepare(r"INSERT INTO tmp.payment
                                       (customer_id, amount, account_name)
                                   VALUES
                                       (:customer_id, :amount, :account_name)").into_iter() {
        for p in payments.iter() {
            // `execute` takes ownership of `params` so we pass account name by reference.
            // Unwrap each result just to make sure no errors happened.
            stmt.execute(params!{
                "customer_id" => my::Value::from(p.customer_id),
                "amount" => my::Value::from(p.amount),
                "account_name" => my::Value::from(&p.account_name),
            }).unwrap();
        }
    }

    // Let's select payments from database
    let selected_payments: Vec<Payment> =
    pool.prep_exec("SELECT customer_id, amount, account_name from tmp.payment", ())
        .map(|result| { // In this closure we sill map `QueryResult` to `Vec<Payment>`
            // `QueryResult` is iterator over `MyResult<row, err>` so first call to `map`
            // will map each `MyResult` to contained `row` (no proper error handling)
            // and second call to `map` will map each `row` to `Payment`
            result.map(|x| x.unwrap()).map(|row| {
                let (customer_id, amount, account_name) = my::from_row(row);
                Payment {
                    customer_id: customer_id,
                    amount: amount,
                    account_name: account_name,
                }
            }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Payment>`
        }).unwrap(); // Unwrap `Vec<Payment>`

    // Now make sure that `payments` equals to `selected_payments`.
    // Mysql gives no guaranties on order of returned rows without `ORDER BY`
    // so assume we are lukky.
    assert_eq!(payments, selected_payments);

}

/*
const SERVER: Token = Token(0);
const CLIENT: Token = Token(1);

#[test]
fn test_mio_echo_server () {
        debug!("Starting TEST_ECHO_SERVER");
        let mut event_loop = EventLoop::new().unwrap();

        let addr = localhost();
        let srv = TcpListener::bind(&addr).unwrap();

        info!("listen for connections");
        event_loop.register(&srv, SERVER, EventSet::readable(),
                            PollOpt::edge() | PollOpt::oneshot()).unwrap();

        let sock = TcpStream::connect(&addr).unwrap();

        // Connect to the server
        event_loop.register(&sock, CLIENT, EventSet::writable(),
                            PollOpt::edge() | PollOpt::oneshot()).unwrap();

        // Start the event loop
        event_loop.run(&mut Echo::new(srv, sock, vec!["foo", "bar"])).unwrap();
}
*/

#[test]
fn test_eventual () {
    // Run a computation in another thread
    let future1 = Future::spawn(|| {
        // Represents an expensive computation, but for now just return a
        // number
        42
    });

    // Run another computation
    let future2 = Future::spawn(|| {
        // Another expensive computation
        18
    });

    let res = join((
                       future1.map(|v| v * 2),
                       future2.map(|v| v + 5)))
        .and_then(|(v1, v2)| Ok(v1 - v2))
        .await().unwrap();

    assert_eq!(61, res);
}

#[test]
fn test_get_simple_data_from_db () {

    #[derive(Debug, PartialEq, Eq)]
    struct User {
        id: i32,
        name: String
    }

    let mut graphQLPool = GraphQLPool::new(
        "mysql://root:root@localhost:3306",
        "/home/serhiy/Desktop/rust-sql/types"
    );
    let mut s = String::new();
    graphQLPool.init_db_file.read_to_string(&mut s);

    graphQLPool.pool.prep_exec(r"CREATE TEMPORARY TABLE tmp.user (
                         id int not null,
                         name text
                     )", ()).unwrap();

    let users = vec![
        User { id: 1, name: "foo".into() },
        User { id: 2, name: "bar".into() }
    ];

    for mut stmt in graphQLPool.pool.prepare(r"INSERT INTO tmp.user
                                       (id, name)
                                   VALUES
                                       (:id, :name)").into_iter() {
        for p in users.iter() {
            stmt.execute(params!{
                "id" => my::Value::from(p.id),
                "name" => my::Value::from(&p.name),
            }).unwrap();
        }
    }

    let selected_users: Vec<User> =
    graphQLPool.pool.prep_exec("SELECT * FROM tmp.user", ())
        .map(|result| {
            result.map(|x| x.unwrap()).map(|row| {
                let (id, name) = my::from_row(row);
                User {
                    id: id,
                    name: name,
                }
            }).collect()
        }).unwrap();

    assert_eq!(users, selected_users);


    /*
    let query =
    "
        {
            user(id: '1') {
                name
            }
        }
    ";

    let expected_answer =
    "
        {
            'data': {
                'user': {
                    'name': 'foo'
                }
            }
        }
    "
    ;
    let answer = graphQLPool.graphql(query);
    */
}