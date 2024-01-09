use axum::http::HeaderMap;
use base64::{engine::general_purpose, Engine as _};

pub fn extract_recipe(headers: HeaderMap) -> Option<String> {
    let cookie_header = match headers.get("Cookie") {
        Some(c) => c,
        None => return None
    };

    let cookie_str = match cookie_header.to_str() {
        Ok(str) => str,
        Err(_) => return None
    };

    let split_cookie: Vec<&str> = cookie_str.split("=").collect();

    if split_cookie.len() < 2 {
        return None;
    }

    let recipe_encoded = split_cookie[1];

    let bytes = match general_purpose::STANDARD_NO_PAD.decode(recipe_encoded) {
        Ok(bytes) => bytes,
        Err(_) => return None
    };

    let string = match String::from_utf8(bytes) {
        Ok(string) => string,
        Err(_) => return None
    };

    Some(string)
}

pub fn is_lsb_1(ulid_bytes: &[u8; 16]) -> bool {
    ulid_bytes[15] & 1 == 1
}
