use std::{str::FromStr, fmt};

use axum::{response::{Response, IntoResponse}, Json, http::StatusCode};
use serde::{Deserialize, Deserializer, de};
use serde_json::Value;

use crate::structs::UlidCalc;

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub offset: Option<usize>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub limit: Option<usize>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub split: Option<usize>
}

pub enum ApiResponse {
    Ok,
    ServerError,
    RequestErrorAndJson(Value),
    JsonValue(Value),
    Integer(i64),
    Unsigned(u64),
    String(String),
    PngImage(Vec<u8>),
    Ulid(UlidCalc),
    HtmlRaw(String)
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        match self {
            ApiResponse::Ok => (StatusCode::OK).into_response(),
            ApiResponse::ServerError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            ApiResponse::JsonValue(data) => (StatusCode::OK, Json(data)).into_response(),
            ApiResponse::Integer(number) => (StatusCode::OK, number.to_string()).into_response(),
            ApiResponse::Unsigned(number) => (StatusCode::OK, number.to_string()).into_response(),
            ApiResponse::String(string) => (StatusCode::OK, string.to_string()).into_response(),
            ApiResponse::PngImage(data) => (StatusCode::OK, [("Content-Type", "image/png")], data).into_response(),
            ApiResponse::Ulid(data) => (StatusCode::OK, Json(data)).into_response(),
            ApiResponse::HtmlRaw(data) => (StatusCode::OK, [("Content-Type", "text/html")], data).into_response(),
            ApiResponse::RequestErrorAndJson(data) => (StatusCode::BAD_REQUEST, Json(data)).into_response(),

        }
    }
}