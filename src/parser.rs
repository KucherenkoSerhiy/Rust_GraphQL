use nom::{not_line_ending, space, alphanumeric, multispace};
use nom::{IResult,digit};
use nom::IResult::*;

use std::str;
use std::vec::Vec;
use std::option::Option;

use def::*;

named!(parse_param <&[u8],(String,String)>,
  chain!(
    key: map_res!(
            alt!(
                alphanumeric |
                delimited!(
                    char!('\"'),
                    alphanumeric,
                    char!('\"')
                )
            ),
            str::from_utf8
         )                                 ~
         space?                            ~
         tag!(":")                         ~
         space?                            ~
    val: map_res!(
            alt!(
                alphanumeric |
                delimited!(
                    char!('\"'),
                    alphanumeric,
                    char!('\"')
                )
            ),
            str::from_utf8
         )                                 ~
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
        phone
    }
}
{
    Human (id:"1"){
        name
        homePlanet
    }
}
*/
named! (parse_query_object <&[u8], Query_Object>,
    chain!(
        multispace?                      ~
        object: map_res!(
                    alphanumeric,
                    str::from_utf8
                )                        ~
        space?                           ~
        params: delimited!(
            char!('('),
            many0!(chain!(
                multispace?              ~
                param: parse_param       ~
                multispace?,
                ||{param}
            )),
            char!(')')
        )?                               ~
        space?                           ~
        attributes: delimited!(
            char!('{'),
            many0!(chain!(
                multispace?              ~
                attr: parse_query_object ~ //recursion
                multispace?,
                ||{attr}
            )),
            char!('}')
        )?                                ~
        multispace?,
        ||{Query_Object{name: object.to_string(), params: params, attrs: attributes}}
    )
);

named! (pub parse_query <&[u8], Query_Object>,
    chain!(
        multispace?                              ~
        res: delimited!(
            char!('{'),
            parse_query_object,
            char!('}')
        )                                        ~
        multispace?,
        ||{res}
    )
);

named! (parse_mutation_object <&[u8], Mutation_Object>,
    chain!(
        multispace?                      ~
        name: map_res!(
            alphanumeric,
            str::from_utf8
        )                                ~
        space?                           ~
        value: chain! (
            tag!(":")                    ~
            space?                       ~
            res: map_res!(
                alt!(
                    alphanumeric |
                    delimited!(
                        char!('\"'),
                        alphanumeric,
                        char!('\"')
                    )
                ),
                str::from_utf8
            ),
            ||{res.to_string()}
        )?                               ~
        params: delimited!(
            char!('('),
            many0!(chain!(
                multispace?              ~
                res: parse_param         ~
                multispace?,
                ||{res}
            )),
            char!(')')
        )?                               ~
        space?                           ~
        attributes: delimited!(
            char!('{'),
            many0!(chain!(
                multispace?              ~
                res: parse_mutation_object ~ //recursion
                multispace?,
                ||{res}
            )),
            char!('}')
        )?                               ~
        multispace?,
        ||{Mutation_Object{name: name.to_string(), value: value, params: params, attrs: attributes}}
    )
);

named! (pub parse_mutation_query <&[u8], Mutation_Object>,
    chain!(
        multispace?                              ~
        res: delimited!(
            char!('{'),
            parse_mutation_object,
            char!('}')
        )                                        ~
        multispace?,
        ||{res}
    )
);


