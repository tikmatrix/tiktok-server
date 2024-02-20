use crate::ddl_actor::DdlActor;
use crate::ddl_actor::DdlMessage;
use crate::job_schedu::JobScheduActor;
use crate::offline_checker::OfflineCheckerActor;
use actix::Actor;
use actix_cors::Cors;
use actix_files as fs;
use actix_multipart::form::{tempfile::TempFileConfig, MultipartFormConfig};
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use flexi_logger::Age;
use flexi_logger::Cleanup;
use flexi_logger::Criterion;
use flexi_logger::Naming;
use flexi_logger::{FileSpec, WriteMode};
use std::io;
use std::sync::Arc;
use std::sync::Mutex;
mod account_dao;
mod avatar_dao;
mod comment_dao;
mod database;
mod ddl_actor;
mod device_dao;
mod dialog_watcher_dao;
mod group_dao;
mod job_schedu;
mod material_dao;
mod models;
mod music_dao;
mod offline_checker;
mod publish_job_dao;
mod request_util;
mod routes;
mod runtime_err;
mod tests;
mod train_job_dao;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // initialize logger
    let _logger = flexi_logger::Logger::try_with_str("info")
        .unwrap()
        .log_to_file(FileSpec::default().suppress_timestamp().directory("logs"))
        .rotate(
            // If the program runs long enough,
            Criterion::Age(Age::Day), // - create a new file every day
            Naming::Timestamps,       // - let the rotated files have a timestamp in their name
            Cleanup::KeepLogFiles(7), // - keep at most 7 log files
        )
        .duplicate_to_stderr(flexi_logger::Duplicate::Info)
        .format(flexi_logger::colored_with_thread)
        .write_mode(WriteMode::BufferAndFlush)
        .start()
        .expect("flexi_logger init error");
    routes::setup_env();
    //init sqlite
    database::create_databases().expect("create sqlite database error");
    let conn = database::get_conn().expect("get sqlite connection error");
    let conn_mutex = Mutex::new(conn);
    let conn_data = web::Data::new(conn_mutex);
    let _addr = JobScheduActor {
        conn: conn_data.clone(),
    }
    .start();
    let _addr = OfflineCheckerActor {}.start();
    let ddl_actor_addr = DdlActor {}.start();
    //创建一个消息通道
    let (tx, rx) = std::sync::mpsc::channel::<DdlMessage>();
    let ddl_sender_data = web::Data::new(Arc::new(Mutex::new(tx.clone())));
    // 创建一个线程，循环接收消息
    std::thread::spawn(move || loop {
        let msg = rx.recv().unwrap();
        // 将消息发送给 Actor
        ddl_actor_addr.do_send(msg);
    });
    log::info!("starting HTTP server at port 8090 with 2 workers");
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(conn_data.clone())
            .app_data(TempFileConfig::default().directory("./tmp"))
            //默认限制50M上传,修改为1GB
            .app_data(
                MultipartFormConfig::default()
                    .total_limit(1024 * 1024 * 1024 * 5)
                    .memory_limit(1024 * 1024 * 100),
            )
            .app_data(ddl_sender_data.clone())
            .service(routes::add_account_api)
            .service(routes::get_account_api)
            .service(routes::update_account_api)
            .service(routes::get_account_by_device_api)
            .service(routes::get_account_auto_train_api)
            .service(routes::delete_account_api)
            .service(routes::add_material_api)
            .service(routes::get_material_api)
            .service(routes::get_material_count_api)
            .service(routes::update_material_api)
            .service(routes::delete_material_api)
            .service(routes::add_job_api)
            .service(routes::get_job_api)
            .service(routes::update_job_api)
            .service(routes::delete_job_api)
            .service(routes::add_train_job_api)
            .service(routes::get_train_job_api)
            .service(routes::runable_train_job_api)
            .service(routes::update_train_job_api)
            .service(routes::delete_train_job_api)
            .service(routes::add_device_api)
            .service(routes::get_device_api)
            .service(routes::get_device_init_api)
            .service(routes::task_status_api)
            .service(routes::shell_api)
            .service(routes::script_api)
            .service(routes::install_api)
            .service(routes::runable_publish_job_api)
            .service(routes::get_group_api)
            .service(routes::add_group_api)
            .service(routes::update_group_api)
            .service(routes::delete_group_api)
            .service(routes::get_music_api)
            .service(routes::get_music_random_api)
            .service(routes::add_music_api)
            .service(routes::update_music_api)
            .service(routes::delete_music_api)
            .service(routes::get_dialog_watcher_api)
            .service(routes::add_dialog_watcher_api)
            .service(routes::update_dialog_watcher_api)
            .service(routes::delete_dialog_watcher_api)
            .service(routes::get_settings_api)
            .service(routes::update_settings_api)
            .service(routes::gen_name_api)
            .service(routes::gen_bio_api)
            .service(routes::gen_email_api)
            .service(routes::add_avatar_api)
            .service(routes::get_avatar_api)
            .service(routes::delete_avatar_api)
            .service(routes::get_avatar_random_api)
            .service(routes::update_username_api)
            .service(routes::get_license_api)
            .service(routes::add_license_api)
            .service(routes::count_all_account_api)
            .service(routes::count_online_device_api)
            .service(routes::count_publish_job_by_status_api)
            .service(routes::count_train_job_by_status_api)
            .service(routes::retry_all_train_job_api)
            .service(routes::retry_all_publish_job_api)
            .service(routes::count_account_by_group_id_api)
            .service(routes::add_post_comment_api)
            .service(routes::add_post_comment_topic_api)
            .service(routes::get_post_comment_api)
            .service(routes::get_runable_comment_job_api)
            .service(fs::Files::new("/avatar", "./upload/avatar/").index_file("index.html"))
            .service(fs::Files::new("/apk", "./upload/apk/").index_file("index.html"))
            .service(fs::Files::new("/material", "./upload/material/").index_file("index.html"))
            .service(fs::Files::new("/", "./bin/dist/").index_file("index.html"))
    })
    .bind(("0.0.0.0", 18090))?
    .workers(2)
    .run()
    .await
}
