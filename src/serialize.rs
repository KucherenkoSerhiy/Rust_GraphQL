use std::option::Option;
use std::vec::Vec;
use std::error::Error;
use def;

#[derive(Clone)]
pub struct Serializer{
    pub relations: Vec<def::Relation>
}

impl Serializer {
    pub fn new() -> Serializer{
        Serializer{
            relations: Vec::new()
        }
    }

    pub fn store_relations(&mut self, rels: &mut Vec<def::Relation>){
        self.relations.append(rels);
    }

    pub fn create_database(&self, db_name: String) -> String {
        "CREATE DATABASE IF NOT EXISTS ".to_string() + &db_name
    }

    pub fn use_database(&self, db_name: String) -> String {
        "USE ".to_string() + &db_name
    }

    pub fn destroy_database(&self, db_name: String) -> String {
        "DROP DATABASE IF EXISTS ".to_string() + &db_name
    }

    pub fn create_table(&self, db_name: String, table: &def::DbTable) -> (String, Vec<def::Relation>){
        let mut rels : Vec<def::Relation> = Vec::new();

        let mut load_table_query: String = "".to_string();
        load_table_query = load_table_query + "CREATE TABLE IF NOT EXISTS " + &db_name + "." + &table.name; load_table_query = load_table_query + "(
        " + &table.name + "_id INT NOT NULL PRIMARY KEY AUTO_INCREMENT";
        for column in &table.columns {
            if column.db_type.starts_with("["){
                if column.db_type.ends_with("]"){
                    let mut target = column.db_type.clone();
                    target.remove(0);
                    target.pop();
                    rels.push(
                        def::Relation{
                            name: column.name.clone(),
                            owner: table.name.clone(),
                            target: target
                        }
                    );
                }
                    else {
                    panic!("Error: Unexpected column type");
                }
            }
                else {
                load_table_query = load_table_query + ",
            "+ &column.name + " "+ &column.db_type;
            }

        };
        load_table_query = load_table_query +"
    );\n";
        (load_table_query, rels)
    }

    pub fn create_relation_table (&self, db_name: String, relation: &def::Relation) -> String{
        let mut load_rel_query: String = "".to_string();
        load_rel_query = load_rel_query + "CREATE TABLE IF NOT EXISTS " + &db_name + "." + &relation.owner + "_" + &relation.name + "_" + &relation.target + "(\n   ";
        if &relation.owner == &relation.target{
            load_rel_query = load_rel_query + "origin_id INT,\n   target_id INT";
        }
        else {
            load_rel_query = load_rel_query + &relation.owner + "_id INT"+ ",\n   " + &relation.target + "_id INT";
        }

        load_rel_query = load_rel_query +"\n);\n";
        load_rel_query
    }

    pub fn perform_get(&self, db_name: String, select_structure : &def::Query_Object) -> String{
        let last_column = select_structure.attrs.as_ref().unwrap().last().unwrap();
        let mut mysql_select: String = "SELECT ".to_string();
        for col in select_structure.attrs.as_ref().unwrap(){
            mysql_select = mysql_select + col.name.as_str();
            if col != last_column {mysql_select = mysql_select + ","};
            mysql_select = mysql_select + " "
        }
        mysql_select = mysql_select + "FROM " + &(db_name) + "." + &select_structure.name + " ";
        if let &Some(parameter) = &select_structure.params.as_ref() {
            mysql_select = mysql_select + "WHERE " + &parameter.0 + "=" + &parameter.1;
        }
        mysql_select = mysql_select + ";";

        mysql_select
    }

    pub fn perform_add_mutation(&self, db_name: String, insert_structure : &(String, Vec<(String, String)> )) -> String{
        let last_column = insert_structure.1.last().unwrap();

        let mut mysql_insert: String = "INSERT INTO ".to_string() + &db_name + "." + &insert_structure.0 + "(";
        /*COLUMNS*/
        for col in &insert_structure.1{
            mysql_insert = mysql_insert + &col.0;
            if col.0 != last_column.0 {mysql_insert = mysql_insert + ","};
            mysql_insert = mysql_insert + " ";
        }

        mysql_insert = mysql_insert + ")\n" +

            "VALUES (";
        for col in &insert_structure.1{
            mysql_insert = mysql_insert + "\"" + &col.1 + "\"";;
            if col.1 != last_column.1 {mysql_insert = mysql_insert + ","};
            mysql_insert = mysql_insert + " ";
        }
        mysql_insert = mysql_insert + ");";

        mysql_insert
    }

    pub fn perform_update_mutation(&self, db_name: String, update_structure : &(String, (String, String), Vec<(String, String)> )) -> String{
        let last_column = &update_structure.2.last().unwrap();
        let mut mysql_update: String = "UPDATE ".to_string() + &db_name + "." + &update_structure.0 + " SET ";
        /*COLUMNS*/
        for col in &update_structure.2{
            mysql_update = mysql_update + &col.0 + " = " + &col.1;
            if col.0 != last_column.0 {mysql_update = mysql_update + ","};
            mysql_update = mysql_update + " ";
        }

        mysql_update = mysql_update + "WHERE " + &(update_structure.1).0 + " = " + &(update_structure.1).1 + ";";

        mysql_update
    }

    pub fn perform_delete_mutation(&self, db_name: String, delete_structure : &(String, Option<(String, String)> )) -> String{
        let mut mysql_delete: String = "DELETE FROM ".to_string() + &db_name + "." + &delete_structure.0 + " ";
        if let Some(id) = delete_structure.1.as_ref() {
            mysql_delete = mysql_delete + "WHERE " + &id.0 + "=" + &id.1;
        }
        mysql_delete = mysql_delete + ";";

        mysql_delete
    }

}
