use nom::{not_line_ending, space, alphanumeric, multispace};
use nom::{IResult,digit};
use nom::IResult::*;

use std::str;
use std::vec::Vec;
use std::option::Option;

use def::*;

named!(parse_param <&[u8],(&str,&str)>,
  chain!(
    key: map_res!(alphanumeric, str::from_utf8) ~
         space?                            ~
         tag!(":")                         ~
         space?                            ~
    val: map_res!(
           take_until_either!(")"),
           str::from_utf8
         )                                 ~
         space?                            ~
         multispace?                       ,
    ||{(key, val)}
  )
);

named!(parse_field <&[u8],(&str,&str)>,
  chain!(
    key: map_res!(alphanumeric, str::from_utf8) ~
         space?                            ~
         tag!(":")                         ~
         space?                            ~
    val: map_res!(
           take_until_either!("\n}"),
           str::from_utf8
         )                                 ~
         space?                            ~
         multispace?                       ,
    ||{(key, val)}
  )
);

named!(parse_object_attributes <&[u8], Vec<(&str,&str)> >,
    delimited!(
        char!('{'),
        many0!(chain!(
            multispace?                      ~
            result: parse_field,
            ||{result}
        )),
        char!('}')
    )
);

named!(parse_object <(&str, Vec<(&str, &str)>)>,
    chain!(
        tag!("type")                         ~
        space                                ~
        name: map_res!(alphanumeric, str::from_utf8) ~
        multispace?                          ~
        attrs: parse_object_attributes,
        || {(name, attrs)}
    )
);

named! (pub parse_all_objects <&[u8], Vec <(&str, Vec<(&str, &str)>)> >,
    many0!(chain!(
        multispace?                          ~
        result: parse_object                 ~
        multispace?,
        ||{result}
    ))
);

/*
WILL BE USED TO PARSE EVERYTHING (Tables, Relations, Enums) IN THE FILE
named! (pub parse_file <&[u8], Vec <(&str, Vec<(&str, &str)>)> >,
    chain!(
        tbl: parse_all_objects,
        ||{tbl}
    )
);
*/

named! (pub parse_select_query <&[u8], (&str, (&str, &str), Vec<&str>)>,
    delimited!(
        char!('{'),
        chain!(
            multispace?                      ~
            table_name: map_res!(
                        alphanumeric,
                        str::from_utf8
                    )                        ~
            space?                           ~
            table_params: delimited!(
                char!('('),
                parse_param,
                char!(')')
            )                                ~
            space?                           ~
            table_cols: delimited!(
                char!('{'),
                many0!(chain!(
                    multispace?              ~
                    result: map_res!(
                        alphanumeric,
                        str::from_utf8
                    )                        ~
                    multispace?,
                    ||{result}
                )),
                char!('}')
            )                                ~
            multispace?,
            ||{(table_name, table_params, table_cols)}
        ),
        char!('}')
    )
);


named! (pub parse_insert_query <&[u8], (&str, Vec<(&str, &str)> )>,
    delimited!(
        char!('{'),
        chain!(
            multispace?                      ~
            table_name: map_res!(
                        alphanumeric,
                        str::from_utf8
                    )                        ~
            multispace?                      ~
            table_cols: delimited!(
                char!('{'),
                many0!(chain!(
                    multispace?              ~
                    res: parse_field         ~
                    multispace?,
                    ||{res}
                )),
                char!('}')
            )                                ~
            multispace?,
            ||{(table_name, table_cols)}
        ),
        char!('}')
    )
);

named! (pub parse_update_query <&[u8], (&str, (&str, &str), Vec<(&str, &str)> )>,
    delimited!(
        char!('{'),
        chain!(
            multispace?                      ~
            table_name: map_res!(
                        alphanumeric,
                        str::from_utf8
                    )                        ~
            space?                           ~
            table_params: delimited!(
                char!('('),
                parse_param,
                char!(')')
            )                                ~
            space?                           ~
            table_mutations: delimited!(
                char!('{'),
                many0!(chain!(
                    multispace?              ~
                    res: parse_field         ~
                    multispace?,
                    ||{res}
                )),
                char!('}')
            )                                ~
            multispace?,
            ||{(table_name, table_params, table_mutations)}
        ),
        char!('}')
    )
);


