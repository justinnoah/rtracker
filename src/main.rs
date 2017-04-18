// Copyright 2015 Justin Noah, All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
extern crate bincode;
extern crate chrono;
extern crate docopt;
extern crate env_logger;
extern crate ini;
#[macro_use]
extern crate log;
extern crate rand;
extern crate rustc_serialize;
extern crate rusqlite;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

use docopt::Docopt;

use config::{ServerConfig};
use handler::handle_received_packet;
use database::{db_connect, db_init, db_prune};

mod config;
mod handler;
mod database;
mod packet_data_types;
mod parse_packets;


static USAGE: &'static str = "
Usage: rtracker [-c <conf>]
       rtracker (--help)

Options:
    -h, --help          Show this message
    -c, --conf=<conf>   Configuration File [default: ]
";

#[derive(RustcDecodable)]
struct Args {
    flag_conf: String,
}


fn main() {
    env_logger::init().unwrap();
    trace!("Logging initialized!");

    // parse commandline args
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let scfg = ServerConfig::new(&args.flag_conf);
    debug!("addr: {:?}", scfg.address);
    debug!("db: {:?}", scfg.db);

    // Let's first initialize the database.
    let sock = match UdpSocket::bind(&scfg.address) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };
    info!("Listening on: {}", &scfg.address);
    db_init(&db_connect(&scfg.db));
    debug!("DB initialized");

    // Spawn the database pruning thread
    let prune_conn_path = scfg.db.clone();
    thread::spawn(move|| {
        loop {
            // Every 31min (default is 30min, this allows for some delay)
            let prune_delay = Duration::new(31 * 60 as u64, 0);
            thread::sleep(prune_delay);
            let prune_conn = db_connect(&prune_conn_path);
            db_prune(&prune_conn);
            let _ = prune_conn.close();
            // Prune the database
            debug!("Prune the database!");
        }
    });

    loop {
        // This will become flexible. Simply a starting point
        debug!("Init udp packet buffer");
        // UDP packet max
        let mut buf = [0u8; 1500];
        debug!("IOWait");
        let (amt, src) = sock.recv_from(&mut buf).unwrap();
        if amt >= 16 {
            debug!("Clone Socket");
            let tsock = sock.try_clone().unwrap();
            debug!("buf.to_vec");
            let mut b: Vec<u8> = buf.to_vec();
            debug!("Trucate vec at {}", amt);
            b.truncate(amt);
            debug!("Spawn a new thread to handle the packet");
            let handler_path = scfg.db.clone();
            thread::spawn(move|| {
                let conn = db_connect(&handler_path);
                handle_received_packet(tsock, &src, b, &conn);
                let _ = conn.close();
            });
        } else {
            debug!("Received a tiny packet (size: {})", amt)
        }
    }
}
