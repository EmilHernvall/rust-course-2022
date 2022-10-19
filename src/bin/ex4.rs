
use std::collections::HashMap;

use rustdemo::{load_cities, City, CityData};

fn get_population(city: &&City) -> i64 {
    -city.fields.population
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cities = load_cities()?;

    let mut cities_by_state: HashMap<String, Vec<&City>> = HashMap::new();
    for city in &cities {
        if &city.fields.country_code != "US" {
            continue;
        }

        let state = match &city.fields.admin1_code {
            Some(x) => x.to_string(),
            None => continue,
        };

        cities_by_state.entry(state)
            .or_default()
            .push(city);
    }

    for cities in cities_by_state.values_mut() {
        // cities.sort_by_key(|city| -city.fields.population);
        cities.sort_by_key(get_population);
    }

    for (state, cities) in cities_by_state {
        println!("{} - {}", state, cities[0].fields.name);
    }

    Ok(())
}
