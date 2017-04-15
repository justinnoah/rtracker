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
        let mut n = ServerConfig {
            address: SocketAddr::from_str("127.0.0.1:6969").unwrap(),
            db: String::new(),
        };

        // Load the ini and grab the server section
        if Path::new(path.as_str()).exists() {
            let ini_file: Ini = Ini::load_from_file(path.as_str()).unwrap();
            let server_section  = ini_file.section(Some("server")).unwrap();

            // Check for a server address
            if server_section.contains_key("address") {
                n.address = SocketAddr::from_str(server_section.get("address").unwrap()).unwrap();
            }

            // Check for a database URI
            if server_section.contains_key("db_address") {
                n.db = server_section.get("db_address").unwrap().to_string();
            }
        }
        debug!("addr: {:?}", n.address);
        debug!("db: {:?}", n.db);

        // Return constructed configuration
        n
    }
}
