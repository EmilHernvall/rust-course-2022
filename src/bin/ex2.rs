use rustdemo::{City, CityData};

pub fn load_cities() -> Result<Vec<City>, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string("cities100k.json")?;
    let cities: Vec<City> = serde_json::from_str(&data)?;
    Ok(cities)
}

pub fn largest_city(cities: &Vec<City>, country: &str) {
    let mut max_population = 0;
    let mut largest_city: Option<&City> = None;
    for city in cities {
        if city.fields.country_code != country {
            continue;
        }

        if city.fields.population > max_population {
            max_population = city.fields.population;
            largest_city = Some(city);
        }
    }

    println!("{:#?}", largest_city);
}

pub enum Filter {
    CountryCode(String),
    Admin1Code(String),
    TimeZone(String),
}

pub fn filter_cities(cities: &Vec<City>, filter: Filter) {
    for city in cities {
        match &filter {
            Filter::CountryCode(v) => {
                if &city.fields.country_code != v {
                    continue;
                }
            }
            Filter::Admin1Code(v) => {
                let code = match &city.fields.admin1_code {
                    Some(x) => x,
                    None => continue,
                };

                if code != v {
                    continue;
                }
            }
            Filter::TimeZone(v) => {
                if &city.fields.timezone != v {
                    continue;
                }
            }
        }

        println!("{:#?}", city);
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cities = load_cities()?;

    let foo: String = "SE".to_string();

    largest_city(&cities, &foo);
    largest_city(&cities, "UZ");

    let filter = Filter::CountryCode("US".to_string());

    filter_cities(&cities, filter);

    Ok(())
}
