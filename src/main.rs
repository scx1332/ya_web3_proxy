mod error;

extern crate core;

use crate::error::*;
use actix_web::http::StatusCode;
use actix_web::web::{Bytes, Data};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder, Scope};
use serde::Serialize;
use serde_json::json;
use std::cmp::min;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;
use structopt::StructOpt;

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

    #[structopt(
        long = "target-addr",
        help = "Target address of the server",
        default_value = "http://polygongas.org:8545"
    )]
    pub target_addr: String,

    #[structopt(
        long = "queue-size",
        help = "How many historical requests to keep",
        default_value = "10000"
    )]
    pub request_queue_size: usize,
}
macro_rules! return_on_error_json {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(err) => {
                log::info!("Returning error: {}", err.to_string());
                return web::Json(json!({
                    "error": err.to_string()
                }))
            },
        }
    }
}
macro_rules! return_on_error_resp {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(err) => {
                log::info!("Returning error: {}", err);
                return HttpResponse::build(StatusCode::from_u16(500).unwrap())
                    .body(format!("{}", err));
            }
        }
    };
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedRequest {
    pub id: serde_json::Value,
    pub method: String,
    pub params: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MethodInfo {
    pub id: String,
    pub method: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub response_time: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallInfo {
    pub request: Option<String>,
    pub response: Option<String>,

    pub parsed_request: Vec<ParsedRequest>,
    pub date: chrono::DateTime<chrono::Utc>,
    pub response_time: f64,
}

pub fn parse_request(
    parsed_body: &serde_json::Value,
) -> Result<Vec<ParsedRequest>, Web3ProxyError> {
    let mut parsed_requests = Vec::new();

    if parsed_body.is_array() {
    } else {
        let jsonrpc = parsed_body["jsonrpc"]
            .as_str()
            .ok_or(err_custom_create!("jsonrpc field is missing"))?;
        if jsonrpc != "2.0" {
            return Err(err_custom_create!("jsonrpc field is not 2.0"));
        }
        let rpc_id = parsed_body["id"].clone();
        let method = parsed_body["method"]
            .as_str()
            .ok_or(err_custom_create!("method field is missing"))?;
        let params = parsed_body["params"]
            .as_array()
            .ok_or(err_custom_create!("params field is missing"))?;

        parsed_requests.push(ParsedRequest {
            id: rpc_id,
            method: method.to_string(),
            params: params.clone(),
        });
    }
    Ok(parsed_requests)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyData {
    pub key: String,
    pub value: String,
    pub total_calls: u64,
    pub total_requests: u64,

    pub calls: VecDeque<CallInfo>,
}

pub struct SharedData {
    pub keys: HashMap<String, KeyData>,
}

pub struct ServerData {
    pub options: CliOptions,
    pub shared_data: Arc<Mutex<SharedData>>,
}

pub async fn get_calls(req: HttpRequest, server_data: Data<Box<ServerData>>) -> impl Responder {
    let limit = req
        .match_info()
        .get("limit")
        .map(|v| v.parse::<usize>().unwrap_or(0));
    let key = req
        .match_info()
        .get("key")
        .ok_or("No key provided")
        .unwrap();
    let mut shared_data = server_data.shared_data.lock().await;
    let key_data = return_on_error_json!(shared_data.keys.get_mut(key).ok_or("Key not found"));
    let limit = min(limit.unwrap_or(key_data.calls.len()), key_data.calls.len());
    let calls: Vec<CallInfo> = key_data.calls.iter().rev().take(limit).cloned().collect();

    web::Json(json!({ "calls": calls }))
}

pub async fn get_methods(req: HttpRequest, server_data: Data<Box<ServerData>>) -> impl Responder {
    let limit = req
        .match_info()
        .get("limit")
        .map(|v| v.parse::<usize>().unwrap_or(0));
    let key = req
        .match_info()
        .get("key")
        .ok_or("No key provided")
        .unwrap();
    let mut shared_data = server_data.shared_data.lock().await;
    let key_data = return_on_error_json!(shared_data.keys.get_mut(key).ok_or("Key not found"));
    let limit = min(limit.unwrap_or(key_data.calls.len()), key_data.calls.len());
    let calls: Vec<CallInfo> = key_data.calls.iter().rev().take(limit).cloned().collect();

    let methods = calls
        .iter()
        .flat_map(|call| {
            call.parsed_request
                .iter()
                .map(|req| MethodInfo {
                    id: req.id.to_string(),
                    method: req.method.clone(),
                    date: call.date,
                    response_time: call.response_time,
                })
                .collect::<Vec<MethodInfo>>()
        })
        .collect::<Vec<MethodInfo>>();

    web::Json(json!({ "methods": methods }))
}

pub async fn web3(
    req: HttpRequest,
    body: Bytes,
    server_data: Data<Box<ServerData>>,
) -> impl Responder {
    let key = return_on_error_resp!(req.match_info().get("key").ok_or("No key provided"));

    let body_str = return_on_error_resp!(String::from_utf8(body.to_vec()));
    let body_json: serde_json::Value = return_on_error_resp!(serde_json::from_str(&body_str));
    let parsed_request = parse_request(&body_json).unwrap_or(vec![]);

    log::info!(
        "key: {}, method: {:?}",
        key,
        parsed_request.get(0).map(|x| x.method.clone())
    );
    // Before call check.
    // Obtain lock and check conditions if we should call the function.
    {
        let mut shared_data = server_data.shared_data.lock().await;
        let key_data = shared_data.keys.get_mut(key);

        if let Some(key_data) = key_data {
            key_data.value = "test".to_string();
            key_data.total_requests += 1;
        } else {
            let key_data = KeyData {
                key: key.to_string(),
                value: "1".to_string(),
                total_calls: 0,
                total_requests: 0,
                calls: VecDeque::new(),
            };
            shared_data.keys.insert(key.to_string(), key_data);
        }
    }
    //do the long call here

    let call_date = chrono::Utc::now();
    let start = Instant::now();

    let client = awc::Client::new();
    let res = client
        .post(&server_data.options.target_addr)
        .send_json(&body_json)
        .await;
    log::debug!("res: {:?}", res);

    let mut response_body_str = None;
    let status_code = match res {
        Ok(mut cr) => {
            let body_res = cr.body().await;
            match body_res {
                Ok(body) => match String::from_utf8(body.to_vec()) {
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
    let finish = Instant::now();
    //After call update info
    {
        let call_info = CallInfo {
            date: call_date,
            request: Some(body_str),
            parsed_request,
            response: response_body_str.clone(),
            response_time: (finish - start).as_secs_f64(),
        };

        let mut shared_data = server_data.shared_data.lock().await;
        let mut key_data = return_on_error_resp!(shared_data
            .keys
            .get_mut(key)
            .ok_or("Key not found - something went really wrong, beacue it should be here"));
        key_data.total_calls += 1;

        key_data.calls.push_back(call_info);
        if key_data.calls.len() > server_data.options.request_queue_size {
            key_data.calls.pop_front();
        }
    }
    if let Some(response_body_str) = response_body_str {
        HttpResponse::build(status_code).body(response_body_str)
    } else {
        HttpResponse::build(status_code).finish()
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

pub async fn config(_req: HttpRequest, server_data: Data<Box<ServerData>>) -> impl Responder {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    web::Json(
        json!({"config": {"version": VERSION, "request_queue_size": server_data.options.request_queue_size}}),
    )
}

async fn main_internal() -> Result<(), Web3ProxyError> {
    if let Err(err) = dotenv::dotenv() {
        panic!("Error loading .env file: {err}");
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
            .route("/config", web::get().to(config))
            .route("/calls/{key}", web::get().to(get_calls))
            .route("/calls/{key}/{limit}", web::get().to(get_calls))
            .route("/methods/{key}", web::get().to(get_methods))
            .route("/methods/{key}/{limit}", web::get().to(get_methods))
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
            eprintln!("Error: {e}");
            Err(e)
        }
    }
}
