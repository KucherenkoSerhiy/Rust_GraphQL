use nom::{IResult,not_line_ending, space, alphanumeric, multispace, digit};

use std::str;
use std::collections::HashMap;

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

//alfanumeric
named!(not_space, is_not!( " \t\r\n" ) );

named!(pub key_value    <&[u8],(&str,&str)>,
  chain!(
    key: map_res!(alphanumeric, str::from_utf8) ~
         space?                            ~
         tag!("=")                         ~
         space?                            ~
    val: map_res!(
           take_until_either!("\n;"),
           str::from_utf8
         )                                 ~
         space?                            ~
         chain!(
           not_line_ending  ,
           ||{}
         ) ?                               ~
         multispace?                       ,
    ||{(key, val)}
  )
);