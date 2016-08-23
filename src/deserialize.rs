use std::option::Option;
use mysql;

pub struct Deserializer{
    tabs: i8
}

impl Deserializer {
    pub fn new() -> Deserializer{
        Deserializer{
            tabs: 0
        }
    }

    fn get_tabulation(&self) -> String{
        let mut tabbings = "".to_string();
        for x in 0..self.tabs {
            tabbings = tabbings + "\t";
        }
        tabbings
    }

    fn endline(&self) -> String {
        "\n".to_string()
    }

    fn add_tabbing(&mut self){
        self.tabs = self.tabs+1;
    }

    fn remove_tabbing(&mut self){
        if self.tabs > 0 {self.tabs = self.tabs-1};
    }

//  pub fn perform_get(&mut self, pool: &mysql::Pool, query: String, select_structure : &(&str, Option<(&str, &str)>, Vec<&str>) ) -> String {


    pub fn perform_get(&mut self, pool: &mysql::Pool, query: String, select_structure : &(String, Option<(String, String)>, Vec<String>) ) -> String {
        let mut json = "".to_string();

        json = json + &(self.get_tabulation()) + "{" + &(self.endline());
        self.add_tabbing();

        json = json + &(self.get_tabulation()) + "\"data\": {" + &(self.endline());
        self.add_tabbing();

        let mut queryResult = pool.prep_exec(query, ()).unwrap();

        for result in queryResult.by_ref() {
            let mut row = result.unwrap();
            let mut resulting_object : String = "".to_string();

            resulting_object = resulting_object + &(self.get_tabulation()) + "\"" + select_structure.0.as_str() + "\": {" + &(self.endline());
            self.add_tabbing();

            for col in &select_structure.2{
                let data : mysql::Value = row.take(col.as_str()).unwrap();
                //let data : String = row.take(*col).unwrap();
                match data {
                    ref Bytes => resulting_object = resulting_object + &(self.get_tabulation()) + "\"" + &col + "\": " + &(data.into_str()) + if *col != *(select_structure.2.last().unwrap()) {","} else {""} + &(self.endline())
                    //_ => unimplemented!()
                };
            }

            self.remove_tabbing();
            resulting_object = resulting_object + &(self.get_tabulation()) + "}" + if row == row {","} else {""} + &(self.endline());
            //let name: String = row.take("name").unwrap();
            json = json + resulting_object.as_str();
        };

        self.remove_tabbing();
        json = json + &(self.get_tabulation()) + "}" + &(self.endline());

        self.remove_tabbing();
        json = json + &(self.get_tabulation()) + "}" + &(self.endline());

        println!("\n\n\n\n\n{}\n\n\n\n\n", json);
        json
    }
}
