extern crate ini;
use self::ini::Ini;

pub fn load_config<T: AsRef<str>>(path: T) -> (String, String) {
    let ini_file = Ini::load_from_file(path.as_ref()).unwrap();

    let server  = ini_file.section(Some("server")).unwrap();
    let ip = server.get("ip").unwrap();
    let port = server.get("port").unwrap();

    (ip.as_ref().to_string(), port.as_ref().to_string())
}
