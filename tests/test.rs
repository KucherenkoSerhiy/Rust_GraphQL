//To successfully run tests please set environment variable RUST_TEST_THREADS to 1

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


use rust_sql::graphql_pool::*;
use eventual::*;
use mysql as my;
use std::str;
use std::str::FromStr;
use std::vec::Vec;
use std::option::Option;
use std::thread;
use nom::{IResult,digit};
use nom::IResult::*;

const DB_NAME: &'static str = "lotr_db";
const DB_USER: &'static str = "root";
const DB_PASSWORD: &'static str = "root";
const HOST: &'static str = "localhost";
const PORT: &'static str = "3306";
const FILE_NAME: &'static str = "types";
const FILE_LOCATION: &'static str = "/home/serhiy/Desktop/rust-sql";


#[test]
fn test_mysql_module(){
    #[derive(Debug, PartialEq, Eq)]
    struct Payment {
        customer_id: i32,
        amount: i32,
        account_name: Option<String>,
    }

    let mysql_connection = "mysql://".to_string()+DB_USER+":"+DB_PASSWORD+"@"+HOST+":"+PORT;
    let pool = my::Pool::new(mysql_connection.as_str()).unwrap(); //mysql://username:password@host:port
    let mut conn = pool.get_conn().unwrap();
    //conn.query("DROP DATABASE IF EXISTS ".to_string() + DB_NAME).unwrap();
    conn.query("CREATE DATABASE IF NOT EXISTS ".to_string() + DB_NAME).unwrap();

    // Let's create payment table.
    // It is temporary so we do not need `tmp` database to exist.
    // Unwap just to make sure no error happened.
    pool.prep_exec("CREATE TEMPORARY TABLE ".to_string() + DB_NAME + ". payment (
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
    for mut stmt in pool.prepare(r"INSERT INTO  ".to_string() + DB_NAME + ". payment
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
    pool.prep_exec("SELECT customer_id, amount, account_name from ".to_string() + DB_NAME + ".payment", ())
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

#[test]
fn test_nom_module(){
    named!(parens<i64>, delimited!(
        char!('('),
        expr,
        char!(')')
      )
    );

    named!(i64_digit<i64>,
      map_res!(
        map_res!(
          digit,
          str::from_utf8
        ),
        FromStr::from_str
      )
    );

    // We transform an integer string into a i64
    // we look for a digit suite, and try to convert it.
    // if either str::from_utf8 or FromStr::from_str fail,
    // the parser will fail
    named!(factor<i64>,
      alt!(
        i64_digit
      | parens
      )
    );

    // we define acc as mutable to update its value whenever a new term is found
    named!(term <i64>,
      chain!(
        mut acc: factor  ~
                 many0!(
                   alt!(
                     tap!(mul: preceded!(tag!("*"), factor) => acc = acc * mul) |
                     tap!(div: preceded!(tag!("/"), factor) => acc = acc / div)
                   )
                 ),
        || { return acc }
      )
    );

    named!(expr <i64>,
      chain!(
        mut acc: term  ~
                 many0!(
                   alt!(
                     tap!(add: preceded!(tag!("+"), term) => acc = acc + add) |

                     tap!(sub: preceded!(tag!("-"), term) => acc = acc - sub)
                   )
                 ),
        || { return acc }
      )
    );

    assert_eq!(expr(b"1+2"),         IResult::Done(&b""[..], 3));
    assert_eq!(expr(b"12+6-4+3"),    IResult::Done(&b""[..], 17));
    assert_eq!(expr(b"1+2*3+4"),     IResult::Done(&b""[..], 11));

    assert_eq!(expr(b"(2)"),         IResult::Done(&b""[..], 2));
    assert_eq!(expr(b"2*(3+4)"),     IResult::Done(&b""[..], 14));
    assert_eq!(expr(b"2*2/(5-1)+3"), IResult::Done(&b""[..], 4));


    named!(alt_tags, alt!(tag!("abcd") | tag!("efgh")));
    assert_eq!(alt_tags(b"abcdxxx"), Done(b"xxx" as &[u8], b"abcd" as &[u8]));
    assert_eq!(alt_tags(b"efghxxx"), Done(b"xxx" as &[u8], b"efgh" as &[u8]));
    //assert_eq!(alt_tags(b"ijklxxx"), Error(1));

    named!( not_space, is_not!( " \t\r\n" ) );

    let r = not_space(&b"abcdefgh\nijklmnopqrstuvwxyz"[..]);
    assert_eq!(r, Done(&b"\nijklmnopqrstuvwxyz"[..], &b"abcdefgh"[..]));
}

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

fn create_weapons(graph_ql_pool: &mut GraphQLPool){
    graph_ql_pool.add(" { Weapon { name: Bow } }");
    graph_ql_pool.add(" { Weapon { name: Spear } }");
    graph_ql_pool.add(" { Weapon { name: Sword } }");
    graph_ql_pool.add(" { Weapon { name: Axe } }");
}
fn create_warriors(graph_ql_pool: &mut GraphQLPool){
    for i in 1..11{
        graph_ql_pool.add((" { Warrior { name: elf".to_string()+&i.to_string()+" race: Elf strength: 50 } }").as_str());
    }
    for i in 1..11{
        graph_ql_pool.add((" { Warrior { name: human".to_string()+&i.to_string()+" race: Human strength: 50 } }").as_str());
    }
    for i in 1..11{
        graph_ql_pool.add((" { Warrior { name: orc".to_string()+&i.to_string()+" race: Orc strength: 50 } }").as_str());
    }
    for i in 1..11{
        graph_ql_pool.add((" { Warrior { name: uruk".to_string()+&i.to_string()+" race: Uruk strength: 50 } }").as_str());
    }
}
fn create_leaders(graph_ql_pool: &mut GraphQLPool){
    graph_ql_pool.add(" { Leader { name: Galadriel wisdom: 50 } }");
    graph_ql_pool.add(" { Leader { name: Aragorn wisdom: 50 } }");
    graph_ql_pool.add(" { Leader { name: Sauron wisdom: 50 } }");
    graph_ql_pool.add(" { Leader { name: Saruman wisdom: 50 } }");
}
fn create_relations(graph_ql_pool: &mut GraphQLPool){
    //alliances
    graph_ql_pool.mysql_query("INSERT INTO Leader_allies_Leader (origin_id, target_id) VALUES (1,2);");
    graph_ql_pool.mysql_query("INSERT INTO Leader_allies_Leader (origin_id, target_id) VALUES (3,4);");

    //leadership
    for elf_id in 1..11{
        graph_ql_pool.mysql_query(&("INSERT INTO Leader_leads_Warrior (origin_id, target_id) VALUES (1,".to_string()+&elf_id.to_string()+");"));
    }
    for human_id in 11..21{
        graph_ql_pool.mysql_query(&("INSERT INTO Leader_leads_Warrior (origin_id, target_id) VALUES (2,".to_string()+&human_id.to_string()+");"));
    }
    for orc_id in 21..31{
        graph_ql_pool.mysql_query(&("INSERT INTO Leader_leads_Warrior (origin_id, target_id) VALUES (3,".to_string()+&orc_id.to_string()+");"));
    }
    for uruk_id in 31..41{
        graph_ql_pool.mysql_query(&("INSERT INTO Leader_leads_Warrior (origin_id, target_id) VALUES (4,".to_string()+&uruk_id.to_string()+");"));
    }

    //weapons
    for elf_id in 1..11{
        graph_ql_pool.mysql_query(&("INSERT INTO Warrior_wears_Weapon (origin_id, target_id) VALUES (".to_string()+&elf_id.to_string()+",1);"));
    }
    for human_id in 11..21{
        graph_ql_pool.mysql_query(&("INSERT INTO Warrior_wears_Weapon (origin_id, target_id) VALUES (".to_string()+&human_id.to_string()+",2);"));
    }
    for orc_id in 21..31{
        graph_ql_pool.mysql_query(&("INSERT INTO Warrior_wears_Weapon (origin_id, target_id) VALUES (".to_string()+&orc_id.to_string()+",3);"));
    }
    for uruk_id in 31..41{
        graph_ql_pool.mysql_query(&("INSERT INTO Warrior_wears_Weapon (origin_id, target_id) VALUES (".to_string()+&uruk_id.to_string()+",4);"));
    }
}

fn test_queries (graph_ql_pool: &mut GraphQLPool){

    let get_weapons_query =
    "{
        Weapon{
            name
        }
    }";

    let mut future = graph_ql_pool.get(get_weapons_query);

    let get_warrior_query =
    "{
        Warrior (id: 8){
            name
            race
            wears {
                name
            }
        }
    }";

    future = graph_ql_pool.get(get_warrior_query);
    future.receive(move |data| {
        let result = match data {
            Ok(res) => res,
            Err(err) => {
                panic!("Error: {:?}",err);
                return;
            },
        };
        assert_eq!(
            result,
            "{\n\t\"data\": {\n\t\t\"Warrior\": {\n\t\t\t\"name\": \'elf8\',\n\t\t\t\"race\": \'Elf\',\n\t\t\t\"wears\": [  \n\t\t\t\t{\n\t\t\t\t\t\"name\": \'Bow\'\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t}\n}\n"
        );
    });

    let get_warriors_query =
    "{
        Warrior (race: \"Human\"){
            name
        }
    }";

    future = graph_ql_pool.get(get_warriors_query);
    future.receive(move |data| {
        let result = match data {
            Ok(res) => res,
            Err(err) => {
                panic!("Error: {:?}",err);
                return;
            },
        };
        assert_eq!(
            result,
            "{\n\t\"data\": {\n\t\t\"Warrior\": {\n\t\t\t\"name\": \'human1\'\n\t\t},\n\t\t\"Warrior\": {\n\t\t\t\"name\": \'human2\'\n\t\t},\n\t\t\"Warrior\": {\n\t\t\t\"name\": \'human3\'\n\t\t},\n\t\t\"Warrior\": {\n\t\t\t\"name\": \'human4\'\n\t\t},\n\t\t\"Warrior\": {\n\t\t\t\"name\": \'human5\'\n\t\t},\n\t\t\"Warrior\": {\n\t\t\t\"name\": \'human6\'\n\t\t},\n\t\t\"Warrior\": {\n\t\t\t\"name\": \'human7\'\n\t\t},\n\t\t\"Warrior\": {\n\t\t\t\"name\": \'human8\'\n\t\t},\n\t\t\"Warrior\": {\n\t\t\t\"name\": \'human9\'\n\t\t},\n\t\t\"Warrior\": {\n\t\t\t\"name\": \'human10\'\n\t\t}\n\t}\n}\n"
        );
    });

    let get_leader_and_his_warriors_query =
    "{
        Leader (id: 1){
            name
            wisdom
            leads {
                name
            }
        }
    }";

    future = graph_ql_pool.get(get_leader_and_his_warriors_query);
    future.receive(move |data| {
        let result = match data {
            Ok(res) => res,
            Err(err) => {
                panic!("Error: {:?}",err);
                return;
            },
        };
        assert_eq!(
            result,
            "{\n\t\"data\": {\n\t\t\"Leader\": {\n\t\t\t\"name\": \'Galadriel\',\n\t\t\t\"wisdom\": 50,\n\t\t\t\"leads\": [  \n\t\t\t\t{\n\t\t\t\t\t\"name\": \'elf1\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'elf2\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'elf3\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'elf4\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'elf5\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'elf6\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'elf7\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'elf8\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'elf9\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'elf10\'\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t}\n}\n"
        );
    });

    let update_leader_query = "{
        Leader (id:3) {
            wisdom: 75
        }
    }";

    graph_ql_pool.update(update_leader_query);

    let get_Sauron_query =
    "{
        Leader (id: 3){
            name
            wisdom
        }
    }";

    future = graph_ql_pool.get(get_Sauron_query);
    future.receive(move |data| {
        let result = match data {
            Ok(res) => res,
            Err(err) => {
                panic!("Error: {:?}",err);
                return;
            },
        };
        assert_eq!(
            result,
            "{\n\t\"data\": {\n\t\t\"Leader\": {\n\t\t\t\"name\": \'Sauron\',\n\t\t\t\"wisdom\": 75\n\t\t}\n\t}\n}\n"
        );
    });

    let delete_warrior_query = "{
        Warrior (id:22)
    }";

    graph_ql_pool.delete(delete_warrior_query);

    let get_leader_and_his_warriors_query =
    "{
        Leader (id: 3){
            name
            wisdom
            leads {
                name
            }
        }
    }";

    future = graph_ql_pool.get(get_leader_and_his_warriors_query);
    future.receive(move |data| {
        let result = match data {
            Ok(res) => res,
            Err(err) => {
                panic!("Error: {:?}",err);
                return;
            },
        };
        assert_eq!(
            result,
            "{\n\t\"data\": {\n\t\t\"Leader\": {\n\t\t\t\"name\": \'Sauron\',\n\t\t\t\"wisdom\": 75,\n\t\t\t\"leads\": [  \n\t\t\t\t{\n\t\t\t\t\t\"name\": \'orc1\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'orc3\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'orc4\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'orc5\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'orc6\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'orc7\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'orc8\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'orc9\'\n\t\t\t\t},\n\t\t\t\t{\n\t\t\t\t\t\"name\": \'orc10\'\n\t\t\t\t}\n\t\t\t]\n\t\t}\n\t}\n}\n"
        );
    });

    let delete_weapons_query = "{
        Weapon
    }";

    graph_ql_pool.delete(delete_weapons_query);

    let get_warriors_query =
    "{
        Warrior (id: 11){
            name
            strength
            wears {
                name
            }
        }
    }";

    future = graph_ql_pool.get(get_warriors_query);
    future.receive(move |data| {
        let result = match data {
            Ok(res) => res,
            Err(err) => {
                panic!("Error: {:?}",err);
                return;
            },
        };
        assert_eq!(
            result,
            "{\n\t\"data\": {\n\t\t\"Warrior\": {\n\t\t\t\"name\": \'human1\',\n\t\t\t\"strength\": 50,\n\t\t\t\"wears\": [ \n\t\t\t]\n\t\t}\n\t}\n}\n"
        );
    });
}

#[test]
fn test_db_creation_and_crud () {
    let mysql_connection = "mysql://".to_string()+DB_USER+":"+DB_PASSWORD+"@"+HOST+":"+PORT;
    let mut graph_ql_pool = GraphQLPool::new(
        mysql_connection.as_str(),
        DB_NAME,
        &(FILE_LOCATION.to_string()+"/"+FILE_NAME)
    );

    create_weapons(&mut graph_ql_pool);
    create_warriors(&mut graph_ql_pool);
    create_leaders(&mut graph_ql_pool);
    create_relations(&mut graph_ql_pool);
    thread::sleep_ms(10000);

    test_queries(&mut graph_ql_pool);
    thread::sleep_ms(10000);
}
