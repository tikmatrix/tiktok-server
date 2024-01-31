use crate::ddl_actor::DdlMessage;
use crate::models::{
    AccountData, DeviceDataList, DialogWatcherData, GroupData, LicenseData, LicenseDetails,
    LicenseResponseData, MaterialData, MaterialFormData, MaterialUesData, MusicData,
    PublishJobData, ResponseData, ScriptQueryParams, TrainJobData,
};
use crate::models::{InstallFormData, ShellData};
use crate::{
    account_dao, aes_util, device_dao, dialog_watcher_dao, group_dao, material_dao, music_dao,
    publish_job_dao, request_util, train_job_dao,
};
use actix_multipart::form::MultipartForm;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use local_ip_address::local_ip;
use rusqlite::Connection;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, fs::File};
use uuid::Uuid;
#[post("/api/account")]
pub(crate) async fn add_account_api(
    conn: web::Data<Mutex<Connection>>,
    web::Json(account_data): web::Json<AccountData>,
) -> actix_web::Result<impl Responder> {
    web::block(move || account_dao::save(&conn, account_data)).await??;
    Ok(web::Json(ResponseData {
        data: "ok".to_string(),
    }))
}
#[put("/api/account")]
pub(crate) async fn update_account_api(
    conn: web::Data<Mutex<Connection>>,
    web::Json(account_data): web::Json<AccountData>,
) -> actix_web::Result<impl Responder> {
    web::block(move || account_dao::update(&conn, account_data)).await??;
    Ok(web::Json(ResponseData {
        data: "ok".to_string(),
    }))
}

#[get("/api/account")]
pub(crate) async fn get_account_api() -> actix_web::Result<impl Responder> {
    let account_response_data = web::block(move || account_dao::list_all()).await??;
    Ok(web::Json(account_response_data))
}
#[get("/api/account/auto_train")]
pub(crate) async fn get_account_auto_train_api(
    web::Query(query): web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let agent_ip = query
        .get("agent_ip")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing agent_ip query parameter"))?
        .clone();
    let account_response_data =
        web::block(move || account_dao::list_auto_train_account_by_agent_ip(agent_ip)).await??;
    Ok(web::Json(account_response_data))
}

#[get("/api/account_by_device")]
pub(crate) async fn get_account_by_device_api(
    web::Query(query): web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let device = query
        .get("device")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing device query parameter"))?
        .clone();
    let account_response_data =
        web::block(move || account_dao::list_account_by_device(device)).await??;
    Ok(web::Json(account_response_data))
}
#[delete("/api/account")]
pub(crate) async fn delete_account_api(
    web::Query(query): web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let id = query
        .get("id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing id query parameter"))?
        .clone();
    web::block(move || account_dao::del(id)).await??;
    Ok(HttpResponse::NoContent())
}
#[post("/api/install")]
pub(crate) async fn install_api(
    MultipartForm(form): MultipartForm<InstallFormData>,
) -> actix_web::Result<impl Responder> {
    log::info!("install api");
    let serial = form.serial.as_ref().map(|s| s.0.clone());
    let file = form.file;
    let file_name = file.file_name.unwrap();
    let path = format!("upload/apk/{}", file_name);
    log::debug!("saving to {path}");
    file.file.persist(path).unwrap();
    let my_local_ip = local_ip();
    if let Ok(my_local_ip) = my_local_ip {
        log::debug!("my_local_ip: {:?}", my_local_ip);
    } else {
        log::error!("my_local_ip: {:?}", my_local_ip);
    }
    let url: String = format!("http://{}:8090/apk/{}", my_local_ip.unwrap(), file_name);
    let devices = device_dao::list_online_device(serial, None);
    if let Ok(devices) = devices {
        for device in devices.data {
            let result = request_util::get_json::<ResponseData>(
                device.agent_ip.as_str(),
                &format!(
                    "/api/device_install?serial={}&url={}",
                    device.serial.as_str(),
                    url.as_str(),
                ),
            )
            .await;
            if let Ok(result) = result {
                log::info!("device install result: {:?}", result);
            } else {
                log::error!("device install error: {:?}", result);
            }
        }
    }

    Ok(HttpResponse::Ok())
}

