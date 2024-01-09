use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Deserialize)]
pub struct Reindeer {
    pub name: String,
    pub strength: u64
}

#[derive(Deserialize)]
pub struct ReindeerContest {
    pub name: String,
    pub strength: u64,
    pub speed: f64,
    pub height: u64,
    pub antler_width: u64,
    pub snow_magic_power: u64,
    pub favorite_food: String,
    #[serde(rename = "cAnD13s_3ATeN-yesT3rdAy")]
    pub candies_eaten_yesterday: u64
}

#[derive(Deserialize, Serialize)]
pub struct ContestResult {
    fastest: String,
    tallest: String,
    magician: String,
    consumer: String
}

impl ContestResult {
    pub fn get_result(reindeers: Vec<ReindeerContest>) -> Option<Self> {
        let fastest= reindeers.iter().max_by(|a, b| a.speed.partial_cmp(&b.speed).unwrap())?;
        let tallest = reindeers.iter().max_by_key(|reindeer| reindeer.height)?;
        let magician = reindeers.iter().max_by_key(|reindeer| reindeer.snow_magic_power)?;
        let consumer = reindeers.iter().max_by_key(|reindeer| reindeer.candies_eaten_yesterday)?;

        return Some(ContestResult {
            fastest: format!("Speeding past the finish line with a strength of {} is {}", fastest.strength, fastest.name),
            tallest: format!("{} is standing tall with his {} cm wide antlers", tallest.name, tallest.antler_width),
            magician: format!("{} could blast you away with a snow magic power of {}", magician.name, magician.snow_magic_power),
            consumer: format!("{} ate lots of candies, but also some {}", consumer.name, consumer.favorite_food)
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Recipe {
    pub flour: u64,
    pub sugar: u64,
    pub butter: u64,
    #[serde(rename = "baking powder")]
    pub baking_powder: u64,
    #[serde(rename = "chocolate chips")]
    pub chocolate_chips: u64
}

#[derive(Serialize, Deserialize)]
pub struct FullRecipe {
    pub recipe: Recipe,
    pub pantry: Recipe
}

#[derive(Serialize)]
pub struct UlidCalc {
    #[serde(rename = "christmas eve")]
    pub christmas_eve: u64,
    pub weekday: u64,
    #[serde(rename = "in the future")]
    pub in_the_future: u64,
    #[serde(rename = "LSB is 1")]
    pub lsb_is_1: u64
}

impl UlidCalc {
    pub fn new(
        christmas_counter: u64,
        weekday_counter: u64,
        future_counter: u64,
        lsb_counter: u64
    ) -> Self {
        UlidCalc {
            christmas_eve: christmas_counter,
            weekday: weekday_counter,
            in_the_future: future_counter,
            lsb_is_1: lsb_counter
        }
    }
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct Order {
    pub id: i32,
    pub region_id: i32,
    pub gift_name: String,
    pub quantity: i32
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct Region {
    pub id: i32,
    pub name: String
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct RegionTotal {
    pub region: String,
    pub total: i64
}

#[derive(Debug, Deserialize)]
pub struct RenderContent{
    pub content: String
}

impl RenderContent {
    pub fn create_html(self) -> String {
        format!(
"<html>
  <head>
    <title>CCH23 Day 14</title>
  </head>
  <body>
    {}
  </body>
</html>", self.content
        )
    }
    pub fn create_safe_html(self) -> String {
        let mut sanitized_content= html_escape::encode_safe(&self.content).to_string();
        sanitized_content = sanitized_content.replace("&#x2F;", "/");

        format!(
"<html>
  <head>
    <title>CCH23 Day 14</title>
  </head>
  <body>
    {}
  </body>
</html>", sanitized_content
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct Password{
    pub input: String
}