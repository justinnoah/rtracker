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

extern crate rusqlite;
extern crate time;

use std::net::{SocketAddr, UdpSocket};
use std::rand;
use std::rand::Rng;
use std::sync::Arc;
use std::thread::Thread;

use rusqlite::SqliteConnection;

#[repr(C)]
struct PacketHeader {
    connection_id:  u64,
    action:         u32,
    transaction_id: u32,
}

fn gen_uuid() -> u64 {
    let mut rng = rand::thread_rng();
    let mut uuid = time::precise_time_ns();
    uuid <<= 32;
    uuid | (rng.gen::<u32>() as u64)
}

fn parse_header(packet: &[u8]) -> PacketHeader {
    let p_con_id            = &packet[0..7];
    let p_action            = &packet[8..11];
    let p_tran_id           = &packet[12..15];

    let mut connection_id:  u64 = 0;
    let mut action:         u32 = 0;
    let mut transaction_id: u32 = 0;

    for i in p_con_id.iter() {
        connection_id <<= 8;
        connection_id |= (*i as u64);
    }

    for i in p_action.iter() {
        action <<= 8;
        action |= (*i as u32);
    }

    for i in p_tran_id.iter() {
        transaction_id <<= 8;
        transaction_id |= (*i as u32);
    }

    PacketHeader {connection_id: connection_id, action: action, transaction_id: transaction_id}
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

fn create_connection(conn: &SqliteConnection) {
    // Cool Story, we got a new connection.
    // We need to generate an unique id for this client.
    // 32bits of the current time in nanoseconds combined with 32bits of
    // random numbers
    let uuid = gen_uuid() as i64;
    conn.execute(
        "INSERT OR REPLACE INTO users (uuid, last_active) VALUES ($1, strftime('%s', 'now'))",
        &[&uuid]
    ).unwrap();
}

fn handle_packet(src: &SocketAddr, amt: usize, packet: [u8; 2048], conn: &SqliteConnection) {
    let header = parse_header(&packet[0..15]);

    if header.connection_id == 17568305177 {
        create_connection(conn);
        return
    }
}

fn main() {
    let DATABASE_PATH = "file::memory:?cache=shared";

    // Let's first initialize the database.
    let conn = init_db(DATABASE_PATH);
    let mut sock = UdpSocket::bind("127.0.0.1:6969").unwrap();

    loop {
        let mut buf = [0; 2048];
        let (amt, src) = sock.recv_from(&mut buf).unwrap();
        Thread::spawn(move|| {
            let conn = SqliteConnection::open(DATABASE_PATH).unwrap();
            handle_packet(&src, amt, buf, &conn);
            conn.close();
        });
    }
    conn.close();
}
