//   Copyright 2017 Justin Noah <justinnoah@gmail.com>
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;

use ini::Ini;

#[derive(Debug)]
pub struct ServerConfig {
    pub address: SocketAddr,
    pub pool_size: usize,
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
            let db_section = ini_file.section(Some("db")).unwrap();

            // Check for a server address
            let mut addr = String::new();
            if server_section.contains_key("address") {
                addr = server_section.get("address").unwrap().to_string();
            }

            // Check for db thread pool size option
            let mut pool_size: usize = 10;
            if db_section.contains_key("thread_pool_size") {
                let str_pool_size = db_section.get("thread_pool_size").unwrap();
                pool_size = str_pool_size.parse::<usize>().unwrap();
            }

            // Return the object
            ServerConfig {
                address: SocketAddr::from_str(addr.as_str()).unwrap(),
                pool_size: pool_size,
            }
        } else {
            ServerConfig {
                address: SocketAddr::from_str("127.0.0.1:6969").unwrap(),
                pool_size: 10,
            }
        }
    }
}
