use actix::prelude::*;
use actix_web::web;
use rand::{seq::SliceRandom, thread_rng};
use rusqlite::Connection;

use std::{sync::Mutex, time::Duration};

use crate::{
    dao::account_dao,
    dao::group_dao,
    dao::material_dao,
    dao::publish_job_dao,
    dao::train_job_dao,
    models::{PublishJobData, TrainJobData},
};

pub struct JobScheduActor {
    pub conn: web::Data<Mutex<Connection>>,
}
impl Actor for JobScheduActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.schedule_check(ctx);
    }
}

impl JobScheduActor {
    fn schedule_check(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_later(Duration::from_secs(60), |act: &mut JobScheduActor, ctx| {
            act.check_publish_job();
            act.check_train_job();
            act.schedule_check(ctx);
        });
    }
    fn check_train_job(&self) {
        //list all auto train group
        let result = group_dao::list_all_auto_train();
        if let Ok(data) = result {
            for group in data.data {
                //check open auto train
                if group.auto_train != 1 {
                    continue;
                }
                //get account in group
                let result = account_dao::list_account_by_group_id(group.id);
                if let Ok(account_data) = result {
                    //check has unfinish train_job by account
                    for account in account_data.data {
                        let start_times = group.train_start_time.split(",");
                        for start_time in start_times {
                            let today = chrono::Local::now()
                                .naive_local()
                                .format("%Y-%m-%d")
                                .to_string();
                            let start_time = format!("{} {}:00", today, start_time);
                            //if start_time < now, continue
                            if chrono::NaiveDateTime::parse_from_str(
                                &start_time,
                                "%Y-%m-%d %H:%M:%S",
                            )
                            .unwrap()
                                < chrono::Local::now().naive_local()
                            {
                                continue;
                            }
                            let account = account.clone();
                            let uesrname = account.username;
                            let email = account.email;
                            let id = account.id;
                            if uesrname.is_none() {
                                log::warn!("account.username is none");
                                continue;
                            }
                            let username = uesrname.unwrap();
                            if username.is_empty() {
                                log::warn!("account.username is empty");
                                continue;
                            }
                            //check username is email
                            if username.is_empty() || username == email {
                                log::info!("username is empty or email,can't use it to publish");
                                continue;
                            }
                            let result: Result<i32, crate::runtime_err::RunTimeError> =
                                train_job_dao::count_job_by_account_today(
                                    id.clone(),
                                    start_time.to_owned(),
                                );
                            if let Ok(count) = result {
                                if count == 0 {
                                    //create train_job
                                    let group_clone = group.clone();
                                    let job_data = TrainJobData {
                                        id: None,
                                        group_id: Some(group_clone.id),
                                        account_id: Some(id.clone()),
                                        floow_probable: Some(group_clone.floow_probable),
                                        like_probable: Some(group_clone.like_probable),
                                        collect_probable: Some(group_clone.collect_probable),
                                        status: Some(0),
                                        start_time: Some(start_time.to_owned()),
                                        duration: Some(group_clone.train_duration),
                                    };
                                    let job_data_clone = job_data.clone();
                                    let result = train_job_dao::save(&self.conn, job_data_clone);
                                    if let Err(err) = result {
                                        log::warn!(
                                            "train_job_dao::save err -> {:?} -> {:?}",
                                            job_data,
                                            err
                                        );
                                        break;
                                    }
                                    log::info!("train_job_dao::save success -> {:?}", job_data);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    fn check_publish_job(&self) {
        //list all auto publish group
        let result = group_dao::list_all_auto_publish();
        if let Ok(data) = result {
            for group in data.data {
                //check open auto publish
                if group.auto_publish != 1 {
                    continue;
                }
                //get account in group
                let result = account_dao::list_account_by_group_id(group.id);
                if let Ok(account_data) = result {
                    //check has unfinish publish_job by account
                    for account in account_data.data {
                        let start_times = group.publish_start_time.split(",");
                        for start_time in start_times {
                            let group_clone = group.clone();
                            let today = chrono::Local::now()
                                .naive_local()
                                .format("%Y-%m-%d")
                                .to_string();
                            let start_time = format!("{} {}:00", today, start_time);
                            //if start_time < now, continue
                            if chrono::NaiveDateTime::parse_from_str(
                                &start_time,
                                "%Y-%m-%d %H:%M:%S",
                            )
                            .unwrap()
                                < chrono::Local::now().naive_local()
                            {
                                continue;
                            }
                            let account = account.clone();
                            let uesrname = account.username;
                            let email = account.email;
                            let id = account.id;
                            if uesrname.is_none() {
                                log::warn!("account.username is none");
                                continue;
                            }
                            let username = uesrname.unwrap();
                            if username.is_empty() {
                                log::warn!("account.username is empty");
                                continue;
                            }
                            //check username is email
                            if username.is_empty() || username == email {
                                log::info!("username is empty or email,can't use it to publish");
                                continue;
                            }
                            let result = publish_job_dao::count_job_by_account_today(
                                id.clone(),
                                start_time.clone(),
                            );
                            if let Ok(count) = result {
                                let start_time = start_time.clone();
                                if count == 0 {
                                    let mut material: String = "".to_string();
                                    if group_clone.publish_type == 1 {
                                        let result = material_dao::count(Some(0), Some(group.id));
                                        if let Ok(count) = result {
                                            if count == 0 {
                                                continue;
                                            }
                                        }
                                        //get material
                                        let result = material_dao::get_and_use_one(
                                            &self.conn,
                                            group_clone.id,
                                        );
                                        //if err, break
                                        if let Err(_) = result {
                                            log::warn!("get_and_use_one err");
                                            continue;
                                        }
                                        material = result.unwrap().name;
                                    }

                                    //create publish_job
                                    let group_clone = group.clone();
                                    //random get a title
                                    let title = group_clone.title.unwrap_or("".to_string());
                                    let title_lines: Vec<&str> = title
                                        .split("\n")
                                        .filter(|line| !line.trim().is_empty())
                                        .collect();

                                    let random_line = title_lines.choose(&mut thread_rng());

                                    let title = match random_line {
                                        Some(line) => *line,
                                        None => "",
                                    };
                                    let job_data = PublishJobData {
                                        id: None,
                                        material: Some(material),
                                        account_id: Some(id.clone()),
                                        title: Some(title.to_string()),
                                        status: Some(0),
                                        start_time: Some(start_time),
                                        group_id: Some(group_clone.id),
                                        publish_type: group_clone.publish_type,
                                        product_link: group_clone.product_link,
                                    };
                                    let job_data_clone = job_data.clone();
                                    let result = publish_job_dao::save(&self.conn, job_data_clone);
                                    if let Err(_) = result {
                                        log::warn!("publish_job_dao::save err -> {:?}", job_data);
                                        break;
                                    } else {
                                        log::info!(
                                            "publish_job_dao::save success -> {:?}",
                                            job_data
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
