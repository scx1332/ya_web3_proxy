mod error;

extern crate core;

use actix_web::{web, App, HttpRequest, HttpServer, Responder, Scope};
use serde_json::json;
use std::fmt::Debug;
use structopt::StructOpt;
use crate::error::Web3ProxyError;

#[derive(Debug, StructOpt)]
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

pub async fn greet(_req: HttpRequest) -> impl Responder {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    return web::Json(json!({
        "name": "web3_proxy",
        "version": VERSION,
    }));
}

async fn main_internal() -> Result<(), Web3ProxyError> {
    if let Err(err) = dotenv::dotenv() {
        panic!("Error loading .env file: {}", err);
    }
    env_logger::init();
    let cli: CliOptions = CliOptions::from_args();

    let server = HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        let scope = Scope::new("api").route("/", web::get().to(greet));

        App::new().wrap(cors).service(scope)
    })
    .workers(cli.http_threads as usize)
    .bind((cli.http_addr.as_str(), cli.http_port))
    .expect("Cannot run server")
    .run();

    log::info!("http server starting on {}:{}", cli.http_addr, cli.http_port);

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
