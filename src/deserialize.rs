use mysql;

pub fn perform_get(pool: &mysql::Pool, query: String, select_structure : &(&str, (&str, &str), Vec<&str>) ) -> String {
    pool.prep_exec(query, ()).map(|mut result| {
        let mut resulting_object : String = "".to_string();

        let mut row = result.next().unwrap().unwrap();

        resulting_object = "{\n  \"data\": {\n".to_string();
        for col in &select_structure.2{
            //let data : ColumnType = row.take(*col).unwrap();
            let data : String = row.take(*col).unwrap();
            match data {
                _ => resulting_object = resulting_object + "    \"" + col + "\": \"" + &data + "\"\n"
            };
        }
        resulting_object = resulting_object + "  }\n}";
        println!("{}", resulting_object);
        //let name: String = row.take("name").unwrap();
        resulting_object

    }).unwrap()
}

