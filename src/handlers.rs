use std::io::Cursor;
use image::io::Reader as ImageReader;
use regex::Regex;
use sqlx::types::JsonValue;
use ulid::Ulid;
use chrono::{Utc, TimeZone, Datelike};
use sha2::{Sha256, Digest};

use axum::{
    extract::{
        Path, 
        Query,
        Multipart, 
        State,
        ws::{WebSocketUpgrade, WebSocket, Message},
    }, 
    Json, 
    http::HeaderMap, response::{IntoResponse, Response},
    http::StatusCode,
};
use serde_json::{json, Value};
use tokio::fs;
use uuid::Uuid;

use crate::{
    types::{
        ApiResponse, 
        Pagination
    }, 
    structs::{
        Reindeer, 
        ReindeerContest, 
        ContestResult, UlidCalc, Order, RenderContent, Password, Region, RegionTotal,
    }, 
    utils::{extract_recipe, is_lsb_1}, state::{IdStore, PacketId, PgState},
};

pub async fn fake_error() -> ApiResponse {
    ApiResponse::ServerError
}

pub async fn cube_bits(
    Path(nums): Path<String>
) -> ApiResponse {
    let strings: Vec<&str> = nums.split("/").collect();

    let mut xor_accumulator: i64 = 0;

    for string in strings {
        match string.parse::<i64>() {
            Ok(number) => xor_accumulator = xor_accumulator ^ number,
            Err(_e) => return ApiResponse::ServerError
        }
    }

    ApiResponse::Integer(xor_accumulator.pow(3))
}

pub async fn reindeer_strength(
    Json(reindeers): Json<Vec<Reindeer>>
) -> ApiResponse {
    let mut sum: u64 = 0;

    for reindeer in reindeers {
        sum += reindeer.strength;
    }

    ApiResponse::Unsigned(sum)
}

pub async fn reindeer_contest(
    Json(contest_data): Json<Vec<ReindeerContest>>
) -> ApiResponse {
    let result = ContestResult::get_result(contest_data);

    match result {
        Some(contest_result) => ApiResponse::JsonValue(json!(contest_result)),
        None => ApiResponse::ServerError
    }
}

pub async fn slice_query(
    Query(pagination): Query<Pagination>,
    Json(names): Json<Vec<String>>
) -> ApiResponse {
    let offset: usize = pagination.offset.unwrap_or(0);

    let limit: usize = pagination.limit.unwrap_or_else(|| names.len() - offset);

    let end_index = std::cmp::min(offset+limit, names.len());

    let before_split = &names[offset..end_index];

    match pagination.split {
        Some(split) => {
            let result: Vec<Vec<String>> = before_split.chunks(split).map(|chunk| chunk.to_vec()).collect();
            ApiResponse::JsonValue(json!(result))
        },
        None => ApiResponse::JsonValue(json!(before_split))
    }
}

pub async fn count_elf(
    string: String
) -> ApiResponse {
    let elf_count = string.matches("elf").count();
    let elf_on_shelf_count = string.matches("elf on a shelf").count();
    let shelf_count = string.matches("shelf").count();

    ApiResponse::JsonValue(json!({
        "elf": elf_count,
        "elf on a shelf": elf_on_shelf_count,
        "shelf with no elf on it": shelf_count - elf_on_shelf_count
    }))
}

pub async fn decode_header(
    headers: HeaderMap
) -> ApiResponse {

    let string = match extract_recipe(headers) {
        Some(r) => r,
        None => return ApiResponse::ServerError
    };

    ApiResponse::String(string)
}

pub async fn bake_recipe(
    headers: HeaderMap
) -> ApiResponse {
    let string = match extract_recipe(headers) {
        Some(r) => r,
        None => return ApiResponse::ServerError
    };

    println!("{}", string);

    let mut full_recipe: Value = serde_json::from_str(&string).unwrap();
    
    let full_recipe_object = full_recipe.as_object_mut().unwrap();
    let full_recipe_clone = full_recipe_object.clone();

    let recipe = full_recipe_clone.get("recipe").unwrap().as_object().unwrap();
    let pantry =  full_recipe_object.get_mut("pantry").unwrap().as_object_mut().unwrap();
    
    let mut ratios: Vec<u64> = Vec::new();

    for (key, value) in pantry.iter() {
        let recipe_value = recipe.get(key).unwrap().as_u64().unwrap();
        let ratio = value.as_u64().unwrap() / recipe_value;
        ratios.push(ratio)
    }

    let min_ratio = ratios.iter().min().unwrap();

    for (key, value) in pantry.iter_mut() {
        let recipe_value = recipe.get(key).unwrap().as_u64().unwrap();
        *value = Value::from(value.as_u64().unwrap() - (recipe_value * min_ratio));

    }

    println!("{}", min_ratio);
    println!("{:?}", pantry);

    ApiResponse::JsonValue(json!({
        "cookies": min_ratio,
        "pantry": pantry
    }))
}

