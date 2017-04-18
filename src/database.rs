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

use std::env::temp_dir;

use rusqlite::{Connection, SQLITE_OPEN_READ_WRITE, SQLITE_OPEN_CREATE, SQLITE_OPEN_MEMORY,
               SQLITE_OPEN_FULL_MUTEX, SQLITE_OPEN_URI, SQLITE_OPEN_SHARED_CACHE};

/// Using a slightly modified set of rusqlite open flags, open and return a connection
pub fn db_connect(path: &String) -> Connection {
    let flags = { SQLITE_OPEN_READ_WRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_MEMORY |
                  SQLITE_OPEN_FULL_MUTEX | SQLITE_OPEN_URI |
                  SQLITE_OPEN_SHARED_CACHE };

    let mut db_handle = temp_dir();
    db_handle.push("tblb");
    Connection::open_with_flags(db_handle, flags).unwrap()
}

// Initialize the database
pub fn db_init(conn: &Connection) {
    conn.execute("
        CREATE TABLE IF NOT EXISTS torrent (
            info_hash   TEXT,
            ip          TEXT,
            port        INTEGER,
            peer_id     TEXT,
            remaining   INTEGER,
            last_active INTEGER,
            PRIMARY KEY (info_hash, ip, port, peer_id)
        );",
        &[]
    ).unwrap();
}

pub fn db_prune(conn: &Connection) {
    conn.execute(
        "DELETE FROM torrent
        WHERE (strftime('%s','now') - last_active) > 1860;",
        &[]
    ).unwrap();
}
