use std::{
    borrow::Cow,
    fmt::{self},
};

use serde::{de, Deserialize, Deserializer};

pub type StationId = u32;
pub type BikeId = u32;

#[derive(Deserialize, Debug)]
pub struct Bike {
    #[serde(deserialize_with = "from_str")]
    pub number: BikeId,
}

#[derive(Deserialize, Debug)]
pub struct Place<'a> {
    pub uid: StationId,
    pub lat: f32,
    pub lng: f32,
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    pub bike_list: Vec<Bike>,
}

#[derive(Deserialize, Debug)]
pub struct Cities<'a> {
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    #[serde(borrow)]
    pub places: Vec<Place<'a>>,
}

#[derive(Deserialize, Debug)]
pub struct Countries<'a> {
    #[serde(borrow)]
    pub cities: Vec<Cities<'a>>,
}

#[derive(Deserialize, Debug)]
pub struct JsonResponse<'a> {
    #[serde(borrow)]
    pub countries: Vec<Countries<'a>>,
}

fn from_str<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
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
