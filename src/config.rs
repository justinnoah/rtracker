use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;

use ini::Ini;

#[derive(Debug)]
pub struct ServerConfig {
    pub address: SocketAddr,
    pub db: String,
}

impl ServerConfig {
    /// Try hard to not fail when reading a config by providing default values
    pub fn new(path: &String) -> ServerConfig {
        let mut cfg_path = Path::new("");

        // Given no config option passed
        if path == "" {
            // Look in default locations for rtracker.ini
            let local = Path::new("./rtracker.ini");
            let home = Path::new("~/.config/rtracker.ini");
            let system = Path::new("/etc/rtracker.ini");
            if local.exists() {
                cfg_path = local;
            } else if home.exists() {
                cfg_path = home;
            } else if system.exists() {
                cfg_path = system;
            }
        } else {
            cfg_path = Path::new(path);
        }

        if !cfg_path.exists() {
            cfg_path = Path::new("");
        }

        debug!("Loading config: {:?}", cfg_path);

        // Load the ini and grab the server section
        if cfg_path != Path::new("") {
            let ini_file: Ini = Ini::load_from_file(cfg_path).unwrap();
            let server_section  = ini_file.section(Some("server")).unwrap();

            // Check for a server address
            let mut addr = String::new();
            if server_section.contains_key("address") {
                addr = server_section.get("address").unwrap().to_string();
            }
            // Check for a database URI
            let mut db = String::new();
            if server_section.contains_key("db_address") {
                db = server_section.get("db_address").unwrap().to_string();
            }

            // Return the object
            ServerConfig {
                address: SocketAddr::from_str(addr.as_str()).unwrap(),
                db:      db,
            }
        } else {
            ServerConfig {
                address: SocketAddr::from_str("127.0.0.1:6969").unwrap(),
                db: String::new(),
            }
        }
    }
}
