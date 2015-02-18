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

#![feature(core)]
#![feature(collections)]
#![feature(net)]
#![feature(std_misc)]

extern crate bincode;
extern crate rand;
extern crate rusqlite;
extern crate "rustc-serialize" as rustc_serialize;
extern crate time;

use rand::Rng;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::thread::Thread;

use rusqlite::SqliteConnection;

use parse_packets::*;

mod parse_packets;

struct ID {
    info_hash:  [u8; 20],
    ip:         u32,
    port:       u16,
    peer_id:    [u8; 20],
    remaining:  i64,
}

fn gen_uuid() -> i64 {
    let mut rng = rand::thread_rng();
    let mut uuid = time::precise_time_ns();
    uuid <<= 32;
    uuid as i64 | rng.gen::<u32>() as i64
}

fn init_db(path: &'static str) -> SqliteConnection {
    let conn = SqliteConnection::open(path).unwrap();
    conn.execute("
        CREATE TABLE IF NOT EXISTS torrent (
            info_hash   TEXT,
            ip          INTEGER,
            port        INTEGER,
            peer_id     TEXT,
            remaining   INTEGER,
            last_active INTEGER,
            PRIMARY KEY (info_hash, ip, port, peer_id)
        );",
        &[]
    ).unwrap();
    conn
}

fn update_announce(conn: &SqliteConnection, id: &ID, data: &ClientAnnounce) -> (Vec<(i32,i32)>,i32, i32) {
    // Update the last seen time
    conn.execute(
        "INSERT OR REPLACE INTO torrent (info_hash, ip, port, peer_id, remaining, last_active)
        VALUES (?, ?, ?, ?, ?, strftime('%s', 'now'))",
        &[&id.info_hash.as_slice(), &(id.ip as i32), &(id.port as i32),
          &id.peer_id.as_slice(), &id.remaining]
    ).unwrap();

    // Get Seeders
    let mut stmt = conn.prepare(
        "SELECT ip,port,COUNT(*)
         FROM torrent
         WHERE info_hash = ? AND remaining = 0
         GROUP BY ip,port"
    ).unwrap();

    let mut swarm: Vec<(i32, i32)> = Vec::new();

    let mut seeders: i32 = 0;
    for row in stmt.query(&[&data.info_hash.as_slice()]).unwrap().map(|row| row.unwrap()) {
        let i: i32 = row.get(0);
        let p: i32 = row.get(1);
        swarm.push((i,p));
        let c: i32 = row.get(2);
        if c > seeders {
            seeders = c;
        }
    }

    // Get Leechers
    let mut stmt = conn.prepare(
        "SELECT ip,port,COUNT(*)
         FROM torrent
         WHERE info_hash = ? AND remaining > 0
         GROUP BY ip,port"
    ).unwrap();

    let mut leechers: i32 = 0;
    for row in stmt.query(&[&data.info_hash.as_slice()]).unwrap().map(|row| row.unwrap()) {
        let i: i32 = row.get(0);
        let p: i32 = row.get(1);
        swarm.push((i,p));
        let c: i32 = row.get(2);
        if c > leechers {
            leechers = c;
        }
    }

    (swarm, seeders, leechers)
}

fn handle_packet(tsock: UdpSocket, src: &SocketAddr, packet: Vec<u8>, conn: &SqliteConnection) {
    // Split the packet into header and body parts
    let mut packet_header = packet;
    let packet_body = packet_header.split_off(16);

    // parse the header to act on it
    let header = parse_header(&packet_header);

    match header.action {
        0 => {
            if header.connection_id == 0x41727101980 {
                // Cool Story, we got a new connection.
                // We need to generate an unique id for this client.
                // 32bits of the current time in nanoseconds combined with 32bits of
                // random numbers
                let uuid = gen_uuid();

                // Now they're in the db, let's say hi
                let encoded = encode_connect_response(uuid, header.transaction_id);
                tsock.send_to(&encoded, src).unwrap();
            } else {
                ()
            }
        },
        1 => {
            let decoded = decode_client_announce(&packet_body);
            let mut ip = decoded.ip;
            if ip == 0 {
                ip = match src.ip() {
                    IpAddr::V4(x) => {
                        bincode::decode(&x.octets()).unwrap()
                    },
                    _ => panic!("This is possible?")
                };
            }
            let id = ID {
                info_hash: decoded.info_hash,
                ip: ip,
                port: decoded.port,
                peer_id: decoded.peer_id,
                remaining: decoded.remaining,
            };
            let (swarm, seeders, leechers) = update_announce(conn, &id, &decoded);
            let serv_announce = encode_server_announce(header.transaction_id, swarm, leechers, seeders);
            tsock.send_to(&serv_announce, src).unwrap();
        },
        _ => {
            println!("Unhandled action: {:?}", header.action);
        },
    }
}

fn main() {
    let database_path = "file::memory:?cache=shared";

    // Let's first initialize the database.
    let _ = init_db(database_path);
    let sock = UdpSocket::bind("0.0.0.0:6969").unwrap();

    loop {
        let mut buf = [0u8; 2048];
        let (amt, src) = sock.recv_from(&mut buf).unwrap();
        let tsock = sock.try_clone().unwrap();
        let mut b: Vec<u8> = buf.to_vec();
        b.truncate(amt);
        Thread::spawn(move|| {
            let conn = SqliteConnection::open(database_path).unwrap();
            handle_packet(tsock, &src, b, &conn);
            let _ = conn.close();
        });
    }
}
