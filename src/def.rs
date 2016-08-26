use std::vec::Vec;
use std::str;

use mysql;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DbColumn {
    pub name: String,
    pub db_type: String
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DbTable {
    pub name: String,
    pub columns: Vec<DbColumn>
}

#[derive(Clone)]
pub struct TargetPool {
    pub pool: mysql::Pool,
    pub database: Vec<DbTable>,
    pub working_database_name: String
}

#[derive(Debug, PartialEq, Eq)]
pub struct Query_Object {
    pub name: String,
    pub params: Option<Vec <(String, String)> >,
    pub attrs: Option<Vec <Query_Object> >
}

#[derive(Debug, PartialEq, Eq)]
pub struct Mutation_Object {
    pub name: String,
    pub value: Option <String>,
    pub params: Option<Vec <(String, String)> >,
    pub attrs: Option<Vec <Mutation_Object> >
}

#[derive(Clone)]
pub struct Relation {
    pub name: String,
    pub owner: String,
    pub target: String
}

pub enum GraphQL_Datatype {
    Number,
    String,
    Boolean,
    Array,
    Value,
    Object,
    Whitespace,
    null
}