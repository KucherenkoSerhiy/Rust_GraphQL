use mysql::PooledConn;
use std::vec::Vec;
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

    fn get_target_by_relation (&self, origin: String, relation: String) -> String{
        let mut target: String = "".to_string();
        for rel in &self.relations{
            if rel.owner == origin && rel.name == relation{
                target = rel.target.clone();
            }
        }
        target
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
        load_table_query = load_table_query + "CREATE TABLE IF NOT EXISTS " + &db_name + "." + &table.name; load_table_query = load_table_query + "(\n    id INT NOT NULL PRIMARY KEY AUTO_INCREMENT";
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
                "+ &column.name + " "+ &column.db_type + if column.is_mandatory {" NOT NULL"} else {""};
            }

        };
        load_table_query = load_table_query +"
    );\n";
        (load_table_query, rels)
    }

    pub fn create_relation_table (&self, db_name: String, relation: &def::Relation) -> String{
        let mut load_rel_query: String = "".to_string();
        load_rel_query = load_rel_query + "CREATE TABLE IF NOT EXISTS " + &db_name + "." + &relation.owner + "_" + &relation.name + "_" + &relation.target + "(\n   ";
        load_rel_query = load_rel_query + "origin_id INT,\n   target_id INT";

        load_rel_query = load_rel_query +"\n);\n";
        load_rel_query
    }

    pub fn perform_get_ids(&self, db_name: String, select_structure : &def::QueryObject) -> String{
        let last_column = select_structure.attrs.as_ref().unwrap().last().unwrap();
        let mut mysql_select: String = "SELECT id FROM ".to_string() + &(db_name) + "." + &select_structure.name + " ";
        if let &Some(parameters) = &select_structure.params.as_ref() {
            let last_param = select_structure.params.as_ref().unwrap().last().unwrap();
            mysql_select = mysql_select + "WHERE ";
            for parameter in parameters {
                mysql_select = mysql_select + &parameter.0 + "=\"" + &parameter.1 + "\"";
                if parameter != last_param {mysql_select = mysql_select + " AND";}
                mysql_select = mysql_select + " ";
            };
        }
        mysql_select = mysql_select + ";";
        //println!("mysql_select: {}", mysql_select);
        mysql_select
    }

    pub fn perform_get_rels (&self, db_name: String, select_structure : &def::QueryObject, origin_ids: Vec<i32>) -> String{
        let mut mysql_select_rels: String = "".to_string();
        let mut mysql_select_cols: String = "".to_string();

        for col in select_structure.attrs.as_ref().unwrap(){
            //println!("GOTCHA: {:?}", col);
            if let Some(attributes) = col.attrs.as_ref() {
                for attribute in attributes {
                    mysql_select_cols = mysql_select_cols + "target." + attribute.name.as_str() + ", ";
                };
                mysql_select_cols.pop();
                mysql_select_cols.pop();
                mysql_select_cols = mysql_select_cols + " ";
                for id in origin_ids.clone() {
                    mysql_select_rels = mysql_select_rels + "SELECT " + &mysql_select_cols;
                    mysql_select_rels = mysql_select_rels + "FROM "
                        + &(db_name) + "." + &select_structure.name + " AS origin, "
                        + &(db_name) + "." + &select_structure.name + "_" + &col.name + "_" + self.get_target_by_relation(select_structure.name.clone(), col.name.clone()).as_str() +" AS relation, "
                        + &(db_name) + "." + self.get_target_by_relation(select_structure.name.clone(), col.name.clone()).as_str() + " AS target ";
                    mysql_select_rels = mysql_select_rels + "WHERE "
                        + "origin.id = relation.origin_id AND "
                        + "target.id = relation.target_id AND "
                        + "origin.id = " + &id.to_string() + " ";
                    mysql_select_rels = mysql_select_rels + "GROUP BY relation.origin_id, relation.target_id";
                    mysql_select_rels = mysql_select_rels + ";";
                }

            }
        }

        //println!("\n\nmysql_select_rels:\n{}\n\n", mysql_select_rels);

        mysql_select_rels
    }
