use nom::{not_line_ending, space, alphanumeric, multispace};
use nom::{IResult,digit};
use nom::IResult::*;

use std::str;
use std::vec::Vec;

use def::*;

/*
GLOSSARY
word = constant word
%variable% = variable word
'c' = one letter c
*/


/*
INPUT
type ' ' %table_name% '{'
    %column_name% ':' %column_type%
    %column_name% ':' %column_type%
    ...
'}'

OUTPUT
    r"CREATE TEMPORARY TABLE %table_name% (
        %column_name%  %column_type%,
        %column_name%  %column_type%,
    )"
*/

named!(pub key_value    <&[u8],(&str,&str)>,
  chain!(
    key: map_res!(alphanumeric, str::from_utf8) ~
         space?                            ~
         tag!(":")                         ~
         space?                            ~
    val: map_res!(
           take_until_either!(" \n)}"),
           str::from_utf8
         )                                 ~
         space?                            ~
         multispace?                       ,
    ||{(key, val)}
  )
);

named!(pub attrs <&[u8], Vec<(&str,&str)> >,
    delimited!(
        char!('{'),
        many0!(chain!(
            multispace?                      ~
            result: key_value,
            ||{result}
        )),
        char!('}')
    )
);

named!(pub table <(&str, Vec<(&str, &str)>)>,
    chain!(
        tag!("type")                         ~
        space                                ~
        name: map_res!(alphanumeric, str::from_utf8) ~
        multispace?                          ~
        cols: attrs,
        || {(name, cols)}
    )
);

named! (pub tables <&[u8], Vec <(&str, Vec<(&str, &str)>)> >,
    many0!(chain!(
        multispace?                          ~
        res: table                           ~
        multispace?,
        ||{res}
    ))
);

named! (pub database <&[u8], Vec <(&str, Vec<(&str, &str)>)> >,
    chain!(
        tbl: tables,
        ||{tbl}
    )
);


named! (pub sql_select <&[u8], (&str, (&str, &str), Vec<&str>)>,
    delimited!(
        char!('{'),
        chain!(
            multispace?                      ~
            entity: map_res!(
                        alphanumeric,
                        str::from_utf8
                    )                        ~
            space?                           ~
            options: delimited!(
                char!('('),
                key_value,
                char!(')')
            )                                ~
            space?                           ~
            attributes: delimited!(
                char!('{'),
                many0!(chain!(
                    multispace?              ~
                    res: map_res!(
                        alphanumeric,
                        str::from_utf8
                    )                        ~
                    multispace?,
                    ||{res}
                )),
                char!('}')
            )                                ~
            multispace?,
            ||{(entity, options, attributes)}
        ),
        char!('}')
    )
);


named! (pub sql_insert <&[u8], (&str, Vec<(&str, &str)> )>,
    delimited!(
        char!('{'),
        chain!(
            multispace?                      ~
            entity: map_res!(
                        alphanumeric,
                        str::from_utf8
                    )                        ~
            multispace?                      ~
            attributes: delimited!(
                char!('{'),
                many0!(chain!(
                    multispace?              ~
                    res: key_value           ~
                    multispace?,
                    ||{res}
                )),
                char!('}')
            )                                ~
            multispace?,
            ||{(entity, attributes)}
        ),
        char!('}')
    )
);
/*
named! (pub sql_update <&[u8], (&str, Vec<(&str, &str)> )>,
    delimited!(
        char!('{'),
        chain!(

        ),
        char!('}')
    )
);

named! (pub sql_delete <&[u8], (&str, Vec<(&str, &str)> )>,
    delimited!(
        char!('{'),
        chain!(

        ),
        char!('}')
    )
);
*/
#[test]
fn test_parser_functions(){
    assert_eq!(
        key_value(&b"id : String
                    "[..]),
        //`nom::IResult<&[u8], rust_sql::def::db_column<'_>>`
        IResult::Done(&b""[..], {("id", "String")})
    );


    assert_eq!(
        key_value(&b"id:'1'
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
        attrs(&b"{
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
        table(&b"type Human{
                    id: String
                    name: String
                    homePlanet: String
                 }"[..]),
        result_table
    );
}

#[test]
fn test_get_object(){
    let get_query =
    &b"{
        user (id:1) {
            name
        }
    }"[..];
    let get_query_data = IResult::Done(&b""[..], {("user", ("id", "1"), vec![{"name"}])});
    assert_eq!(sql_select(get_query), get_query_data);
}

#[test]
fn test_insert_object(){
    let insert_query =
    &b"{
        Human {
            id: 1
            name: Luke
            homePlanet: Char
        }
    }"[..];
    let insert_query_data = IResult::Done(&b""[..], {("Human", vec![{("id", "1")}, {("name", "Luke")}, {("homePlanet", "Char")}])});
    assert_eq!(sql_insert(insert_query), insert_query_data);
}