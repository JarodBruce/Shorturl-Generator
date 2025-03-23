use axum::extract::Json;
use axum::http::{Method, header};
use axum::{
    Router,
    response::{IntoResponse, Redirect},
    routing::get,
};
use rand::Rng;
use rand::distributions::Alphanumeric;
use redis::Commands;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};

async fn redirect_url(
    axum::extract::Path(short_url): axum::extract::Path<String>,
    con: Arc<Mutex<redis::Connection>>, // 修正: Arc<Mutex<Connection>>を使用
) -> impl IntoResponse {
    let mut con = con.lock().unwrap(); // 接続をロック
    let keys: Vec<String> = con.keys("*").unwrap();
    println!("Keys: {:?}", keys);
    println!("Short URL: {}", short_url);

    for s in &keys {
        println!("Key: {:?}", s);
        let value: Option<String> = con.get(s).unwrap_or(None);
        println!("Value: {:?}", value);
    }

    if keys.contains(&short_url) {
        let long_url: String = con.get(&short_url).unwrap();
        println!("long_url: {}", long_url);
        Redirect::to(&long_url).into_response()
    } else {
        println!("URL is not valid");
        Redirect::to("https://www.google.com/404").into_response()
    }
}

async fn post_data(
    Json(payload): Json<Value>,
    con: Arc<Mutex<redis::Connection>>, // 修正: Arc<Mutex<Connection>>を使用
) -> impl IntoResponse {
    let mut con = con.lock().unwrap(); // 接続をロック
    let mut response_key = "Data in True".to_string();
    if let Value::Object(_map) = &payload {
        if let Some(long_url) = payload["long"].as_str() {
            println!("Received payload: {}", long_url);

            let mut rng = rand::thread_rng();
            let random_code: String = (&mut rng)
                .sample_iter(&Alphanumeric)
                .take(6)
                .map(char::from)
                .collect();

            let keys: Vec<String> = con.keys("*").unwrap();
            let mut read_json = true;
            for db_keys in &keys {
                println!("Key: {:?}", db_keys);
                let value: Option<String> = con.get(db_keys).unwrap_or(None);
                println!("Value: {:?}", value);
                if value == Some(long_url.to_string()) {
                    response_key = db_keys.clone();
                    read_json = false;
                    break;
                }
            }
            if read_json {
                con.set::<String, String, ()>(random_code.clone(), long_url.to_string())
                    .unwrap();
                response_key = random_code;
            }
        }
    } else {
        println!("Payload is not valid");
    }
    println!("Response key: {}", response_key);
    response_key.into_response()
}

#[tokio::main]
async fn main() {
    let client = redis::Client::open("redis://redis-server:6379/0").unwrap();
    let con = Arc::new(Mutex::new(client.get_connection().unwrap())); // 修正: ArcとMutexでラップ

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);

    let app = Router::new()
        .route(
            "/c/:short_url", // ここを修正
            get({
                let con = Arc::clone(&con);
                move |path| redirect_url(path, con.clone())
            }),
        )
        .route(
            "/c/post",
            axum::routing::post({
                let con = Arc::clone(&con);
                move |json| post_data(json, con.clone())
            }),
        )
        .layer(cors);

    axum::Server::bind(&"0.0.0.0:7001".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
