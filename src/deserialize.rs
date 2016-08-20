use std::option::Option;
use mysql;

pub fn perform_get(pool: &mysql::Pool, query: String, select_structure : &(&str, Option<(&str, &str)>, Vec<&str>) ) -> String {
    let mut json = "{\n  \"data\": {\n".to_string();
    for result in pool.prep_exec(query, ()).unwrap() {
        let mut row = result.unwrap();
        let mut resulting_object : String = "".to_string();

        resulting_object = resulting_object + "    \"" + select_structure.0 + "\": {\n";

        for col in &select_structure.2{
            //let data : ColumnType = row.take(*col).unwrap();
            let data : String = row.take(*col).unwrap();
            match data {
                _ => resulting_object = resulting_object + "      \"" + col + "\": \"" + &data + "\"\n"
            };
        }
        resulting_object = resulting_object + "    }\n";
        //let name: String = row.take("name").unwrap();
        json = json + resulting_object.as_str();
    };
    json = json + "  }\n}";

    //println!("\n\n\n\n\n{}\n\n\n\n\n", json);
    json
}
