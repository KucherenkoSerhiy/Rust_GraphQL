use nom::{not_line_ending, space, alphanumeric, multispace};
use nom::{IResult,digit};
use nom::IResult::*;

use std::str;
use std::vec::Vec;
use std::option::Option;

use def::*;

named!(parse_param <&[u8],(String,String)>,
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
    ||{(key.to_string(), val.to_string())}
  )
);

named!(parse_field <&[u8],(String,String)>,
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
    ||{(key.to_string(), val.to_string())}
  )
);

named!(parse_object_attributes <&[u8], Vec<(String,String)> >,
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

named!(parse_object <(String, Vec<(String, String)>)>,
    chain!(
        tag!("type")                         ~
        space                                ~
        name: map_res!(alphanumeric, str::from_utf8) ~
        multispace?                          ~
        attrs: parse_object_attributes,
        || {(name.to_string(), attrs)}
    )
);

named! (pub parse_all_objects <&[u8], Vec <(String, Vec<(String, String)>)> >,
    many0!(chain!(
        multispace?                          ~
        result: parse_object                 ~
        multispace?,
        ||{result}
    ))
);

/*
WILL BE USED TO PARSE EVERYTHING (Tables, Relations, Enums) IN THE FILE
named! (pub parse_file <&[u8], Vec <(String, Vec<(String, String)>)> >,
    chain!(
        tbl: parse_all_objects,
        ||{tbl}
    )
);
*/




/*
mutation {
  delete  (id: 1234) {
    status
  }
}
*/
/*
named!(parse_query_type <&[u8], Option<&[u8]> >,
    chain!(
        operation: tag!("mutation")? ~
        multispace?,
        ||{operation}
    )
);

named!(parse_mutation_type <&[u8], Option<&[u8]> >,
    chain!(
        mutation_type: alt!(tag!("abcd") | tag!("efgh")) ~
        space?,
        ||{mutation_type}
    )
);
*/


/*
{
        user (id:1) {
            name
            friends {
              id
              name
            }
        }
    }
*/

/*
map_res
expected `std::string::String`,
found `std::result::Result<_, _>`

map
expected `String`,
found `std::result::Result<String, std::str::Utf8Error>`
*/
named! (parse_select_object <&[u8], (String, Option<(String, String)>, Vec<String>)>,
    chain!(
        multispace?                      ~
        object: map_res!(
                    alphanumeric,
                    str::from_utf8
                )                        ~
        space?                           ~
        params: delimited!(
            char!('('),
            parse_param,
            char!(')')
        )?                               ~
        space?                           ~
        attributes: delimited!(
            char!('{'),
            many0!(chain!(
                multispace?              ~
                attr: map_res!(alphanumeric, str::from_utf8)   ~
                    //parse_select_object //recursivitat

                multispace?,
                ||{attr.to_string()}
            )),
            char!('}')
        )                                ~
        multispace?,
        ||{(object.to_string(), params, attributes)}
    )
);

named! (pub parse_select_query <&[u8], (String, Option<(String, String)>, Vec<String>)>,
    chain!(
        multispace?                              ~
        res: delimited!(
            char!('{'),
            parse_select_object,
            char!('}')
        )                                        ~
        multispace?,
        ||{res}
    )
);


named! (pub parse_insert_query <&[u8], (String, Vec<(String, String)> )>,
    chain!(
        multispace?                              ~
        res: delimited!(
            char!('{'),
            chain!(
                multispace?                      ~
                object: map_res!(
                            alphanumeric,
                            str::from_utf8
                        )                        ~
                multispace?                      ~
                attributes: delimited!(
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
                ||{(object.to_string(), attributes)}
            ),
            char!('}')
        )                                        ~
        multispace?,
        ||{res}
    )
);

named! (pub parse_update_query <&[u8], (String, (String, String), Vec<(String, String)> )>,
    chain!(
        multispace?                              ~
        res: delimited!(
            char!('{'),
            chain!(
                multispace?                      ~
                object: map_res!(
                            alphanumeric,
                            str::from_utf8
                        )                        ~
                space?                           ~
                params: delimited!(
                    char!('('),
                    parse_param,
                    char!(')')
                )                                ~
                space?                           ~
                mutations: delimited!(
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
                ||{(object.to_string(), params, mutations)}
            ),
            char!('}')
        )                                        ~
        multispace?,
        ||{res}
    )
);


named! (pub parse_delete_query <&[u8], (String, Option<(String, String)> )>,
    chain!(
        multispace?                              ~
        res: delimited!(
            char!('{'),
            chain!(
                multispace?                      ~
                object: map_res!(
                            alphanumeric,
                            str::from_utf8
                        )                        ~
                multispace?                      ~
                attributes: delimited!(
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
                ||{(object.to_string(), attributes)}
            ),
            char!('}')
        )                                        ~
        multispace?,
        ||{res}
    )
);


#[test]
fn test_internal_parser_functions(){
    assert_eq!(
        parse_field(&b"id : String
                    "[..]),
        //`nom::IResult<&[u8], rust_sql::def::db_column<'_>>`
        IResult::Done(&b""[..], {("id".to_string(), "String".to_string())})
    );


    assert_eq!(
        parse_field(&b"id:'1'
                    "[..]),
        //`nom::IResult<&[u8], rust_sql::def::db_column<'_>>`
        IResult::Done(&b""[..], {("id".to_string(), "\'1\'".to_string())})
    );

    let cols = IResult::Done(&b""[..], vec![
        {("id".to_string(), "String".to_string())},
        {("name".to_string(), "String".to_string())},
        {("homePlanet".to_string(), "String".to_string())},
        {("list".to_string(), "[String]".to_string())}
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

    let result = IResult::Done(
        &b""[..],
        ("Human".to_string(),
         vec![
            {("id".to_string(), "String".to_string())},
            {("name".to_string(), "String".to_string())},
            {("homePlanet".to_string(), "String".to_string())}
        ])
    );
    assert_eq!(
        parse_object(
                &b"type Human{
                    id: String
                    name: String
                    homePlanet: String
                }"[..]
        ),
        result
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
    let get_query_data = IResult::Done(&b""[..], {("user".to_string(), Some({("id".to_string(), "1".to_string())}), vec![{"name".to_string()}])});
    assert_eq!(parse_select_query(get_query), get_query_data);

    let get_query =
    &b"{
        user (id:1) {
            name
            friends {
              id
              name
            }
        }
    }"[..];
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
    let mut insert_query_data = IResult::Done(&b""[..], {("Human".to_string(), vec![{("id".to_string(), "1".to_string())}, {("name".to_string(), "Luke".to_string())}, {("homePlanet".to_string(), "Char".to_string())}])});
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
    insert_query_data = IResult::Done(&b""[..], {("Droid".to_string(), vec![{("id".to_string(), "1".to_string())}, {("name".to_string(), "R2D2".to_string())}, {("age".to_string(), "3".to_string())}, {("primaryFunction".to_string(), "Mechanic".to_string())}, {("created".to_string(), "STR_TO_DATE('1-01-2012', '%d-%m-%Y')".to_string())}])});
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
    let update_query_data = IResult::Done(&b""[..], {("Droid".to_string(), ("id".to_string(), "1".to_string()), vec![{("age".to_string(), "4".to_string())}])});
    assert_eq!(parse_update_query(update_query), update_query_data);
}

#[test]
fn test_delete_parser_function(){
    let mut delete_query =
    &b"{
        user (id:1)
    }"[..];
    let mut delete_query_data = IResult::Done(&b""[..], {("user".to_string(), Some(("id".to_string(), "1".to_string())))});
    assert_eq!(parse_delete_query(delete_query), delete_query_data);

    delete_query =
    &b"{
        user
    }"[..];
    delete_query_data = IResult::Done(&b""[..], {("user".to_string(), None)});
    assert_eq!(parse_delete_query(delete_query), delete_query_data);
}