
#[derive(Clone, Debug, serde::Deserialize)]
pub struct City {
    pub datasetid: String,
    pub recordid: String,
    pub fields: CityData,
    pub record_timestamp: String,
}

impl City {
    pub fn country_name_eng(&self) -> &str {
        match &self.fields.cou_name_en {
            Some(x) => x,
            None => &self.fields.country_code,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct CityData {
    pub coordinates: [f64; 2],
    pub cou_name_en: Option<String>,
    pub label_en: Option<String>,
    pub feature_code: String,
    pub population: i64,
    pub dem: i64,
    pub geoname_id: String,
    pub name: String,
    pub admin1_code: Option<String>,
    pub admin2_code: Option<String>,
    pub admin3_code: Option<String>,
    pub admin4_code: Option<String>,
    pub feature_class: String,
    pub country_code: String,
    pub timezone: String,
    pub modification_date: String,
}

pub fn load_cities() -> Result<Vec<City>, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string("cities100k.json")?;
    let cities: Vec<City> = serde_json::from_str(&data)?;
    Ok(cities)
}