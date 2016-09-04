use mysql;
use def;

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
        let mut n = 0;
        while n < self.tabs {
            tabbings = tabbings + "\t";
            n = n+1;
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

    pub fn perform_get_ids(&mut self, pool: &mysql::Pool, query: String) -> Vec<i32> {
        let mut ids : Vec<i32> = Vec::new();

        let mut query_result = pool.prep_exec(query, ()).unwrap();

        for result in query_result.by_ref() {
            let mut row = result.unwrap();
            ids.push(row.take("id").unwrap());
        };
        ids
    }

    fn perform_get_relations(&mut self, pool: &mysql::Pool, query_relations: String, select_structure : &def::QueryObject ) -> String{
        let mut json = "".to_string();

        let mut query_result = pool.prep_exec(query_relations, ()).unwrap();

        for col in select_structure.attrs.as_ref().unwrap(){
            if col.attrs.as_ref() != None {

                json = json + &(self.get_tabulation()) + "\"" + col.name.as_str() + "\": [  " + &(self.endline());
                self.add_tabbing();

                for result in query_result.by_ref() {
                    json = json + &(self.get_tabulation()) + "{" + &(self.endline());
                    self.add_tabbing();

                    let mut related_col = 0;
                    let mut row: Vec<mysql::Value> = result.unwrap().unwrap();
                    for value in row{
                        json = json + &(self.get_tabulation()) + "\"" + col.attrs.as_ref().unwrap()[related_col].name.as_str() + "\": " + &value.into_str() + &(self.endline());
                        related_col = related_col + 1;
                    }

                    self.remove_tabbing();
                    json = json + &(self.get_tabulation()) + "}," + &(self.endline());
                }
                json.pop();
                json.pop();
                json = json + &(self.endline());

                self.remove_tabbing();
                json = json + &(self.get_tabulation()) + "]" + &(self.endline());
            }
        }

        json
    }

    pub fn perform_get(&mut self, pool: &mysql::Pool, query_objects: String, query_relations: String, select_structure : &def::QueryObject ) -> String {
        let mut json = "".to_string();

        json = json + &(self.get_tabulation()) + "{" + &(self.endline());
        self.add_tabbing();

        json = json + &(self.get_tabulation()) + "\"data\": {" + &(self.endline());
        self.add_tabbing();

        let mut query_result = pool.prep_exec(query_objects, ()).unwrap();

        for result in query_result.by_ref() {
            let mut row = result.unwrap();
            let mut resulting_object : String = "".to_string();

            resulting_object = resulting_object + &(self.get_tabulation()) + "\"" + select_structure.name.as_str() + "\": {" + &(self.endline());
            self.add_tabbing();

            for col in select_structure.attrs.as_ref().unwrap(){
                if col.attrs.as_ref() == None {
                    let data : mysql::Value = row.take(col.name.as_str()).unwrap();
                    resulting_object = resulting_object + &(self.get_tabulation()) + "\"" + &col.name + "\": " + &(data.into_str()) + if col != select_structure.attrs.as_ref().unwrap().last().unwrap() {","} else {""} + &(self.endline())
                }
                else {
                    resulting_object = resulting_object + self.perform_get_relations(pool, query_relations.clone(), select_structure).as_str();
                }
            }

            self.remove_tabbing();
            resulting_object = resulting_object + &(self.get_tabulation()) + "}," + &(self.endline());
            json = json + resulting_object.as_str();

        };
        json.pop();
        json.pop();
        json = json + &(self.endline());

        self.remove_tabbing();
        json = json + &(self.get_tabulation()) + "}" + &(self.endline());

        self.remove_tabbing();
        json = json + &(self.get_tabulation()) + "}" + &(self.endline());

        json
    }
}
