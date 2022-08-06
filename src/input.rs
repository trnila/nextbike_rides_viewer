use serde::Deserialize;

pub type StationId = u32;

#[derive(Deserialize, Debug)]
pub struct Bike {
    pub number: String,
}

#[derive(Deserialize, Debug)]
pub struct Place {
    pub uid: StationId,
    pub lat: f32,
    pub lng: f32,
    pub name: String,
    pub bike_list: Vec<Bike>,
}

#[derive(Deserialize, Debug)]
pub struct Cities {
    pub name: String,
    pub places: Vec<Place>,
}

#[derive(Deserialize, Debug)]
pub struct Countries {
    pub cities: Vec<Cities>,
}

#[derive(Deserialize, Debug)]
pub struct JSON {
    pub countries: Vec<Countries>,
}