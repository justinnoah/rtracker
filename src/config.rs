//  rtracker: bittorrent tracker
//  Copyright (C) 2019  Justin Noah <justinnoah@gmail.com>
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License.
//
//  This program is distributed in the hope that it will be useful,
//   but WITHOUT ANY WARRANTY; without even the implied warranty of
//   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//   GNU Affero General Public License for more details.
//
//   You should have received a copy of the GNU Affero General Public License
//   along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
