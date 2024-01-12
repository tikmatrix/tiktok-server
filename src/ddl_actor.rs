use actix::prelude::*;

use crate::database;

pub struct DdlActor {}
impl Actor for DdlActor {
    type Context = Context<Self>;
}
pub struct DdlMessage {
    pub sql: String,
    pub params: Vec<rusqlite::types::Value>,
}
impl Message for DdlMessage {
    type Result = ();
}

impl Handler<DdlMessage> for DdlActor {
    type Result = ();

    fn handle(&mut self, msg: DdlMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let conn = database::get_conn();
        if let Err(e) = conn {
            log::error!("Failed to get conn: {:?}", e);
            return;
        }
        let conn = conn.unwrap();
        let result = conn.execute(msg.sql.as_str(), rusqlite::params_from_iter(msg.params));
        if let Err(e) = result {
            log::error!("Failed to execute ddl: {:?}", e);
            return;
        }
        log::debug!(
            "Success to execute ddl: {:?} -> {:?}",
            msg.sql,
            result.unwrap()
        );
    }
}
