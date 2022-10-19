use std::collections::HashMap;

use apricity::{Coordinate, Point, gui::SimpleWindow};

use rustdemo::{FeatureCollection, Geometry};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string("countries.geojson")?;
    let countries: FeatureCollection = serde_json::from_str(&data)?;

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

        window.stroke_circle(
            750.0,
            350.0,
            50.0,
            1.0, 
            [ 0xFF, 0, 0, 0xFF ],
        )?;

        Ok(())
    })?;

    Ok(())
}