#[post("/api/material")]
pub(crate) async fn add_material_api(
    conn: web::Data<Mutex<Connection>>,
    MultipartForm(form): MultipartForm<MaterialFormData>,
) -> actix_web::Result<impl Responder> {
    let mut materials: Vec<MaterialData> = Vec::new();
    let group_id = form.group_id.as_ref().map(|s| s.0.clone()).unwrap_or(0);
    for f in form.files {
        let file_name = f.file_name.unwrap();
        let extension = Path::new(&file_name)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("");
        let name = format!("{}.{}", Uuid::new_v4(), extension);
        let path = format!("upload/material/{}", name);
        log::debug!("saving to {path}");
        f.file.persist(path.clone()).unwrap();
        let mut file = File::open(path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let md5 = format!("{:x}", md5::compute(&buffer));
        materials.push(MaterialData {
            id: None,
            name: format!("material/{}", name),
            md5,
            group_id: group_id.clone(),
        });
    }

    web::block(move || material_dao::save(&conn, materials)).await??;
    Ok(HttpResponse::Ok())
}

#[put("/api/material")]
pub(crate) async fn update_material_api(
    conn: web::Data<Mutex<Connection>>,
    web::Json(material_data): web::Json<MaterialUesData>,
) -> actix_web::Result<impl Responder> {
    let name = material_data.name;
    let used: i32 = material_data.used;
    web::block(move || material_dao::update(&conn, name, used)).await??;
    Ok(HttpResponse::NoContent())
}
#[get("/api/material")]
pub(crate) async fn get_material_api(
    web::Query(query): web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let used = query.get("used").cloned();
    let used = used.map(|s| s.parse::<i32>().unwrap_or(0));
    let group_id = query.get("group_id").cloned();
    let group_id = group_id.map(|s| s.parse::<i32>().unwrap_or(0));
    let material_response_data = web::block(move || material_dao::list(used, group_id)).await??;
    Ok(web::Json(material_response_data))
}
#[derive(Serialize)]
struct MaterialCountResponse {
    data: i32,
}
#[get("/api/material/count")]
pub(crate) async fn get_material_count_api(
    web::Query(query): web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let used = query.get("used").cloned();
    let used = used.map(|s| s.parse::<i32>().unwrap_or(0));
    let group_id = query.get("group_id").cloned();
    let group_id = group_id.map(|s| s.parse::<i32>().unwrap_or(0));
    let count = web::block(move || material_dao::count(used, group_id)).await??;
    Ok(web::Json(MaterialCountResponse { data: count }))
}
#[delete("/api/material")]
pub(crate) async fn delete_material_api(
    web::Query(query): web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let id = query
        .get("id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing id query parameter"))?
        .clone();
    let id: i32 = id.parse::<i32>().unwrap_or(0);
    web::block(move || material_dao::del(id)).await??;
    Ok(HttpResponse::NoContent())
}
#[post("/api/publish_job")]
pub(crate) async fn add_job_api(
    conn: web::Data<Mutex<Connection>>,
    web::Json(job_data): web::Json<PublishJobData>,
) -> actix_web::Result<impl Responder> {
    let material = job_data.material.clone();
    let conn_clone = conn.clone();
    web::block(move || publish_job_dao::save(&conn_clone, job_data)).await??;
    //update material used
    let used = 1;
    let conn_clone = conn.clone();
    web::block(move || material_dao::update(&conn_clone, material.unwrap(), used)).await??;
    Ok(HttpResponse::NoContent())
}
#[put("/api/publish_job")]
pub(crate) async fn update_job_api(
    conn: web::Data<Mutex<Connection>>,
    web::Json(job_data): web::Json<PublishJobData>,
) -> actix_web::Result<impl Responder> {
    web::block(move || publish_job_dao::update(&conn, job_data)).await??;
    Ok(web::Json(ResponseData {
        data: "ok".to_string(),
    }))
}
#[get("/api/publish_job")]
pub(crate) async fn get_job_api() -> actix_web::Result<impl Responder> {
    let job_response_data = web::block(move || publish_job_dao::list_all()).await??;
    Ok(web::Json(job_response_data))
}

#[get("/api/runable_publish_job")]
pub(crate) async fn runable_publish_job_api(
    web::Query(query): web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let agent_ip = query
        .get("agent_ip")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing agent_ip query parameter"))?
        .clone();
    let job_response_data = web::block(move || publish_job_dao::list_runable(agent_ip)).await??;
    Ok(web::Json(job_response_data))
}

