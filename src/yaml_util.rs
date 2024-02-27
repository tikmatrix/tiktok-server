use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ListConfig {
    pub files: Vec<FileConfig>,
    pub index: i64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FileConfig {
    pub url: String,
    pub time: String,
    pub name: String,
    pub selected: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub port: i64,
    #[serde(rename = "socks-port")]
    pub socks_port: i64,
    #[serde(rename = "allow-lan")]
    pub allow_lan: bool,
    pub mode: String,
    #[serde(rename = "log-level")]
    pub log_level: String,
    #[serde(rename = "external-controller")]
    pub external_controller: String,
    pub secret: String,

    pub proxies: Vec<ProxyConfig>,
    #[serde(rename = "proxy-groups")]
    pub proxy_groups: Vec<ProxyGroupConfig>,
    pub script: ScriptConfig,
    pub rules: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub server: String,
    pub port: i64,
    pub username: String,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyGroupConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub proxies: Vec<String>,
}
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScriptConfig {
    pub engine: String,
    pub shortcuts: HashMap<String, String>,
}

use dirs;
use std::io::Write;
use std::path::PathBuf;

use crate::runtime_err::RunTimeError;

pub fn get_config_path() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    home_dir.join(".config\\clash\\profiles\\config.yml")
}

pub fn new_proxy_config(proxy: &str) -> ProxyConfig {
    let parts: Vec<&str> = proxy.split(':').collect();
    ProxyConfig {
        name: parts[0].to_string(),
        type_: "http".to_string(),
        server: parts[0].to_string(),
        port: parts[1].parse().unwrap(),
        username: parts[2].to_string(),
        password: parts[3].to_string(),
    }
}
pub fn read_yaml() -> Result<ProfileConfig, RunTimeError> {
    let config_path = get_config_path();
    print!("config_path: {:?}", config_path);
    let contents = std::fs::read_to_string(config_path);
    if contents.is_err() {
        return Err(RunTimeError::new("read config file error"));
    }
    let contents = contents.unwrap();
    let config: Result<ProfileConfig, serde_yaml::Error> = serde_yaml::from_str(&contents);
    if config.is_err() {
        return Err(RunTimeError::new("parse config file error"));
    }
    Ok(config.unwrap())
}

pub fn write_yaml(config: &ProfileConfig) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    let yaml = serde_yaml::to_string(config)?;
    let mut file = std::fs::File::create(config_path)?;
    file.write_all(yaml.as_bytes())?;
    Ok(())
}
pub fn reload_clash() {
    let url = "http://127.0.0.1:60979/configs";
    let path = get_config_path();
    //replace \\ with /
    let path = path.to_str().unwrap().replace("\\", "/");
    let body = format!("{{\"path\":\"{}\"}}", path);
    println!("reload_clash: {:?}", body);
    let client = reqwest::blocking::Client::new();
    let resp = client
        .put(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .unwrap();
    let body = resp.text().unwrap();
    println!("reload_clash: {:?}", body);
}
pub fn add_proxys_to_config(proxys: Vec<String>) {
    let mut config = read_yaml().unwrap();
    for proxy in proxys {
        let proxy_config = new_proxy_config(&proxy);
        let name = proxy_config.name.clone();
        config.proxies.push(proxy_config);
        config.proxy_groups[0].proxies.push(name);
    }
    write_yaml(&config).unwrap();
    reload_clash();
}
pub fn remove_proxys_from_config(proxys: Vec<String>) {
    let mut config = read_yaml().unwrap();
    for proxy in proxys {
        let index = config.proxies.iter().position(|x| x.name == proxy).unwrap();
        config.proxies.remove(index);
        let index = config.proxy_groups[0]
            .proxies
            .iter()
            .position(|x| x == &proxy)
            .unwrap();
        config.proxy_groups[0].proxies.remove(index);
    }
    write_yaml(&config).unwrap();
    reload_clash();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_yaml() {
        let config = read_yaml().unwrap();
        println!("{:?}", config);
    }
    #[test]
    fn test_write_yaml() {
        let mut config = read_yaml().unwrap();
        let proxy_config = new_proxy_config("207.228.0.183:49559:xxxx:xxxx");
        let name = proxy_config.name.clone();
        config.proxies.push(proxy_config);
        config.proxy_groups[0].proxies.push(name);
        write_yaml(&config).unwrap();
    }
    #[test]
    fn test_reload_clash() {
        reload_clash();
    }
}