pub async fn pokemon_weight(
    Path(id): Path<u64>
) -> ApiResponse {
    println!("{}", id);
    let body = match reqwest::get(format!("https://pokeapi.co/api/v2/pokemon/{}", id)).await.unwrap().text().await {
        Ok(r) => r,
        Err(_) => return ApiResponse::ServerError
    };

    let pokemon: Value = serde_json::from_str(&body).unwrap();

    let weight = pokemon.get("weight").unwrap().as_f64().unwrap();

    let weight = weight / 10.0;

    ApiResponse::String(weight.to_string())
}

pub async fn pokemon_momentum(
    Path(id): Path<u64>
) -> ApiResponse {
    let body = match reqwest::get(format!("https://pokeapi.co/api/v2/pokemon/{}", id)).await.unwrap().text().await {
        Ok(r) => r,
        Err(_) => return ApiResponse::ServerError
    };

    let pokemon: Value = serde_json::from_str(&body).unwrap();
    let weight = pokemon.get("weight").unwrap().as_f64().unwrap();
    let weight = weight / 10.0;
    let velocity = (2.0_f64 * 9.825 * 10.0).sqrt();
    let momentum = velocity * weight;

    ApiResponse::String(momentum.to_string())
}

pub async fn serve_image(
    Path(path): Path<String>
) -> ApiResponse {
    match fs::read(format!("/home/sebatustra/rust/shuttle-christmas/assets/{}", path)).await {
        Ok(image_data) => ApiResponse::PngImage(image_data),
        Err(e) => {
            println!("{:?}", e);
            ApiResponse::ServerError
        }
    }
}

pub async fn read_pixels(
    mut multipart: Multipart
) -> ApiResponse {
    match multipart.next_field().await {
        Ok(field) => {
            match field {
                Some(field) => {
                    let data = field.bytes().await.unwrap().to_vec();
                    let image = ImageReader::new(Cursor::new(data)).with_guessed_format().unwrap().decode().unwrap();
                    let rgb = image.as_rgb8().unwrap();
                    let mut counter: u64 = 0;
                    for pixel in rgb.pixels() {
                        let color = pixel.0;
                        if u16::from(color[0]) > u16::from(color[1]) + u16::from(color[2]) {
                            counter += 1;
                        }
                    }

                    return ApiResponse::Unsigned(counter)
                },
                None => return ApiResponse::ServerError
            }
        },
        Err(e) => {
            println!("{:?}", e);
            ApiResponse::ServerError
        }
    }
}

// pub async fn save_packet(
//     Path(packet_id): Path<String>,
//     State(store): State<IdStore>
// ) -> ApiResponse {
//     let mut packet_store = store.store.lock().await;
//     match packet_store.iter_mut().find(|el| el.packet_id == packet_id) {
//         Some(packet) => packet.timestamp = Instant::now(),
//         None => {
//             packet_store.push(PacketId::new(packet_id))
//         }
//     }

//     ApiResponse::Ok
// }

// pub async fn load_packet(
//     Path(packet_id): Path<String>,
//     State(store): State<IdStore>
// ) -> ApiResponse {
//     match store.store.lock().await.iter().find(|el| el.packet_id == packet_id) {
//         Some(packet) => {
//             ApiResponse::String(packet.timestamp.elapsed().as_secs().to_string())
//         },
//         None => ApiResponse::ServerError
//     }
// }

pub async fn handle_ulids(
    Json(strings): Json<Vec<String>>
) -> ApiResponse {

    let ulids: Vec<Ulid> = strings.iter().map(|el| {
        Ulid::from_string(el).unwrap()
    }).collect();

    let mut uuids: Vec<Uuid> = ulids.iter().map(|ulid| {
        let bytes = ulid.to_bytes();
        let uuid = Uuid::from_bytes(bytes);
        uuid
    }).collect();

    uuids.reverse();

    ApiResponse::JsonValue(json!(uuids))
}

