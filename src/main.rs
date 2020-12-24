use std::sync::Arc;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;



mod process;
use process::Request;

#[derive(Debug, Deserialize)]
struct Config{
    pub port: u16,
    pub temp_path: String
}

#[get("/")]
async fn make(data: web::Query<Request>, state: web::Data<Arc<Config>>) -> impl Responder {
    let config = &*state;
    match data.response(&config.temp_path){
        Ok(r)=>r,
        Err(e)=>e.render()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ron = std::fs::read_to_string("config.ron").expect("Failed to read config");
    let config: Arc<Config> = Arc::new(ron::de::from_str(&ron).expect("Failed to parse config"));
    println!("Running server on 127.0.0.1:6201");

    HttpServer::new(move || App::new()
        .data(Arc::clone(&config))
        .service(make))
        .bind("127.0.0.1:6201")?
        .run()
        .await
}
