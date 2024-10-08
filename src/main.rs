mod client;

use std::collections::{HashMap, HashSet};
use std::fmt::{format, Debug, Display, Formatter};
use std::str::FromStr;
use std::{io, iter, thread};
use std::env::args;
use std::io::Write;
use std::time::{Duration, Instant};
use itertools::Itertools;
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumCount, EnumIter};
use unidecode::unidecode;
use Element::Diodik;
use crate::client::{Client};
use crate::Element::{Ahmedium, Andymon, Avatar, Awentit, Badassium, Droid, Forcyklat, Javascrypton, Jeep, Kapacitat, Magneton, Misonit, Obskurcium, Rekurzium, Rezistor, Tank, Transcitor, Triodium, Valteren};
use crate::Miestnost::{Datacentrum, Dielna, Hangar, Hub, Konstrukcia, Labak, Raketa, Velin, Vypoctovka, Avecko, Sklad};

fn main() {
    if args().nth(1).map(|arg|arg=="research").unwrap_or(false){
        let mut clients: Vec<_> = std::fs::read_to_string("clients.txt").unwrap().lines().filter(|str|!str.starts_with("#")).map(|str| {
            let (name,password) = str.split_once(";").unwrap();
            Client::new("scm.lstme.sk:7000", name, password)
        }).collect();
        loop {
            let time = Instant::now();
            for client in &mut clients {
                client.go_to(Datacentrum);
                let json = client.perform("use", vec!["PEPU".to_string()]);
                if json.has_key("error"){
                    println!("error: {}", json["error"]);
                }
            }
            println!("took: {}", time.elapsed().as_millis());
            thread::sleep(Duration::from_millis((10000/* - time.elapsed().as_millis()*/) as u64));
        }
        return;
    }
    //println!("{}", Miestnost::from_str("AVečko").unwrap());
    let mut client = Client::new("scm.lstme.sk:7000", "mmm1245", "gtvC6h9mbD9jGKFpe41aKRB6");
    //let mut client2 = Client::new("scm.lstme.sk:7000", "mmm1245's slave #1", "3hr2mxWYNuQ3qu2p96ZdzEN8");
    //let mut client3 = Client::new("scm.lstme.sk:7000", "mmm1245's slave #2", "PDUJeB7eaCKXZhVvFty2hyei");
    //let mut client = Client::new("localhost:6969", "mmm1245", "gtvC6h9mbD9jGKFpe41aKRB6");

    /*let places = vec!["9x6"/*,"10x2","9x3","9x4","9x5","10x6","10x7"*/];
    let mut together: HashMap<String, Vec<String>> = HashMap::new();
    for (i, unit) in client.perform("units", vec![])["combat_units"].members().enumerate(){
        together.entry(format!("{}{}",unit["placement"]["name"].as_str().unwrap(),unit["name"].as_str().unwrap() )).or_insert(Vec::new()).push(unit["callsign"].as_str().unwrap().to_string());
        continue;
        client.perform("order", vec!["move".to_string(), unit["callsign"].as_str().unwrap().to_string(), places[i%places.len()].to_string()]);
    }
    for (place,list) in together{
        let mut aaa =  vec!["join".to_string()];
        aaa.extend(list);
        client.perform("order", aaa);
    }
    return;*/
    let mut required: HashMap<Element, isize> = HashMap::new();
    required.insert(Andymon, 1000-10);
    //let mut t = Vec::new();
    for (material,mut count) in required{
        //let mut client = clients.pop().unwrap();
        //t.push(thread::spawn(move || {
            while count > 0 {
                client.deposit_all();
                let items: Vec<Element> = iter::repeat(material).take(count.min(10) as usize).collect();
                client.take_items(items.clone()).unwrap();
                client.go_to(Hub);
                let mut args = vec!["sg".to_string()];
                args.extend(items.iter().map(|elem| elem.to_string()));
                client.perform("use", args);
                count -= 10;
            }
        //}));
    }
    /*for t in t{
        t.join().unwrap();
    }*/

    /*client.craft(&Recipe{ artefakt kolektor
        station: Dielna,
        ingredients: vec![Rekurzium, Kapacitat, Javascrypton, Obskurcium, Rezistor]
    }).unwrap();*/
    //println!("here");
    //println!("{}", client.perform("examine", vec![]));
    //return;
    /*for _ in 0..50 {
        //recursive_craft(&mut client, Rezistor);
        recursive_craft(&mut client, Jeep);

        println!("here");
        //client.deposit_all();
        client.go_to(Hangar);
        client.perform("use", vec!["jeep".to_string()]);
    }*/
    /*let mut jednotka = 0;
    let policka = vec![(3,3),(1,7),(0,9),(0,9),(0,9),(0,9),(0,9),(2,5),(4,1)];
    for (i,policko) in policka.iter().enumerate(){
        for j in 0..policko.1{
            println!("here");
            client.perform("order", vec!["move".to_string(), format!("{}WPV", (jednotka%4)+1), format!("{}x{}", i+4, policko.0+j)]);
            jednotka += 1;
            println!("ordered");
            thread::sleep(Duration::from_secs(30));
        }
    }*/
    //bruteforce_crafts(&mut client, 7, 3, &[Forcyklat, Rekurzium, Kapacitat, Javascrypton, Misonit, Obskurcium, Rezistor, Triodium, Badassium, Magneton, Transcitor, Diodik, Ahmedium, Valteren, Andymon]);
}
fn recursive_craft(client: &mut Client, element: Element){
    client.deposit_all();
    let mut items = client.list_elements_in_storage();
    recurse(client, element, &mut items);
    fn recurse(client: &mut Client, element: Element, items: &mut HashMap<Element,u32>){
        if let Some(recipe) = element.get_recipe(){
            for ing in &recipe.ingredients{
                let entry = items.entry(*ing).or_insert(0);
                if *entry > 0{
                    *entry -= 1;
                } else {
                    recurse(client, *ing, items);
                }
            }
            client.craft(&recipe).unwrap();
        }
    }
}
fn bruteforce_crafts(client: &mut Client, ingredient_count: usize, max_per: usize, bruteforcing: &[Element]){
    let mut tested: HashSet<String> = std::fs::read_to_string("testedrecipes.txt").unwrap_or(String::new()).lines().map(|s|s.to_string()).collect();
    println!("bruteforce start");
    let mut i = 0;
    let mut values = vec![0; Element::COUNT];
    'end: loop{
        if values.iter().sum::<usize>() == ingredient_count{
            let mut ingredients = Vec::new();
            for i in 0..bruteforcing.len(){
                for _ in 0..values[i]{
                    ingredients.push(bruteforcing[i]);
                }
            }
            let recipe_id = ingredients.iter().map(|ing|ing.to_string()).sorted().join("");
            if tested.insert(recipe_id) {
                i += 1;
                if i%10==0{
                    std::fs::write("testedrecipes.txt", tested.iter().cloned().join("\n").as_bytes()).unwrap();
                }
                client.have_items(ingredients.clone());
                let result = client.perform("craft", ingredients.iter().map(|ing| ing.to_string()).collect());
                if result["error"].as_str().unwrap().starts_with("Recept pre") {
                    print!(".");
                    io::stdout().flush().unwrap();
                } else {
                    println!("\ni: {:?} {}", ingredients, result);
                }
            }
            //thread::sleep(Duration::from_secs(1));
        }
        for i in 0..bruteforcing.len(){
            values[i] += 1;
            if values[i] > max_per{
                if i == bruteforcing.len()-1{
                    break 'end;
                }
                values[i] = 0;
            } else {
                break;
            }
        }
    }
    std::fs::write("testedrecipes.txt", tested.iter().cloned().join("\n").as_bytes()).unwrap();
}
#[derive(Copy, Clone, Eq, PartialEq, Hash, Display)]
enum Miestnost {
    Hub,
    Hangar,
    Raketa,
    Avecko,
    Labak,
    Vypoctovka,
    Velin,
    Datacentrum,
    Konstrukcia,
    Dielna,
    Sklad,
}
impl FromStr for Miestnost{
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match unidecode(s.trim().to_lowercase().as_str()).as_str(){
             "hub" => Hub,
             "hangar" => Hangar,
             "raketa" => Raketa,
             "avecko" => Avecko,
             "labak" => Labak,
             "vypoctovka" => Vypoctovka,
             "velin" => Velin,
             "datacentrum" => Datacentrum,
             "konstrukcia" => Konstrukcia,
             "dielna" => Dielna,
            "sklad" => Sklad,
            _ => return Err(())
        })
    }
}
impl Miestnost {
    pub fn get_neighbors(self) -> &'static [Miestnost]{
        match self{
            Hub => &[Vypoctovka, Hangar, Avecko, Konstrukcia],
            Hangar => &[Hub, Raketa],
            Raketa => &[Hangar],
            Avecko => &[Hub, Labak],
            Labak => &[Avecko],
            Vypoctovka => &[Hub,Datacentrum,Velin],
            Velin => &[Vypoctovka],
            Datacentrum => &[Vypoctovka],
            Konstrukcia => &[Hub, Dielna, Sklad],
            Dielna => &[Konstrukcia],
            Sklad => &[Konstrukcia],
        }
    }
    pub fn pathfind(self, other: Miestnost) -> Vec<Miestnost>{
        let mut path = Vec::new();
        if self == other{
            return path;
        }
        let mut went = HashSet::new();
        went.insert(self);
        fn recurse(miestnost: Miestnost, other: Miestnost, went: &mut HashSet<Miestnost>, path: &mut Vec<Miestnost>) -> bool {
            for neighbor in miestnost.get_neighbors(){
                if went.contains(neighbor){
                    continue;
                }
                went.insert(*neighbor);
                path.push(*neighbor);
                if *neighbor == other || recurse(*neighbor, other, went, path){
                    return true;
                }
                path.pop();
            }
            false
        };
        recurse(self, other, &mut went, &mut path);
        path
    }
}
#[derive(Copy, Clone, Eq, PartialEq, Hash, Display, EnumIter, EnumCount, Debug)]
pub enum Element {
    Ahmedium,
    Andymon,
    Valteren,
    Rekurzium,
    Kapacitat,
    Forcyklat,
    Javascrypton,
    Misonit,
    Obskurcium,
    Rezistor,
    Droid,
    Avatar,
    Magneton,
    Triodium,
    Badassium,
    Jeep,
    Transcitor,
    Tank,
    Diodik,
    Artefact,
    Awentit
}
impl Element{
    pub fn get_recipe(self) -> Option<Recipe>{
        Some(match self{
            Kapacitat => Recipe{
                station: Labak,
                ingredients: vec![Ahmedium, Ahmedium, Valteren]
            },
            Forcyklat => Recipe{
                station: Labak,
                ingredients: vec![Ahmedium, Valteren, Andymon]
            },
            Rekurzium => Recipe{
                station: Labak,
                ingredients: vec![Ahmedium, Andymon, Andymon]
            },
            Javascrypton => Recipe{
                station: Labak,
                ingredients: vec![Ahmedium, Valteren, Valteren]
            },
            Misonit => Recipe{
                station: Labak,
                ingredients: vec![Valteren, Rekurzium, Kapacitat, Javascrypton]
            },
            Obskurcium => Recipe{
                station: Labak,
                ingredients: vec![Ahmedium, Forcyklat, Rekurzium, Rekurzium, Javascrypton],
            },
            Rezistor => Recipe{
                station: Labak,
                ingredients: vec![Andymon, Forcyklat, Rekurzium, Rekurzium, Javascrypton],
            },
            Droid => Recipe{
                station: Dielna,
                ingredients: vec![Ahmedium, Valteren, Andymon, Kapacitat, Javascrypton],
            },
            Avatar => Recipe{
                station: Dielna,
                ingredients: vec![Ahmedium, Ahmedium, Valteren, Andymon, Andymon],
            },
            Triodium => Recipe{
                station: Labak,
                ingredients: vec![Ahmedium, Valteren, Forcyklat, Misonit, Obskurcium],
            },
            Jeep => Recipe{
                station: Dielna,
                ingredients: vec![Forcyklat, Kapacitat, Javascrypton, Misonit, Rezistor],
            },
            Transcitor => Recipe{
                station: Labak,
                ingredients: vec![Andymon, Forcyklat, Misonit, Misonit, Rezistor],
            },
            Tank => Recipe{
                station: Dielna,
                ingredients: vec![Misonit, Obskurcium, Rezistor, Magneton, Transcitor]
            },
            Diodik => Recipe{
                station: Labak,
                ingredients: vec![Rekurzium, Kapacitat, Misonit, Rezistor, Rezistor]
            },
            _ => return None,
        })
    }
}
impl FromStr for Element{
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = unidecode(s.trim().to_lowercase().as_str());
        Element::iter().find(|e|e.to_string().to_lowercase() == s).ok_or(())
    }
}
pub struct Inventory{
    pub items: Vec<Element>
}
pub struct Recipe{
    pub station: Miestnost,
    pub ingredients: Vec<Element>,
}