#[delete("/api/publish_job")]
pub(crate) async fn delete_job_api(
    web::Query(query): web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let id_str = query
        .get("id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing id query parameter"))?;

    let id = id_str
        .parse::<i32>()
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid id query parameter"))?;
    web::block(move || publish_job_dao::del(id)).await??;
    Ok(HttpResponse::NoContent())
}
#[post("/api/train_job")]
pub(crate) async fn add_train_job_api(
    conn: web::Data<Mutex<Connection>>,
    web::Json(job_data): web::Json<TrainJobData>,
) -> actix_web::Result<impl Responder> {
    web::block(move || train_job_dao::save(&conn, job_data)).await??;
    Ok(HttpResponse::NoContent())
}
#[put("/api/train_job")]
pub(crate) async fn update_train_job_api(
    conn: web::Data<Mutex<Connection>>,
    web::Json(job_data): web::Json<TrainJobData>,
) -> actix_web::Result<impl Responder> {
    web::block(move || train_job_dao::update(&conn, job_data)).await??;
    Ok(web::Json(ResponseData {
        data: "ok".to_string(),
    }))
}
#[get("/api/train_job")]
pub(crate) async fn get_train_job_api() -> actix_web::Result<impl Responder> {
    let job_response_data = web::block(move || train_job_dao::list_all()).await??;
    Ok(web::Json(job_response_data))
}
#[get("/api/runable_train_job")]
pub(crate) async fn runable_train_job_api(
    web::Query(query): web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let agent_ip = query
        .get("agent_ip")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing agent_ip query parameter"))?
        .clone();
    let job_response_data = web::block(move || train_job_dao::list_runable(agent_ip)).await??;
    Ok(web::Json(job_response_data))
}
#[delete("/api/train_job")]
pub(crate) async fn delete_train_job_api(
    web::Query(query): web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let id_str = query
        .get("id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing id query parameter"))?;

    let id = id_str
        .parse::<i32>()
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid id query parameter"))?;
    web::block(move || train_job_dao::del(id)).await??;
    Ok(HttpResponse::NoContent())
}
#[post("/api/device")]
pub(crate) async fn add_device_api(
    ddl_sender_data: web::Data<Arc<Mutex<Sender<DdlMessage>>>>,
    web::Json(device_data_list): web::Json<DeviceDataList>,
) -> actix_web::Result<impl Responder> {
    for device_data in device_data_list.data {
        let device_data = device_data.clone();
        let ddl_sender_data_clone = ddl_sender_data.clone();
        web::block(move || device_dao::save(&ddl_sender_data_clone, device_data)).await??;
    }
    Ok(web::Json(ResponseData {
        data: { "ok".to_string() },
    }))
}

#[get("/api/device")]
pub(crate) async fn get_device_api(
    web::Query(query): web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let agent_ip = query.get("agent_ip");
    let agent_ip = agent_ip.map(|s| s.clone());
    let device_response_data =
        web::block(move || device_dao::list_online_device(None, agent_ip)).await??;
    Ok(web::Json(device_response_data))
}
#[get("/api/device/init")]
pub(crate) async fn get_device_init_api(
    conn: web::Data<Mutex<Connection>>,
    web::Query(query): web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let serial = query
        .get("serial")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing serial query parameter"))?
        .clone();
    let init = query
        .get("init")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing init query parameter"))?
        .clone();
    //convert init i32
    let init = init
        .parse::<i32>()
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid init query parameter"))?;
    web::block(move || device_dao::update_init(&conn, &serial, &init)).await??;
    Ok(web::Json(ResponseData {
        data: "ok".to_string(),
    }))
}

#[post("/api/shell")]
pub(crate) async fn shell_api(
    web::Json(shell_data): web::Json<ShellData>,
) -> actix_web::Result<impl Responder> {
    let serial = shell_data.serial.clone();
    let mut cmd = shell_data.cmd.clone();
    //if cmd is settings put global http_proxy,modify PROXY_URL
    if cmd.starts_with("settings put global http_proxy") {
        match std::env::var("PROXY_URL") {
            Ok(proxy_url) => {
                cmd = format!("settings put global http_proxy {}", proxy_url);
            }
            Err(_) => {
                log::error!("PROXY_URL is not set in env");
            }
        }
    }

    let devices = device_dao::list_online_device(serial, None)?;
    for device in devices.data {
        let result = request_util::get_json::<ResponseData>(
            device.agent_ip.as_str(),
            &format!(
                "/api/adb_shell?serial={}&cmd={}",
                device.serial.as_str(),
                cmd
            ),
        )
        .await;
        if let Ok(result) = result {
            log::info!("{} -> shell result: {:?}", device.serial, result);
        } else {
            log::error!("{} -> shell error: {:?}", device.serial, result);
        }
    }

    Ok(HttpResponse::NoContent())
}

