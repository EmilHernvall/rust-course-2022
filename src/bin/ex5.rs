use std::{collections::HashMap, ptr};

use apricity::{Coordinate, Point, gui::SimpleWindow};

use rustdemo::{FeatureCollection, Geometry, load_cities, City};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string("countries.geojson")?;
    let countries: FeatureCollection = serde_json::from_str(&data)?;

    let cities: Vec<City> = load_cities()?;

    let mut cities_by_country: HashMap<String, Vec<&City>> = HashMap::new();
    for city in &cities {
        cities_by_country.entry(city.fields.country_code.to_string())
            .or_default()
            .push(city);
    }

    let largest_cities: Vec<&City> = cities_by_country.values()
        .map(|city_list| *city_list.iter().max_by_key(|x| x.fields.population).unwrap())
        .collect();

    let width = 1500;
    let height = 750;

    let mut image = apricity::gui::SimpleImage::new(width, height);

    for country in &countries.features {
        let multi_polygon = match &country.geometry {
            Geometry::MultiPolygon(p) => p,
            _ => continue,
        };

        for polygon in &multi_polygon.coordinates {
            for ring in polygon {
                let ring: Vec<Point> = ring.iter()
                    .map(|x| x.screen(width as f64, height as f64))
                    .collect();

                image.draw_polygon(&ring, [ 0, 0xFF, 0, 0xFF ]);
            }
        }
    }

    let window = SimpleWindow::new(width, height, ())?;

    window.run(|window, _events| {
        window.draw_image(&image, None, false)?;

        for city in &largest_cities {

            let p = match &city.geometry {
                Geometry::Point(p) => p,
                _ => continue,
            };

            let p = p.coordinates.screen(width as f64, height as f64);

            window.stroke_circle(
                p.x,
                p.y,
                5.0,
                1.0, 
                [ 0xFF, 0, 0, 0xFF ],
            )?;
        }

        Ok(())
    })?;

    Ok(())
}