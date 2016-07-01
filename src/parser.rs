use nom::{not_line_ending, space, alphanumeric, multispace};

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
           take_until_either!(" \n"),
           str::from_utf8
         )                                 ~
         space?                            ~
         not_line_ending                   ~
         multispace?                       ,
    ||{(key, val)}
  )
);

named!(pub attrs <&[u8], Vec<(&str,&str)> >,
    delimited!(
        char!('{'),
        //take_until_either!(" \n"),
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

named! (pub database < Vec <(&str, Vec<(&str, &str)>)> >,
    many0!(chain!(
        multispace?                          ~
        res: table                           ~
        multispace?,
        ||{res}
    ))
);