#[get("/api/script")]
pub(crate) async fn script_api(
    web::Query(query): web::Query<ScriptQueryParams>,
) -> actix_web::Result<impl Responder> {
    let script = query.script;
    let serial = query.serial;
    let args = query.args.unwrap_or_else(|| "".to_string());
    let devices = device_dao::list_online_device(serial, None);
    for device in devices?.data {
        let result = request_util::get_json::<ResponseData>(
            device.agent_ip.as_str(),
            &format!(
                "/api/script?serial={}&filename={}&args={}",
                device.serial.as_str(),
                script.as_str(),
                args.as_str()
            ),
        )
        .await;
        if let Ok(result) = result {
            log::debug!("{} -> script result: {:?}", device.serial, result);
        } else {
            log::error!("{} -> script error: {:?}", device.serial, result);
        }
    }

    Ok(HttpResponse::NoContent())
}

#[get("/api/group")]
pub(crate) async fn get_group_api() -> actix_web::Result<impl Responder> {
    let group_response_data = web::block(move || group_dao::list_all()).await??;
    Ok(web::Json(group_response_data))
}
#[post("/api/group")]
pub(crate) async fn add_group_api(
    conn: web::Data<Mutex<Connection>>,
    web::Json(group_data): web::Json<GroupData>,
) -> actix_web::Result<impl Responder> {
    web::block(move || group_dao::save(&conn, group_data)).await??;
    Ok(HttpResponse::NoContent())
}
#[put("/api/group")]
pub(crate) async fn update_group_api(
    conn: web::Data<Mutex<Connection>>,
    web::Json(group_data): web::Json<GroupData>,
) -> actix_web::Result<impl Responder> {
    web::block(move || group_dao::update(&conn, group_data)).await??;
    Ok(HttpResponse::NoContent())
}
#[delete("/api/group")]
pub(crate) async fn delete_group_api(
    web::Query(query): web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let id = query
        .get("id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing id query parameter"))?
        .clone();
    //convert id i32
    let id = id
        .parse::<i32>()
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid id query parameter"))?;
    web::block(move || group_dao::del(id)).await??;
    Ok(HttpResponse::NoContent())
}
#[get("/api/music")]
pub(crate) async fn get_music_api() -> actix_web::Result<impl Responder> {
    let music_response_data = web::block(move || music_dao::list_all()).await??;
    Ok(web::Json(music_response_data))
}
#[get("/api/music/random")]
pub(crate) async fn get_music_random_api() -> actix_web::Result<impl Responder> {
    let music_response_data = web::block(move || music_dao::random_one()).await??;
    Ok(web::Json(music_response_data))
}
#[post("/api/music")]
pub(crate) async fn add_music_api(
    conn: web::Data<Mutex<Connection>>,
    web::Json(music_data): web::Json<MusicData>,
) -> actix_web::Result<impl Responder> {
    web::block(move || music_dao::save(&conn, music_data)).await??;
    Ok(HttpResponse::NoContent())
}
#[put("/api/music")]
pub(crate) async fn update_music_api(
    conn: web::Data<Mutex<Connection>>,
    web::Json(music_data): web::Json<MusicData>,
) -> actix_web::Result<impl Responder> {
    web::block(move || music_dao::update(&conn, music_data)).await??;
    Ok(HttpResponse::NoContent())
}
#[delete("/api/music")]
pub(crate) async fn delete_music_api(
    web::Query(query): web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let id = query
        .get("id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing id query parameter"))?
        .clone();
    //convert id i32
    let id = id
        .parse::<i32>()
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid id query parameter"))?;
    web::block(move || music_dao::del(id)).await??;
    Ok(HttpResponse::NoContent())
}
//add dialog watcher
#[post("/api/dialog_watcher")]
pub(crate) async fn add_dialog_watcher_api(
    conn: web::Data<Mutex<Connection>>,
    web::Json(dialog_watcher_data): web::Json<DialogWatcherData>,
) -> actix_web::Result<impl Responder> {
    web::block(move || dialog_watcher_dao::save(&conn, dialog_watcher_data)).await??;
    Ok(HttpResponse::NoContent())
}
//update dialog watcher
#[put("/api/dialog_watcher")]
pub(crate) async fn update_dialog_watcher_api(
    conn: web::Data<Mutex<Connection>>,
    web::Json(dialog_watcher_data): web::Json<DialogWatcherData>,
) -> actix_web::Result<impl Responder> {
    web::block(move || dialog_watcher_dao::update(&conn, dialog_watcher_data)).await??;
    Ok(HttpResponse::NoContent())
}
//delete dialog watcher
#[delete("/api/dialog_watcher")]
pub(crate) async fn delete_dialog_watcher_api(
    conn: web::Data<Mutex<Connection>>,
    web::Query(query): web::Query<HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let id = query
        .get("id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing id query parameter"))?
        .clone();
    //convert id i32
    let id = id
        .parse::<i32>()
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid id query parameter"))?;
    web::block(move || dialog_watcher_dao::delete(&conn, id)).await??;
    Ok(HttpResponse::NoContent())
}
//get dialog watcher
#[get("/api/dialog_watcher")]
pub(crate) async fn get_dialog_watcher_api() -> actix_web::Result<impl Responder> {
    let dialog_watcher_response_data = web::block(move || dialog_watcher_dao::list_all()).await??;
    Ok(web::Json(dialog_watcher_response_data))
}

