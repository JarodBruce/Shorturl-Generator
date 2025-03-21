use axum::{
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde_json::{Value, json};
use std::fs::File;
use std::io::{Read, Write};
use axum::extract::Json;

async fn redirect_url(axum::extract::Path(short_url): axum::extract::Path<String>) -> impl IntoResponse {
    // JSONファイルを読み込む
    let mut file = match File::open("data.json") {
        Ok(file) => file,
        Err(err) => {
        eprintln!("Failed to open file: {}", err);
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to open file".to_string(),
        )
            .into_response();
        }
    };

    let mut contents = String::new();
    if let Err(err) = file.read_to_string(&mut contents) {
        eprintln!("Failed to read file: {}", err);
        return (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "Failed to read file".to_string(),
        )
        .into_response();
    }

    let json_data: Value = match serde_json::from_str(&contents) {
        Ok(data) => data,
        Err(err) => {
        eprintln!("Failed to parse JSON: {}", err);
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to parse JSON".to_string(),
        )
            .into_response();
        }
    };
    let mut keys: Vec<String> = Vec::new();
    if let Value::Object(map) = &json_data {
        keys = map.keys().cloned().collect();
    }

    if keys.contains(&short_url) {
        let long_url = json_data[&short_url][0].as_str().unwrap();
        Redirect::to(&long_url).into_response()
    } else {
        println!("URL is not valid");
        Redirect::to("https://www.google.com/404").into_response()
    }
}

async fn post_data(Json(payload): Json<Value>) -> impl IntoResponse {
    let mut response_key = "Data in True".to_string();
    if let Value::Object(map) = &payload {
        if map.contains_key("long") {

            let mut rng = rand::thread_rng();
            let random_code: String = (&mut rng)
                .sample_iter(&Alphanumeric)
                .take(6)
                .map(char::from)
                .collect();

            // JSONファイルを読み込む
            let mut file = match File::open("data.json") {
                Ok(file) => file,
                Err(err) => {
                    eprintln!("Failed to open file: {}", err);
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to open file".to_string(),
                    )
                        .into_response();
                }
            };

            let mut contents = String::new();
            if let Err(err) = file.read_to_string(&mut contents) {
                eprintln!("Failed to read file: {}", err);
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to read file".to_string(),
                )
                    .into_response();
            }

            let mut json_data: Value = match serde_json::from_str(&contents) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("Failed to parse JSON: {}", err);
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to parse JSON".to_string(),
                    )
                        .into_response();
                }
            };

            let mut read_json = true;
            if let Value::Object(json_map) = &mut json_data {
                // 同じデータが存在すれば書き込みをスキップし、キーを出力
                if let Some(existing_key) = json_map.iter().find_map(|(key, value)| {
                    if value.as_array()
                        .map(|arr| arr.iter().filter_map(|item| item.as_str().map(String::from)).collect::<Vec<String>>())
                        == Some(map.values().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<String>>())
                    {
                        Some(key.clone())
                    } else {
                        None
                    }
                }) {
                    response_key = existing_key.as_str().to_string();
                    println!("{}", existing_key);
                    read_json = false;
                }
            }

            // JSONデータを書き換え
            if read_json {
                json_data[random_code.clone()] = json!(map.values().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<String>>());

                // JSONファイルに書き込む
                let mut file = match File::create("data.json") {
                    Ok(file) => file,
                    Err(err) => {
                        eprintln!("Failed to create file: {}", err);
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to create file".to_string(),
                        )
                            .into_response();
                    }
                };

                if let Err(err) = file.write_all(json_data.to_string().as_bytes()) {
                    eprintln!("Failed to write to file: {}", err);
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to write to file".to_string(),
                    )
                        .into_response();
                } else {
                    response_key =  random_code.to_string().clone();
                }
            }
        } else {
            println!("Data is not valid");
        }
    } else {
        println!("Payload is not valid");
    }
    println!("Response key: {}", response_key);
    response_key.into_response()
}

async fn handler() -> impl IntoResponse {
    axum::response::Html(
        r#"
        <form action="http://localhost:3000/post" method="post">
            <label for="long">Enter URL:</label>
            <input type="text" id="long" name="long" value="">
            <input type="submit" value="Submit" onclick="submitJson(event)">
        </form>
        <script>
            function submitJson(event) {
                event.preventDefault();
                const form = event.target.form;
                const longUrl = form.long.value;
                fetch(form.action, {
                    method: form.method,
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({ long: longUrl }),
                }).then(response => response.text())
                  .then(data => alert('Response: ' + data))
                  .catch(error => console.error('Error:', error));
            }
        </script>
        "#
    )
}
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/:short_url", get(redirect_url))
        .route("/post", axum::routing::post(post_data))
        .route("/post_server", get(handler));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
