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
#![feature(net)]

extern crate rand;
extern crate rusqlite;
extern crate "rustc-serialize" as rustc_serialize;
extern crate time;

use rand::Rng;
use std::net::{SocketAddr, UdpSocket};
use std::thread::Thread;

use rusqlite::SqliteConnection;

use parse_packets::{parse_header, encode_connect_response, decode_client_announce};

mod parse_packets;

fn gen_uuid() -> u64 {
    let mut rng = rand::thread_rng();
    let mut uuid = time::precise_time_ns();
    uuid <<= 32;
    uuid | (rng.gen::<u32>() as u64)
}

fn init_db(path: &'static str) -> SqliteConnection {
    let conn = SqliteConnection::open(path).unwrap();
    conn.execute_batch(
        "BEGIN;
        CREATE TABLE IF NOT EXISTS users (
            uuid          INTEGER PRIMARY KEY,
            last_active   INTEGER
        );
        CREATE TABLE IF NOT EXISTS torrent (
            info_hash   TEXT,
            uuid        INTEGER,
            downloaded  INTEGER,
            uploaded    INTEGER,
            remaining   INTEGER,
            PRIMARY KEY (info_hash, uuid)
            FOREIGN KEY(uuid) REFERENCES users(uuid)
        );
        COMMIT;"
    ).unwrap();
    conn
}

fn add_new_connection(conn: &SqliteConnection) -> u64 {
    // Cool Story, we got a new connection.
    // We need to generate an unique id for this client.
    // 32bits of the current time in nanoseconds combined with 32bits of
    // random numbers
    let uuid = gen_uuid() as i64;
    conn.execute(
        "INSERT INTO users (uuid, last_active) VALUES ($1, strftime('%s', 'now'))",
        &[&uuid]
    ).unwrap();
    uuid as u64
}

fn handle_packet(tsock: UdpSocket, src: &SocketAddr, packet: Vec<u8>, conn: &SqliteConnection) {
    // Split the packet into header and body parts
    let mut packet_header = packet;
    let packet_body = packet_header.split_off(16);

    // parse the header to act on it
    let header = parse_header(&packet_header);

    println!("Connection ID: 0x{:x}", header.connection_id);
    match header.action {
        0 => {
            if header.connection_id == 0x41727101980 {
                let uuid = add_new_connection(conn);
                // Now they're in the db, let's say hi
                let encoded = encode_connect_response(uuid, header.transaction_id);
                tsock.send_to(&encoded, src).unwrap();
            } else {
                ()
            }
        },
        1 => {
            let decoded = decode_client_announce(&packet_body);
        },
        _ => {
            ()
        },
    }
}

fn main() {
    let database_path = "file::memory:?cache=shared";

    // Let's first initialize the database.
    let _ = init_db(database_path);
    let sock = UdpSocket::bind("127.0.0.1:6969").unwrap();

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
