use crate::{device_dao, models::ResponseData};
use actix::prelude::*;
use std::time::Duration;

use super::request_util;
pub struct OfflineCheckerActor {}
impl Actor for OfflineCheckerActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        actix_rt::spawn(async move {
            check().await;
        });
        self.schedule_check(ctx);
    }
}

impl OfflineCheckerActor {
    fn schedule_check(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(Duration::from_secs(10), move |_actor, _ctxx| {
            actix_rt::spawn(async move {
                check().await;
            });
        });
    }
}

async fn check() {
    log::debug!("check offline devices");
    let online_devices = device_dao::list_online_device(None, None);
    if online_devices.is_err() {
        log::error!(
            "get online devices failed with error: {}",
            online_devices.err().unwrap()
        );
        return;
    }
    let online_devices = online_devices.unwrap().data;
    for device in online_devices {
        let serial = device.serial.clone();
        let host = device.agent_ip.clone();
        let result = request_util::get_json::<ResponseData<String>>(
            &host,
            &format!("/api/is_online?serial={}", &serial),
        )
        .await;
        if result.is_ok() && result.unwrap().data == "online" {
            continue;
        }
        log::warn!("device: {} offline", &serial);
        let result = device_dao::update_online(&serial, &0);
        if result.is_err() {
            log::error!(
                "update device: {} online failed with error: {}",
                &serial,
                result.err().unwrap()
            );
        }
    }
}
