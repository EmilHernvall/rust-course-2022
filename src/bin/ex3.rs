use std::collections::HashMap;

use rustdemo::{load_cities, City, CityData};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cities = load_cities()?;

    let mut countries: HashMap<String, Vec<&City>> = HashMap::new();
    for city in &cities {
        let name = city.country_name_eng();

        // if !countries.contains_key(name) {
        //     countries.insert(name.to_string(), vec![]);
        // }

        // let mut country_cities = countries.get_mut(name).unwrap();
        // country_cities.push(city);

        countries.entry(name.to_string()).or_default().push(city);
    }

    for city_list in countries.values_mut() {
        city_list.sort_by_key(|city| -city.fields.population);
    }

    for (country_name, city_list) in &countries {
        println!("{}", country_name);

        for city in city_list {
            println!("\t{} {}", city.fields.name, city.fields.population);
        }
    }

    Ok(())
}

// Exercise: Write a program that prints the largest city for each US state
// (States are in admin1_code)