pub async fn analize_ulids(
    Path(day): Path<u32>,
    Json(strings): Json<Vec<String>>
) -> ApiResponse {

    let ulids: Vec<Ulid> = strings.iter().map(|el| {
        Ulid::from_string(el).unwrap()
    }).collect();

    let christmas_eve = Utc.with_ymd_and_hms(2000, 12, 24, 0 ,0,0).unwrap();
    let now = Utc::now();

    let mut christmas_counter: u64 = 0;
    let mut weekday_counter: u64 = 0;
    let mut future_counter: u64 = 0;
    let mut lsb_counter: u64 = 0;

    for ulid in ulids {
        let ulid_date = Utc.timestamp_millis_opt(ulid.timestamp_ms().try_into().unwrap()).unwrap();
        let ulid_bytes = ulid.to_bytes();
        if (ulid_date.day(), ulid_date.month()) == (christmas_eve.day(), christmas_eve.month()) {
            christmas_counter += 1;
        }
        if ulid_date.weekday().num_days_from_monday() == day {
            weekday_counter += 1;
        }
        if ulid_date.date_naive() > now.date_naive() {
            future_counter += 1;
        }
        if is_lsb_1(&ulid_bytes) {
            lsb_counter += 1;
        }
    }

    let answer = UlidCalc::new(
        christmas_counter, 
        weekday_counter, 
        future_counter, 
        lsb_counter
    );

    ApiResponse::Ulid(answer)
}


pub async fn unsafe_render(
    Json(content): Json<RenderContent>
) -> ApiResponse {
    ApiResponse::HtmlRaw(content.create_html())
}

pub async fn safe_render(
    Json(content): Json<RenderContent>
) -> ApiResponse {
    ApiResponse::HtmlRaw(content.create_safe_html())
}

pub async fn check_password(
    Json(password): Json<Password>
) -> ApiResponse {
    let input_lower = password.input;
    
    let has_three_vowels = input_lower.chars()
    .filter(|c| "aeiouy".contains(*c))
    .count() >= 3;

let has_double_letter = input_lower.as_bytes()
.windows(2)
.any(|w| w[0] == w[1]);

let no_forbidden_substrings = !["ab", "cd", "pq", "xy"]
.iter()
.any(|&s| input_lower.contains(s));

if has_three_vowels && has_double_letter && no_forbidden_substrings {
    ApiResponse::JsonValue(json!({"result": "nice"}))
} else {
    ApiResponse::RequestErrorAndJson(json!({"result": "naughty"}))
}
}

pub async fn game_password(
    Json(password): Json<Password>
) -> impl IntoResponse {
    let content_str: &str = &password.input;
    
    //1
    if content_str.len() < 8 {
        return StatusCode::BAD_REQUEST
    }
    
    //2
    if !Regex::new(r"[A-Z]").unwrap().is_match(content_str) ||
    !Regex::new(r"[a-z]").unwrap().is_match(content_str) ||
    !Regex::new(r"\d").unwrap().is_match(content_str) {
        return StatusCode::BAD_REQUEST
    }
    
    //3
    if Regex::new(r"\d").unwrap().find_iter(content_str).count() < 5 {
        return StatusCode::BAD_REQUEST
    }
    
    //4
    let sum: u32 = Regex::new(r"\d+").unwrap().find_iter(content_str)
    .map(|m| m.as_str().parse::<u32>().unwrap())
    .sum();
if sum != 2023 {
    return StatusCode::BAD_REQUEST
}

//5
if !Regex::new(r"(.)\1").unwrap().is_match(content_str) {
    return StatusCode::NOT_ACCEPTABLE
}

//6
if !content_str.chars().any(|c| ('\u{2980}'..='\u{2BFF}').contains(&c)) {
    return StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS
}

//7
if !content_str.chars().any(|c| c as u32 > 0xFFFF) {
    return StatusCode::RANGE_NOT_SATISFIABLE
}


//8
let emoji_regex = Regex::new(r"(?:[\u{1F600}-\u{1F64F}\u{1F300}-\u{1F5FF}\u{1F680}-\u{1F6FF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}]+)").unwrap();
if !emoji_regex.is_match(content_str) {
    return StatusCode::UPGRADE_REQUIRED
}


//9
let hash = Sha256::digest(content_str.as_bytes());
if format!("{:x}", hash).chars().last().unwrap() != 'a' {
    return StatusCode::IM_A_TEAPOT
}

StatusCode::OK
}

pub async fn dumb_query(
    State(state): State<PgState>
) -> ApiResponse {
    match sqlx::query!("SELECT 20231213 AS value;")
        .fetch_one(&state.pool)
        .await {
            Ok(result) => {
                ApiResponse::String(result.value.unwrap().to_string())
            },
            Err(e) => {
                println!("{}", e);
                ApiResponse::ServerError
            }
        }
}

