pub const HOST: &str = "127.0.0.1";
pub const DNS: &str = "localhost";

pub const DISPATCH_ALWAYS_CHECK_HOTFIX: bool = true;
pub const DISPATCH_BIND_TARGET: (&str, u16) = (HOST, 21000);
pub const DISPATCH_ENV_TYPE: &str = "2";
pub const DISPATCH_REGION_NAME: &str = "Railgun";
pub const GAMESERVER_BIND_TARGET: (&str, u16) = (HOST, 23301);

pub const CERT_DIR: &str = "./certs";
pub const CRT_FILE_PATH: &str = "./certs/default.crt";
pub const KEY_FILE_PATH: &str = "./certs/default.key";
