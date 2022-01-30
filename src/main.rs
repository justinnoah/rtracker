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

extern crate bincode;
extern crate chrono;
extern crate docopt;
extern crate env_logger;
extern crate ini;
#[macro_use]
extern crate log;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rand;
extern crate rusqlite;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

use docopt::Docopt;

use config::ServerConfig;
use database::{db_connection_pool, db_init, db_prune};
use handler::handle_received_packet;

mod config;
mod database;
mod handler;
mod packet_data_types;
mod parse_packets;

static USAGE: &str = "
Usage: rtracker [-c <conf>]
       rtracker (--help)

Options:
    -h, --help          Show this message
    -c, --conf=<conf>   Configuration File [default: ]
";

#[derive(Deserialize)]
struct Args {
    flag_conf: String,
}

fn main() {
    env_logger::init();
    trace!("Logging initialized!");

    // parse commandline args
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let scfg = ServerConfig::new(&args.flag_conf);
    debug!("addr: {:?}", scfg.address);

    // Let's first initialize the database.
    let sock = match UdpSocket::bind(&scfg.address) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };
    info!("Listening on: {}", &scfg.address);
    let pool = db_connection_pool(scfg.pool_size);
    db_init(pool.get().unwrap());
    debug!("DB initialized");

    // Spawn the database pruning thread
    let prune_pool = pool.clone();
    thread::spawn(move || {
        loop {
            // Every 31min (default is 30min, this allows for some delay)
            let prune_delay = Duration::new(31 * 60u64, 0);
            thread::sleep(prune_delay);
            let prune_conn = prune_pool.get().unwrap();
            db_prune(prune_conn);
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
        let tsock = sock.try_clone().unwrap();
        let tpool = pool.clone();
        if amt >= 16 {
            debug!("Spawn a new thread to handle the packet");
            thread::spawn(move || {
                let mut packet: Vec<u8> = buf.to_vec();
                packet.resize(amt, 0);
                handle_received_packet(packet, src, tsock, tpool.get().unwrap());
            });
        } else {
            debug!("Received a tiny packet (size: {}), ignoring", amt)
        }
    }
}
