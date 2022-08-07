use std::{str::FromStr, fmt::{Display, self}};

use serde::{Deserialize, Deserializer, de};

pub type StationId = u32;

#[derive(Deserialize, Debug)]
pub struct Bike {
    #[serde(deserialize_with = "from_str")]
    pub number: u32,
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

fn from_str<'de, D>(deserializer: D) -> Result<u32, D::Error>
    where 
          D: Deserializer<'de>
{

    struct MyVisitor;
    impl<'de> de::Visitor<'de> for MyVisitor {
        type Value = u32;

        fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt.write_str("integer in string")
        }

        fn visit_str<E>(self, val: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match val.parse::<u32>() {
                    Ok(val) => Ok(val),
                    Err(_) => Err(E::custom("failed to parse integer")),
                }
            }
    }

    deserializer.deserialize_any(MyVisitor)
}