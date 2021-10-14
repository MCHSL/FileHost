mod auth;
mod file_methods;
mod guards;

#[macro_use]
extern crate rocket;

use envconfig::Envconfig;
use qrcode::render::unicode;
use qrcode::QrCode;
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "FILEHOST_AUTH_SECRET")]
    pub auth_secret: String,
    #[envconfig(from = "FILEHOST_FILES_DIR", default = "./files/")]
    pub file_directory: String,
    #[envconfig(from = "FILEHOST_ADDRESS", default = "0.0.0.0")]
    pub address: String,
    #[envconfig(from = "FILEHOST_PORT", default = "6880")]
    pub port: u16,
    #[envconfig(from = "FILEHOST_MAX_FILE_SIZE", default = "50MB")]
    pub max_file_size: String,
}

#[launch]
fn rocket() -> _ {
    let config = Config::init_from_env().unwrap();

    if std::env::args().skip(1).next() == Some("--code".to_string()) {
        let scheme = auth::create_scheme("", config.auth_secret.as_str(), "FileHost");

        let code = QrCode::new(scheme).unwrap();
        let image = code
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build();
        println!("{}", image);
        std::process::exit(0);
    }

    let allowed_origins = AllowedOrigins::all();

    use rocket::http::Method::*;
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Get, Post, Options, Delete]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .unwrap();

    let tokens: guards::ValidTokens = Arc::new(Mutex::new(HashSet::new()));

    let mut rocket_config = rocket::Config::default();
    rocket_config.address = config
        .address
        .parse::<std::net::IpAddr>()
        .expect("Invalid address");

    rocket_config.port = config.port;
    rocket_config.limits = rocket_config
        .limits
        .limit(
            "file",
            config.max_file_size.parse().expect("Invalid max file size"),
        )
        .limit(
            "data-form",
            config.max_file_size.parse().expect("Invalid max file size"),
        );

    use file_methods::*;
    rocket::build()
        .configure(rocket_config)
        .manage(tokens)
        .manage(config)
        .mount(
            "/",
            routes![auth::login, files, upload, download, delete_file],
        )
        .mount("/", rocket::fs::FileServer::from("./static"))
        .attach(cors)
}