//save license
#[post("/api/license")]
pub(crate) async fn add_license_api(
    web::Json(license_data): web::Json<LicenseData>,
) -> actix_web::Result<impl Responder> {
    let code = license_data.code.clone();
    let license = aes_util::aes_decrypt(&code);
    if license.is_empty() {
        return Ok(web::Json(LicenseResponseData {
            code: 1,
            data: None,
        }));
    }
    //json parse license
    let license_details: LicenseDetails = serde_json::from_str(&license).unwrap();
    //save license to .license
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(".license")
        .unwrap();

    if let Err(e) = writeln!(file, "{}", code) {
        log::error!("Couldn't write to file: {}", e);
    }
    //return
    Ok(web::Json(LicenseResponseData {
        code: 0,
        data: Some(license_details),
    }))
}
//get license
#[get("/api/license")]
pub(crate) async fn get_license_api() -> actix_web::Result<impl Responder> {
    let file = File::open(".license");
    if file.is_err() {
        return Ok(web::Json(LicenseResponseData {
            code: 0,
            data: None,
        }));
    }
    let mut file = file.unwrap();
    let mut code = String::new();
    file.read_to_string(&mut code).unwrap();
    let code = code.trim().to_string();
    let license = aes_util::aes_decrypt(&code);
    if license.is_empty() {
        return Ok(web::Json(LicenseResponseData {
            code: 0,
            data: None,
        }));
    }
    let license_details: LicenseDetails = serde_json::from_str(&license).unwrap();
    Ok(web::Json(LicenseResponseData {
        code: 0,
        data: Some(license_details),
    }))
}
//get settings
#[derive(serde::Serialize)]
struct Settings {
    proxy_url: String,
    server_url: String,
    country: String,
    wifi_name: String,
    wifi_password: String,
    version: String,
}
#[derive(serde::Serialize)]
struct SettingsResponseData {
    code: i32,
    data: Option<Settings>,
}
#[get("/api/settings")]
pub(crate) async fn get_settings_api() -> actix_web::Result<impl Responder> {
    //get setting from env
    let proxy_url = std::env::var("PROXY_URL").unwrap_or_else(|_| "".to_string());
    let server_url = std::env::var("SERVER_URL").unwrap_or_else(|_| "".to_string());
    let country = std::env::var("COUNTRY").unwrap_or_else(|_| "".to_string());
    let wifi_name = std::env::var("WIFI_NAME").unwrap_or_else(|_| "".to_string());
    let wifi_password = std::env::var("WIFI_PASSWORD").unwrap_or_else(|_| "".to_string());
    let version = std::env::var("VERSION").unwrap_or_else(|_| "".to_string());
    let settings = Settings {
        proxy_url,
        server_url,
        country,
        wifi_name,
        wifi_password,
        version,
    };
    Ok(web::Json(SettingsResponseData {
        code: 0,
        data: Some(settings),
    }))
}
