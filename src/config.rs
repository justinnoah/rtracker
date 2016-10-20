extern crate ini;
use self::ini::Ini;

pub fn load_config<T: AsRef<str>>(path: T) -> (String, String) {
    let ini_file = Ini::load_from_file(path.as_ref()).unwrap();

    let server  = ini_file.section(Some("server")).unwrap();
    let ip: &String = server.get("ip").unwrap();
    let port: &String = server.get("port").unwrap();

    (ip.to_string(), port.to_string())
}
