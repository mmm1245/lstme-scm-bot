use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use anyhow::anyhow;
use json::{object, JsonValue};
use crate::{Element, Inventory, Miestnost, Recipe};
use crate::Miestnost::Sklad;

pub struct Client{
    socket: TcpStream,
    reader: BufReader<TcpStream>,
    current_room: Miestnost,
    current_inventory: Inventory
}
impl Client{
    pub fn new(address: &str, name: &str, key: &str) -> Self{
        let mut socket = TcpStream::connect(address).unwrap();
        socket.write(".mode api\n".as_ref()).unwrap();
        let mut reader = BufReader::new(socket.try_clone().unwrap());
        let mut client = Client{
            socket,
            reader,
            current_room: Miestnost::Raketa,
            current_inventory: Inventory{items: Vec::new()},
        };
        let login = client.perform("login", vec![name.to_string(), key.to_string()]);
        client.current_room = Miestnost::from_str(login["location"]["name"].as_str().unwrap()).unwrap();
        client.reload_inventory();
        client
    }
    pub fn perform(&mut self, action: &str, arguments: Vec<String>) -> JsonValue{
        let json = object! {
            action: format!(".{}", action).as_str(),
            args: arguments,
        };
        //println!("send: {}", json.to_string());
        self.socket.write(format!("{}\n", json.to_string()).as_bytes()).unwrap();
        self.socket.flush().unwrap();
        //thread::sleep(Duration::from_millis(100));
        if action != "order"{
            self.read_json(if action == "login" {"look"} else {action})
        } else {
            JsonValue::Null
        }
    }
    fn reload_inventory(&mut self){
        let json = self.perform("examine", vec![]);
        self.current_inventory = Inventory{items: json["items"].members().map(|json|Element::from_str(json["name"].as_str().unwrap()).unwrap()).collect()}
    }
    pub fn deposit_all(&mut self){
        self.go_to(Sklad);
        let mut args = vec!["elem".to_string(), "deposit".to_string()];
        args.extend(self.current_inventory.items.iter().map(|item|item.to_string()));
        self.perform("use", args);
        self.current_inventory.items.clear();
    }
    pub fn take_items(&mut self, items: Vec<Element>) -> anyhow::Result<()>{
        if self.current_inventory.items.len() + items.len() > 10{
            return Err(anyhow!("full inventory"));
        }
        self.go_to(Miestnost::Sklad);
        /*let mut args = vec!["elem".to_string(), "take".to_string()];
        args.extend(items.iter().map(|item|item.to_string()));
        let code = self.perform("use", args);*/
        for item in items{
            let code = self.perform("use", vec!["el".to_string(), "take".to_string(), item.to_string()]);
            if !code.has_key("error"){
                self.current_inventory.items.push(item);
            } else {
                return Err(anyhow!("not enough items {}", code));
            }
        }
        //self.current_inventory.items.extend(items);
        Ok(())
    }
    pub fn have_items(&mut self, items: Vec<Element>){
        let mut to_take = items.clone();
        for current in &self.current_inventory.items{
            if let Some(pos) = to_take.iter().position(|x| *x == *current) {
                to_take.remove(pos);
            }
        }
        let mut extra = self.current_inventory.items.clone();
        for needed in &items{
            if let Some(pos) = extra.iter().position(|x| *x == *needed) {
                extra.remove(pos);
            }
        }
        {
            let mut args = vec!["elem".to_string(), "deposit".to_string()];
            let dep: Vec<Element> = extra.iter().take((self.current_inventory.items.len() as isize + to_take.len() as isize - 10).max(0) as usize).cloned().collect();
            args.extend(dep.iter().map(|item|item.to_string()));
            for dep in &dep{
                if let Some(pos) = self.current_inventory.items.iter().position(|x| *x == *dep) {
                    self.current_inventory.items.remove(pos);
                }
            }
            if !dep.is_empty(){
                self.perform("use", args);
            }
        }
        self.reload_inventory();
        self.take_items(to_take).unwrap();
    }
    pub fn craft(&mut self, recipe: &Recipe) -> anyhow::Result<()>{
        self.have_items(recipe.ingredients.clone());
        self.go_to(recipe.station);
        let craft = self.perform("craft", recipe.ingredients.iter().map(|elem|elem.to_string()).collect());
        //println!("craft: {}", craft);
        //craft: {"action":"craft","crafted":[{"id":"d2ba1524-a182-4d79-871a-19b9e9d2c3cc","name":"Rekurzium","amount":1}],"used":[{"id":"e4965b7f-1252-46da-aa49-a8a68d721e54","name":"Andymón","amount":2},{"id":"547d60d1-826f-461e-95f7-e6454d01d52c","name":"Ahmedium","amount":1}],"recipe":{"id":"ef8e61c8-84eb-428a-a0e3-c54204df846b","name":"Rekurzium"}}
        self.reload_inventory();
        Ok(())
    }
    pub fn go_to(&mut self, miestnost: Miestnost){
        for path in self.current_room.pathfind(miestnost){
            self.perform("go", vec![path.to_string()]);
            self.current_room = path;
        }
    }
    pub fn read_string(&mut self) -> String{
        let mut str = String::new();
        self.reader.read_line(&mut str).unwrap();
        str
    }
    pub fn list_elements_in_storage(&mut self) -> HashMap<Element, u32>{
        self.go_to(Sklad);
        let elements = self.perform("use", vec!["el".to_string(), "list".to_string()]);
        let elements = &elements["data"]["elements"];
        elements.members().map(|json|(Element::from_str(json["name"].as_str().unwrap()).expect(format!("unknown item {}", json["name"].as_str().unwrap()).as_str()), json["count"].as_u32().unwrap())).collect()
    }
    pub fn read_json(&mut self, action: &str) -> JsonValue{
        loop{
            let str = self.read_string();
            //println!("mfg: {str}");
            match json::parse(str.as_str()){
                Ok(json) => {
                    if json["action"].as_str().map(|str|str==action).unwrap_or(false) || json.has_key("error") || (action == "units" && json.has_key("combat_units")){
                        return json;
                    } else {
                        continue;
                    }
                },
                Err(_) => continue,
            }
        }
    }
}