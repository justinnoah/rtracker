use std::net::SocketAddr;
use std::str::FromStr;

use ini::Ini;


pub struct ServerConfig {
    pub address: SocketAddr,
    pub db: String,
}

impl ServerConfig {

    /// Try hard to not fail when reading a config by providing default values
    pub fn new<T: AsRef<str>>(path: T) -> ServerConfig {
        // Load the ini and grab the server section
        let ini_file: Ini = Ini::load_from_file(path.as_ref()).unwrap();
        let server_section  = ini_file.section(Some("server")).unwrap();

        // Check for a server address
        let mut addr = SocketAddr::from_str("127.0.0.1:6969").unwrap();
        if server_section.contains_key("address") {
            addr = SocketAddr::from_str(server_section.get("address").unwrap()).unwrap();
        }

        // Check for a database URI
        let mut db_addr: String = String::new();
        if server_section.contains_key("db_address") {
            db_addr = server_section.get("db_address").unwrap().to_string();
        }

        // Lets build us a nice rustic object
        ServerConfig {
            address: addr,
            db: db_addr,
        }
    }
}
