use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub download: DownloadConfig,
    pub servers: Vec<ServerConfig>,
    pub news: NewsConfig,
}

#[derive(Deserialize)]
pub struct GeneralConfig {
    pub listen_port: u16,
    pub last_version: i16,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct DownloadConfig {
    pub ftp_url: String,
    pub ftp_path: String,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub ip: String,
    pub name: String,
    pub user_limit: i16,
}

#[derive(Deserialize)]
pub struct NewsConfig {
    pub title: String,
    pub message: String,
}

pub struct ServerState {
    pub ip: String,
    pub name: String,
    pub user_count: i16,
    pub user_limit: i16,
}
