mod error;

extern crate core;

use crate::error::Web3ProxyError;
use actix_web::http::StatusCode;
use actix_web::web::{Bytes, Data};
use actix_web::{
    web, App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer, Responder, Scope,
};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::net::windows::named_pipe::PipeEnd::Client;
use tokio::sync::Mutex;

#[derive(Debug, StructOpt, Clone)]
pub struct CliOptions {
    #[structopt(long = "http", help = "Enable http server")]
    pub http: bool,

    #[structopt(
        long = "http-threads",
        help = "Number of threads to use for the server",
        default_value = "2"
    )]
    pub http_threads: u64,

    #[structopt(
        long = "http-port",
        help = "Port number of the server",
        default_value = "8080"
    )]
    pub http_port: u16,

    #[structopt(
        long = "http-addr",
        help = "Bind address of the server",
        default_value = "127.0.0.1"
    )]
    pub http_addr: String,
}
macro_rules! return_on_error {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(err) => {
                return web::Json(json!({
                    "error": err.to_string()
                }))
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallInfo {
    pub id: i64,
    pub request: Option<String>,
    pub response: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyData {
    pub key: String,
    pub value: String,
    pub total_calls: u64,
    pub total_requests: u64,

    pub calls: Vec<CallInfo>,
}

pub struct SharedData {
    pub keys: HashMap<String, KeyData>,
}

pub struct ServerData {
    pub options: CliOptions,
    pub shared_data: Arc<Mutex<SharedData>>,
}

pub async fn get_calls(req: HttpRequest, server_data: Data<Box<ServerData>>) -> impl Responder {
    let key = req
        .match_info()
        .get("key")
        .ok_or("No key provided")
        .unwrap();
    let key_data = {
        let mut shared_data = server_data.shared_data.lock().await;
        let mut key_data = shared_data
            .keys
            .get_mut(key)
            .ok_or("Key not found - something went really wrong, beacue it should be here")
            .unwrap();
        key_data.clone()
    };
    web::Json(json!({ "key_data": key_data }))
}

pub async fn web3(
    req: HttpRequest,
    body: Bytes,
    server_data: Data<Box<ServerData>>,
) -> impl Responder {
    let key = req
        .match_info()
        .get("key")
        .ok_or("No key provided")
        .unwrap();

    let body_str = String::from_utf8(body.to_vec()).unwrap();
    let body_json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    println!("key: {}, body: {:?}", key, body_str);
    // Before call check.
    // Obtain lock and check conditions if we should call the function.
    {
        let mut shared_data = server_data.shared_data.lock().await;
        let mut key_data = shared_data.keys.get_mut(key);

        if let Some(key_data) = key_data {
            key_data.value = "test".to_string();
            key_data.total_requests += 1;
        } else {
            let key_data = KeyData {
                key: key.to_string(),
                value: "1".to_string(),
                total_calls: 0,
                total_requests: 0,
                calls: Vec::new(),
            };
            shared_data.keys.insert(key.to_string(), key_data);
        }
    }
    //do the long call here

    let client = awc::Client::new();
    let res = client
        .post("https://bor.golem.network")
        .send_json(&body_json)
        .await;
    log::debug!("res: {:?}", res);

    let mut response_body_str = None;
    let statusCode = match res {
        Ok(mut cr) => {
            let body_res = cr.body().await;
            match (body_res) {
                Ok(body) => match (String::from_utf8(body.to_vec())) {
                    Ok(body_str) => {
                        response_body_str = Some(body_str);
                        cr.status()
                    }
                    Err(err) => {
                        log::error!("Error getting body: {:?}", err);
                        StatusCode::from_u16(500).unwrap()
                    }
                },
                Err(e) => {
                    log::error!("Error getting body: {:?}", e);
                    StatusCode::from_u16(500).unwrap()
                }
            }
        }
        Err(err) => {
            log::error!("Error: {}", err);
            StatusCode::from_u16(500).unwrap()
        }
    };
    //After call update info
    {
        let mut shared_data = server_data.shared_data.lock().await;
        let mut key_data = shared_data
            .keys
            .get_mut(key)
            .ok_or("Key not found - something went really wrong, beacue it should be here")
            .unwrap();
        key_data.total_calls += 1;
        key_data.calls.push(CallInfo {
            id: 1,
            request: Some(body_str),
            response: response_body_str.clone(),
        });
    }
    if let Some(response_body_str) = response_body_str {
        HttpResponse::build(statusCode).body(response_body_str)
    } else {
        HttpResponse::build(statusCode).finish()
    }
}

pub async fn greet(_req: HttpRequest, server_data: Data<Box<ServerData>>) -> impl Responder {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    return web::Json(json!({
        "name": "web3_proxy",
        "server_info": format!("Listen: {}:{}", server_data.options.http_addr, server_data.options.http_port),
        "version": VERSION,
    }));
}

async fn main_internal() -> Result<(), Web3ProxyError> {
    if let Err(err) = dotenv::dotenv() {
        panic!("Error loading .env file: {}", err);
    }
    env_logger::init();
    let cli: CliOptions = CliOptions::from_args();

    let server_data = Data::new(Box::new(ServerData {
        options: cli.clone(),
        shared_data: Arc::new(Mutex::new(SharedData {
            keys: HashMap::new(),
        })),
    }));

    let server = HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        let scope = Scope::new("api")
            .app_data(server_data.clone())
            .route("/", web::get().to(greet))
            .route("/calls/{key}", web::get().to(get_calls))
            .route("/version", web::get().to(greet));

        App::new()
            .wrap(cors)
            .app_data(server_data.clone())
            .route("web3/{key}", web::get().to(web3))
            .route("web3/{key}", web::post().to(web3))
            .route("/", web::get().to(greet))
            .route("/api", web::get().to(greet))
            .service(scope)
    })
    .workers(cli.http_threads as usize)
    .bind((cli.http_addr.as_str(), cli.http_port))
    .expect("Cannot run server")
    .run();

    log::info!(
        "http server starting on {}:{}",
        cli.http_addr,
        cli.http_port
    );

    server.await.unwrap();

    println!("Hello, world!");
    Ok(())
}

#[actix_web::main]
async fn main() -> Result<(), Web3ProxyError> {
    match main_internal().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}