/*

*/
    pub fn perform_get(&self, db_name: String, select_structure : &def::QueryObject) -> String{
        let mut last_column_name = "".to_string();
        for col in select_structure.attrs.as_ref().unwrap(){
            if col.attrs.as_ref() == None {
                last_column_name = col.name.clone();
            }
        };


        let mut mysql_select: String = "SELECT ".to_string();
        for col in select_structure.attrs.as_ref().unwrap(){
            if col.attrs.as_ref() == None {
                mysql_select = mysql_select + col.name.as_str();
                if col.name != last_column_name {mysql_select = mysql_select + ","};
                mysql_select = mysql_select + " ";
            }
        }
        mysql_select = mysql_select + "FROM " + &(db_name) + "." + &select_structure.name + " ";
        if let &Some(parameters) = &select_structure.params.as_ref() {
            let last_param = select_structure.params.as_ref().unwrap().last().unwrap();
            mysql_select = mysql_select + "WHERE ";
            for parameter in parameters {
                mysql_select = mysql_select + &parameter.0 + "=\"" + &parameter.1 + "\"";
                if parameter != last_param {mysql_select = mysql_select + " AND";}
                mysql_select = mysql_select + " ";
            };
        }
        mysql_select = mysql_select + ";";

        //println!("\n\n mysql_select: \n{}\n\n", mysql_select);
        mysql_select
    }

    pub fn perform_add_rels(&self, db_name: String, origin: &def::MutationObject, rels: &def::MutationObject) -> String{
        let mut mysql_insert_rels: String = "".to_string();

        for target in rels.attrs.as_ref().unwrap(){
            mysql_insert_rels = mysql_insert_rels + "INSERT INTO "  + &db_name + "." + &origin.name + "_" + rels.name.as_str() + "_" + target.name.as_str() + "(\n    ";
            mysql_insert_rels = mysql_insert_rels + "    " + &origin.name + "_id" + &target.name + "_id" + "\n)\n" + "VALUES (\n    ";
            if let Some(params) = target.params.as_ref(){
                /*
                    MutationObject {
                        name: "Droid".to_string(),
                        value: None,
                        params: Some(vec![("id".to_string(), "1".to_string())]),
                        attrs: None
                    },
                    MutationObject {
                        name: "Droid".to_string(),
                        value: None,
                        params: Some(vec![("name".to_string(), "R2D2".to_string())]),
                        attrs: None
                    }
                */

            }
        }
        mysql_insert_rels
    }

    pub fn perform_add_mutation(&self, conn: &mut PooledConn, db_name: String, insert_structure : &def::MutationObject){
        let mut last_column_name = "".to_string();
        let mut last_value = &"".to_string();
        for col in insert_structure.attrs.as_ref().unwrap(){
            if let Some(val) = col.value.as_ref(){
                last_column_name = col.name.clone();
                last_value = val;
            }
        };

        let mut mysql_insert_rels: String = "".to_string();
        let mut mysql_insert: String = "INSERT INTO ".to_string() + &db_name + "." + &insert_structure.name + "(\n    ";
        /*COLUMNS*/
        for col in insert_structure.attrs.as_ref().unwrap(){
            if let Some(val) = col.value.as_ref(){
                mysql_insert = mysql_insert + col.name.as_str();
                if col.name != last_column_name {mysql_insert = mysql_insert + ","};
                mysql_insert = mysql_insert + " ";
            }
        }

        mysql_insert = mysql_insert + "\n)\n" + "VALUES (\n    ";
        for col in insert_structure.attrs.as_ref().unwrap(){
            if let Some(val) = col.value.as_ref(){
                mysql_insert = mysql_insert + "\"" + val.as_str() + "\"";;
                if val != last_value {mysql_insert = mysql_insert + ","};
                mysql_insert = mysql_insert + " ";
            }
        }
        mysql_insert = mysql_insert + "\n);\n";

        mysql_insert = mysql_insert + mysql_insert_rels.as_str();
        //println!("\n\n\n\n\n{}\n\n\n\n\n", mysql_insert);
        //got the id from origin
        let last_insert_id = conn.query(&mysql_insert).unwrap().last_insert_id();

        //got the id from targets

        //self.perform_add_rels(db_name, &insert_structure, last_insert_id);
        //println!("last_insert_id = {}", last_insert_id);
        //conn.query(&mysql_add_rels).unwrap().affected_rows();
    }

    pub fn perform_update_mutation(&self, db_name: String, update_structure : &def::MutationObject) -> String{
        let last_column = &update_structure.attrs.as_ref().unwrap().last().unwrap();
        let mut mysql_update: String = "UPDATE ".to_string() + &db_name + "." + &update_structure.name + " SET ";
        /*COLUMNS*/
        for col in update_structure.attrs.as_ref().unwrap(){
            mysql_update = mysql_update + col.name.as_str() + " = " + col.value.as_ref().unwrap().as_str();
            if col.name != last_column.name {mysql_update = mysql_update + ","};
            mysql_update = mysql_update + " ";
        }

        if let &Some(parameters) = &update_structure.params.as_ref() {
            let last_param = update_structure.params.as_ref().unwrap().last().unwrap();
            mysql_update = mysql_update + "WHERE ";
            for parameter in parameters {
                mysql_update = mysql_update + &parameter.0 + "=\"" + &parameter.1 + "\"";
                if parameter != last_param {mysql_update = mysql_update + " AND";}
                mysql_update = mysql_update + " ";
            };
        }

        mysql_update = mysql_update + ";";

        mysql_update
    }

    pub fn perform_delete_mutation(&self, db_name: String, delete_structure : &def::MutationObject) -> String{
        let mut mysql_delete: String = "DELETE FROM ".to_string() + &db_name + "." + &delete_structure.name + " ";
        if let &Some(parameters) = &delete_structure.params.as_ref() {
            let last_param = delete_structure.params.as_ref().unwrap().last().unwrap();
            mysql_delete = mysql_delete + "WHERE ";
            for parameter in parameters {
                mysql_delete = mysql_delete + &parameter.0 + "=\"" + &parameter.1 + "\"";
                if parameter != last_param {mysql_delete = mysql_delete + " AND";}
                mysql_delete = mysql_delete + " ";
            };
        }
        mysql_delete = mysql_delete + ";";

        mysql_delete
    }

}
