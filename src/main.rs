mod handlers;
mod types;
mod structs;
mod utils;
mod state;

use dotenv;
// use std::sync::Arc;
use axum::{routing::{get, post}, Router};
use handlers::{
    fake_error, 
    cube_bits, 
    reindeer_strength, 
    reindeer_contest, 
    slice_query, 
    count_elf, 
    decode_header, 
    bake_recipe, 
    pokemon_weight, 
    pokemon_momentum, 
    serve_image, 
    read_pixels, 
    // save_packet, 
    // load_packet, 
    handle_ulids, 
    analize_ulids, 
    dumb_query, 
    reset_db, 
    insert_orders, 
    total_orders, 
    popular_order, 
    unsafe_render, safe_render, check_password, game_password, insert_regions, total_regions, handler_sockets
};
use state::{
    // IdStore, 
    PgState
};
// use tokio::sync::Mutex;
use sqlx::PgPool;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool
) -> shuttle_axum::ShuttleAxum {

    dotenv::dotenv().ok();
    sqlx::migrate!().run(&pool)
        .await
        .unwrap();

    // let state = IdStore {
    //     store: Arc::new(Mutex::new(Vec::new()))
    // };

    let pg_state = PgState { pool };

    let router = Router::new()
        .route("/-1/error", get(fake_error))
        .route("/1/*nums", get(cube_bits))
        .route("/4/strength", post(reindeer_strength))
        .route("/4/contest", post(reindeer_contest))
        .route("/5", post(slice_query))
        .route("/6", post(count_elf))
        .route("/7/decode", get(decode_header))
        .route("/7/bake", get(bake_recipe))
        .route("/8/weight/:id", get(pokemon_weight))
        .route("/8/drop/:id", get(pokemon_momentum))
        .route("/11/assets/:path", get(serve_image))
        .route("/11/red_pixels", post(read_pixels))
        // .route("/12/save/:packet_id", post(save_packet))
        // .route("/12/load/:packet_id", get(load_packet))
        .route("/12/ulids", post(handle_ulids))
        .route("/12/ulids/:day", post(analize_ulids))
        .route("/13/sql", get(dumb_query))
        .route("/13/orders/total", get(total_orders))
        .route("/13/orders/popular", get(popular_order))
        .route("/14/unsafe", post(unsafe_render))
        .route("/14/safe", post(safe_render))
        .route("/15/nice", post(check_password))
        .route("/15/game", post(game_password))
        .route("/18/reset", post(reset_db))
        .route("/18/orders", post(insert_orders))
        .route("/18/regions", post(insert_regions))
        .route("/18/regions/total", get(total_regions))
        .route("/19/ws/ping", get(handler_sockets))
        .with_state(pg_state);

    Ok(router.into())
}