#[test]
fn test_internal_parser_functions(){
    assert_eq!(
        parse_param(&b"id: \"1\"
                    "[..]),
        IResult::Done(&b""[..], {("id".to_string(), "1".to_string())})
    );

    assert_eq!(
        parse_field(&b"id : String
                    "[..]),
        IResult::Done(&b""[..], {("id".to_string(), "String".to_string())})
    );


    assert_eq!(
        parse_field(&b"id:'1'
                    "[..]),
        IResult::Done(&b""[..], {("id".to_string(), "\'1\'".to_string())})
    );

    assert_eq!(
        parse_field(&b"id:[Object]
                    "[..]),
        IResult::Done(&b""[..], {("id".to_string(), "[Object]".to_string())})
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
            phone
        }
    }"[..];
    let get_query_data = IResult::Done(&b""[..],
        {Query_Object {
            name:"user".to_string(),
            params: Some(vec![{("id".to_string(), "1".to_string())}]),
            attrs: Some(vec![
                Query_Object {
                    name: "name".to_string(),
                    params: None,
                    attrs: None
                },
                Query_Object {
                    name: "phone".to_string(),
                    params: None,
                    attrs: None
                }
            ])
        }}
    );

    assert_eq!(parse_query(get_query), get_query_data);

    let get_query =
    &b"{
        user (id:\"1\") {
            name
            friends {
              id
              name
            }
        }
    }"[..];

    let get_query_data = IResult::Done(&b""[..],
                                       {Query_Object {
                                           name:"user".to_string(),
                                           params: Some(vec![{("id".to_string(), "1".to_string())}]),
                                           attrs: Some(vec![
                                               Query_Object {
                                                    name: "name".to_string(),
                                                    params: None,
                                                    attrs: None
                                               },
                                               Query_Object {
                                                    name: "friends".to_string(),
                                                    params: None,
                                                    attrs: Some(vec![
                                                        Query_Object {
                                                            name: "id".to_string(),
                                                            params: None,
                                                            attrs: None
                                                        },
                                                        Query_Object {
                                                            name: "name".to_string(),
                                                            params: None,
                                                            attrs: None
                                                        }
                                                    ])
                                                }
                                            ])
                                       }}
    );
    assert_eq!(parse_query(get_query), get_query_data);
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
    //let mut insert_query_data = IResult::Done(&b""[..], {("Human".to_string(), vec![{("id".to_string(), "1".to_string())}, {("name".to_string(), "Luke".to_string())}, {("homePlanet".to_string(), "Char".to_string())}])});
    let mut insert_query_data = IResult::Done(&b""[..], {Mutation_Object {
        name: "Human".to_string(),
        value: None,
        params: None,
        attrs: Some(vec![
                        Mutation_Object {
                            name: "id".to_string(),
                            value: Some("1".to_string()),
                            params: None,
                            attrs: None
                        },
                        Mutation_Object {
                            name: "name".to_string(),
                            value: Some("Luke".to_string()),
                            params: None,
                            attrs: None
                        },
                        Mutation_Object {
                            name: "homePlanet".to_string(),
                            value: Some("Char".to_string()),
                            params: None,
                            attrs: None
                        }
                    ])
    }});
    assert_eq!(parse_mutation_query(insert_query), insert_query_data);
    insert_query =
    &b"{
        Droid {
            id: 1
            name: \"R2D2\"
            age: 3
            primaryFunction: \"Mechanic\"
        }
    }"[..];
    insert_query_data = IResult::Done(&b""[..], {Mutation_Object {
        name: "Droid".to_string(),
        value: None,
        params: None,
        attrs: Some(vec![
                        Mutation_Object {
                            name: "id".to_string(),
                            value: Some("1".to_string()),
                            params: None,
                            attrs: None
                        },
                        Mutation_Object {
                            name: "name".to_string(),
                            value: Some("R2D2".to_string()),
                            params: None,
                            attrs: None
                        },
                        Mutation_Object {
                            name: "age".to_string(),
                            value: Some("3".to_string()),
                            params: None,
                            attrs: None
                        },
                        Mutation_Object {
                            name: "primaryFunction".to_string(),
                            value: Some("Mechanic".to_string()),
                            params: None,
                            attrs: None
                        }
                    ])
    }});
    assert_eq!(parse_mutation_query(insert_query), insert_query_data);

    insert_query =
    &b"{
        Human {
            id: 1
            name: Luke
            friends {
                Human {
                    id: 2
                    name: Leia
                }
                Human {
                    id: 3
                    name: Han
                }
            }
        }
    }"[..];
    insert_query_data = IResult::Done(&b""[..], {Mutation_Object {
        name: "Human".to_string(),
        value: None,
        params: None,
        attrs: Some(vec![
                        Mutation_Object {
                            name: "id".to_string(),
                            value: Some("1".to_string()),
                            params: None,
                            attrs: None
                        },
                        Mutation_Object {
                            name: "name".to_string(),
                            value: Some("Luke".to_string()),
                            params: None,
                            attrs: None
                        },
                        Mutation_Object {
                            name: "friends".to_string(),
                            value: None,
                            params: None,
                            attrs: Some(vec![
                                Mutation_Object {
                                    name: "Human".to_string(),
                                    value: None,
                                    params: None,
                                    attrs: Some(vec![
                                        Mutation_Object {
                                            name: "id".to_string(),
                                            value: Some("2".to_string()),
                                            params: None,
                                            attrs: None
                                        },
                                        Mutation_Object {
                                            name: "name".to_string(),
                                            value: Some("Leia".to_string()),
                                            params: None,
                                            attrs: None
                                        }
                                    ])
                                },
                                Mutation_Object {
                                    name: "Human".to_string(),
                                    value: None,
                                    params: None,
                                    attrs: Some(vec![
                                        Mutation_Object {
                                            name: "id".to_string(),
                                            value: Some("3".to_string()),
                                            params: None,
                                            attrs: None
                                        },
                                        Mutation_Object {
                                            name: "name".to_string(),
                                            value: Some("Han".to_string()),
                                            params: None,
                                            attrs: None
                                        }
                                    ])
                                }
                            ])
                        }
                    ])
    }});
    assert_eq!(parse_mutation_query(insert_query), insert_query_data);

}

#[test]
fn test_update_parser_function(){
    let update_query =
    &b"{
        Droid (id:1) {
            age: 4
        }
    }"[..];
    let update_query_data = IResult::Done(&b""[..], {Mutation_Object {
            name: "Droid".to_string(),
            value: None,
            params: Some(vec![{("id".to_string(), ("1".to_string()))}]),
            attrs: Some(vec![
                Mutation_Object {
                    name: "age".to_string(),
                    value: Some("4".to_string()),
                    params: None,
                    attrs: None
                }
            ])
    }});
    assert_eq!(parse_mutation_query(update_query), update_query_data);
}

#[test]
fn test_delete_parser_function(){
    let mut delete_query =
    &b"{
        user (id:1)
    }"[..];
    let mut delete_query_data = IResult::Done(&b""[..], {Mutation_Object {
        name: "user".to_string(),
        value: None,
        params: Some(vec![{("id".to_string(), ("1".to_string()))}]),
        attrs: None
    }});
    assert_eq!(parse_mutation_query(delete_query), delete_query_data);

    delete_query =
    &b"{
        user
    }"[..];
    delete_query_data = IResult::Done(&b""[..], {Mutation_Object {
        name: "user".to_string(),
        value: None,
        params: None,
        attrs: None
    }});
    assert_eq!(parse_mutation_query(delete_query), delete_query_data);
}