named! (pub parse_delete_query <&[u8], (&str, Option<(&str, &str)> )>,
    delimited!(
        char!('{'),
        chain!(
            multispace?                      ~
            table_name: map_res!(
                        alphanumeric,
                        str::from_utf8
                    )                        ~
            multispace?                      ~
            table_cols: delimited!(
                char!('('),
                chain!(
                    multispace?              ~
                    res: parse_param         ~
                    multispace?,
                    ||{res}
                ),
                char!(')')
            )?                               ~
            multispace?,
            ||{(table_name, table_cols)}
        ),
        char!('}')
    )
);


#[test]
fn test_internal_parser_functions(){
    assert_eq!(
        parse_field(&b"id : String
                    "[..]),
        //`nom::IResult<&[u8], rust_sql::def::db_column<'_>>`
        IResult::Done(&b""[..], {("id", "String")})
    );


    assert_eq!(
        parse_field(&b"id:'1'
                    "[..]),
        //`nom::IResult<&[u8], rust_sql::def::db_column<'_>>`
        IResult::Done(&b""[..], {("id", "\'1\'")})
    );

    let cols = IResult::Done(&b""[..], vec![
        {("id", "String")},
        {("name", "String")},
        {("homePlanet", "String")},
        {("list", "[String]")}
    ]);
    assert_eq!(
        parse_object_attributes(&b"{
                    id: String
                    name: String
                    homePlanet: String
                    list: [String]
                 }"[..]),
        cols
    );

    let result_table = IResult::Done(
        &b""[..],
        (&"Human"[..],
         vec![
            {("id", "String")},
            {("name", "String")},
            {("homePlanet", "String")}
        ])
    );
    assert_eq!(
        parse_object(&b"type Human{
                    id: String
                    name: String
                    homePlanet: String
                 }"[..]),
        result_table
    );
}

#[test]
fn test_get_parser_function(){
    let get_query =
    &b"{
        user (id:1) {
            name
        }
    }"[..];
    let get_query_data = IResult::Done(&b""[..], {("user", ("id", "1"), vec![{"name"}])});
    assert_eq!(parse_select_query(get_query), get_query_data);
}

#[test]
fn test_insert_parser_function(){
    let mut insert_query =
    &b"{
        Human {
            id: 1
            name: Luke
            homePlanet: Char
        }
    }"[..];
    let mut insert_query_data = IResult::Done(&b""[..], {("Human", vec![{("id", "1")}, {("name", "Luke")}, {("homePlanet", "Char")}])});
    assert_eq!(parse_insert_query(insert_query), insert_query_data);

    insert_query =
    &b"{
        Droid {
            id: 1
            name: R2D2
            age: 3
            primaryFunction: Mechanic
            created: STR_TO_DATE('1-01-2012', '%d-%m-%Y')
        }
    }"[..];
    insert_query_data = IResult::Done(&b""[..], {("Droid", vec![{("id", "1")}, {("name", "R2D2")}, {("age", "3")}, {("primaryFunction", "Mechanic")}, {("created", "STR_TO_DATE('1-01-2012', '%d-%m-%Y')")}])});
    assert_eq!(parse_insert_query(insert_query), insert_query_data);
}

#[test]
fn test_update_parser_function(){
    let update_query =
    &b"{
        Droid (id:1) {
            age: 4
        }
    }"[..];
    let update_query_data = IResult::Done(&b""[..], {("Droid", ("id", "1"), vec![{("age", "4")}])});
    assert_eq!(parse_update_query(update_query), update_query_data);
}

#[test]
fn test_delete_parser_function(){
    let mut delete_query =
    &b"{
        user (id:1)
    }"[..];
    let mut delete_query_data = IResult::Done(&b""[..], {("user", Some(("id", "1")))});
    assert_eq!(parse_delete_query(delete_query), delete_query_data);

    delete_query =
    &b"{
        user
    }"[..];
    delete_query_data = IResult::Done(&b""[..], {("user", None)});
    assert_eq!(parse_delete_query(delete_query), delete_query_data);
}