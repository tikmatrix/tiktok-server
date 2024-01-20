use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AccountData {
    pub id: Option<i32>,
    pub email: String,
    pub pwd: String,
    pub fans: i32,
    pub shop_creator: i32,
    pub device: Option<String>,
    pub username: Option<String>,
    pub group_id: Option<i32>,
    pub earnings: Option<i32>,
    pub today_sales: Option<i32>,
    pub today_sold_items: Option<i32>,
    pub today_orders: Option<i32>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccountDetails {
    pub id: i32,
    pub email: String,
    pub pwd: String,
    pub fans: i32,
    pub shop_creator: i32,
    pub device: Option<String>,
    pub username: Option<String>,
    pub group_id: Option<i32>,
    pub earnings: i32,
    pub today_sales: i32,
    pub today_sold_items: i32,
    pub today_orders: i32,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct AccountResponseData {
    pub data: Vec<AccountDetails>,
}
#[derive(Debug, MultipartForm)]
pub struct MaterialFormData {
    #[multipart(limit = "512 MiB")]
    pub files: Vec<TempFile>,
    pub group_id: Option<Text<i32>>,
}
#[derive(Debug, MultipartForm)]
pub struct InstallFormData {
    #[multipart(limit = "512 MiB")]
    pub file: TempFile,
    pub serial: Option<Text<String>>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct MaterialData {
    pub id: Option<i32>,
    pub name: String,
    pub md5: String,
    pub group_id: i32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialUesData {
    pub name: String,
    pub used: i32,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct MaterialDetails {
    pub id: i32,
    pub name: String,
    pub md5: String,
    pub used: i32,
    pub group_id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MaterialResponseData {
    pub data: Vec<MaterialDetails>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PublishJobData {
    pub id: Option<i32>,
    pub material: Option<String>,
    pub account: Option<String>,
    pub title: Option<String>,
    pub tags: Option<String>,
    pub status: Option<i32>,
    pub start_time: Option<String>,
    pub group_id: Option<i32>,
    pub publish_type: i32,
    pub product_link: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct PublishJobDetails {
    pub id: i32,
    pub material: String,
    pub account: String,
    pub title: Option<String>,
    pub tags: Option<String>,
    pub status: i32,
    pub start_time: String,
    pub end_time: String,
    pub device: Option<String>,
    pub group_id: i32,
    pub publish_type: i32,
    pub product_link: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PublishJobResponseData {
    pub data: Vec<PublishJobDetails>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TrainJobData {
    pub id: Option<i32>,
    pub group_id: Option<i32>,
    pub click: Option<i32>,
    pub follow: Option<i32>,
    pub favorites: Option<i32>,
    pub account: Option<String>,
    pub status: Option<i32>,
    pub start_time: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct TrainJobDetails {
    pub id: i32,
    pub group_id: i32,
    pub click: i32,
    pub follow: i32,
    pub favorites: i32,
    pub account: String,
    pub status: i32,
    pub start_time: String,
    pub end_time: String,
    pub device: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct TrainJobResponseData {
    pub data: Vec<TrainJobDetails>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeviceData {
    pub serial: String,
    pub forward_port: i32,
    pub online: i32,
    pub ip: Option<String>,
    pub agent_ip: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceDetails {
    pub id: i32,
    pub serial: String,
    pub forward_port: i32,
    pub online: i32,
    pub ip: Option<String>,
    pub agent_ip: String,
    pub init: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceResponseData {
    pub data: Vec<DeviceDetails>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ShellData {
    pub serial: Option<String>,
    pub cmd: String,
}

#[derive(Deserialize)]
pub struct ScriptQueryParams {
    pub script: String,
    pub serial: Option<String>,
    pub args: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseData {
    pub data: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupData {
    pub id: Option<i32>,
    pub name: String,
    pub auto_train: i32,
    pub auto_publish: i32,
    pub publish_start_time: String,
    pub train_start_time: String,
    pub title: Option<String>,
    pub tags: Option<String>,
    pub publish_type: i32,
    pub product_link: Option<String>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GroupDetails {
    pub id: i32,
    pub name: String,
    pub auto_train: i32,
    pub auto_publish: i32,
    pub publish_start_time: String,
    pub train_start_time: String,
    pub title: Option<String>,
    pub tags: Option<String>,
    pub publish_type: i32,
    pub product_link: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct GroupResponseData {
    pub data: Vec<GroupDetails>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct DialogWatcherData {
    pub id: Option<i32>,
    pub conditions: Option<String>,
    pub action: Option<String>, //click,back
    pub status: Option<i32>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct DialogWatcherDetails {
    pub id: i32,
    pub name: String,
    pub conditions: String,
    pub action: String, //click,back
    pub status: i32,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct DialogWatcherResponseData {
    pub data: Vec<DialogWatcherDetails>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceDataList {
    pub data: Vec<DeviceData>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct LicenseData {
    pub code: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct LicenseResponseData {
    pub code: i32,
    pub data: Option<LicenseDetails>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct LicenseDetails {
    pub name: String,
    pub expire: i64,
}