pub async fn reset_db(
    State(state): State<PgState>
) -> ApiResponse {

    match sqlx::query!("DROP TABLE IF EXISTS regions;").execute(&state.pool).await {
        Ok(_) => {
            println!("dropped previous regions table");
            ()
        },
        Err(e) => {
            println!("{:?}", e);
            return ApiResponse::ServerError
        }
    }

    match sqlx::query!("DROP TABLE IF EXISTS orders;").execute(&state.pool).await {
        Ok(_) => {
            println!("dropped previous orders table");
            ()
        },
        Err(e) => {
            println!("{:?}", e);
            return ApiResponse::ServerError
        }
    }

    match sqlx::query!("
        CREATE TABLE regions (
        id INT PRIMARY KEY,
        name VARCHAR(50)
        );
    ").execute(&state.pool).await {
        Ok(_) => {
            println!("created new regions table");
            ()
        },
        Err(e) => {
            println!("{:?}", e);
            return ApiResponse::ServerError
        }
    }

    match sqlx::query!("
        CREATE TABLE orders (
        id INT PRIMARY KEY,
        region_id INT,
        gift_name VARCHAR(50),
        quantity INT
        );
    ").execute(& state.pool).await {
        Ok(_) => {
            println!("created new orders table");
            return ApiResponse::Ok
        }
        Err(e) => {
            println!("{:?}", e);
            return ApiResponse::ServerError
        }
    }
}

pub async fn insert_orders(
    State(state): State<PgState>,
    Json(orders): Json<Vec<Order>>
) -> ApiResponse {

    for order in orders {
        match sqlx::query(
            "INSERT INTO orders (id, region_id, gift_name, quantity) VALUES ($1, $2, $3, $4);", 
        )
        .bind(order.id)
        .bind(order.region_id)
        .bind(order.gift_name)
        .bind(order.quantity)
        .execute(&state.pool)
        .await {
            Ok(_) => {
                println!("inserted order with id {}.", order.id);
                ()
            },
            Err(e) => {
                println!("{:?}", e);
                return ApiResponse::ServerError
            }
        }
    }

    return ApiResponse::Ok
}

pub async fn insert_regions(
    State(state): State<PgState>,
    Json(regions): Json<Vec<Region>>
) -> ApiResponse {
    for region in regions {
        match sqlx::query(
            "INSERT INTO regions (id, name) VALUES ($1, $2);", 
        )
        .bind(region.id)
        .bind(region.name)
        .execute(&state.pool)
        .await {
            Ok(_) => {
                println!("inserted region with id {}.", region.id);
                ()
            },
            Err(e) => {
                println!("{:?}", e);
                return ApiResponse::ServerError
            }
        }
    }

    return ApiResponse::Ok
}

pub async fn total_regions(
    State(state): State<PgState>
) -> ApiResponse {
    match sqlx::query_as::<_, RegionTotal>("
        SELECT name AS region, SUM(quantity) AS total
        FROM regions 
        JOIN orders 
            ON regions.id = orders.region_id
        GROUP BY name
        HAVING COUNT(orders.id) > 0
        ORDER BY name ASC;
    ")
    .fetch_all(&state.pool)
    .await {
        Ok(result) => ApiResponse::JsonValue(json!(result)),
        Err(e) => {
            println!("{}", e);
            ApiResponse::ServerError
        }
    }
}

pub async fn total_orders(
    State(state): State<PgState>
) -> ApiResponse {
    match sqlx::query_scalar::<_, i64>(
        "SELECT SUM(quantity) FROM orders;"
    )
    .fetch_one(&state.pool)
    .await {
        Ok(result) => ApiResponse::JsonValue(json!({"total": result})),
        Err(e) => {
            println!("{}", e);
            ApiResponse::ServerError
        }
    }
}

pub async fn popular_order(
    State(state): State<PgState>
) -> ApiResponse {
    match sqlx::query_scalar::<_, String>(
    "SELECT gift_name from orders GROUP BY gift_name ORDER BY SUM(quantity) DESC LIMIT 1;"
    )
    .fetch_optional(&state.pool)
    .await {
        Ok(result) => {
            match result {
                Some(order_name) => ApiResponse::JsonValue(json!({"popular": order_name})),
                None => ApiResponse::JsonValue(json!({"popular": JsonValue::Null}))
            }
        },
        Err(e) => {
            println!("{:?}", e);
            ApiResponse::ServerError
        }
    }
}

pub async fn handler_sockets(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            match &msg {
                Message::Text(text) => {
                    if text.eq("serve") {
                        socket.send("hola".into());
                        msg
                    } else {
                        return
                    }
                },
                _ => return,
            }
        } else {
            return
        };

        if socket.send(msg).await.is_err() {
            return;
        }
    }
}   
