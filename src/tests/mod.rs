#[cfg(test)]
mod tests {
    use flexi_logger::Logger;

    use crate::{aes_util, database, models::LicenseDetails};
    #[test]
    fn test_db() {
        let result = database::create_databases();
        match result {
            Ok(_) => {
                println!("create database success");
            }
            Err(e) => {
                println!("create database error:{}", e);
            }
        }
    }

    #[test]
    fn test_aes() {
        Logger::try_with_str("info").unwrap().start().unwrap();

        let expire = (chrono::Local::now() + chrono::Duration::days(30)).timestamp();
        let license_detail = LicenseDetails {
            name: "Niostack".to_string(),
            expire,
        };
        let result =
            aes_util::aes_encrypt(serde_json::to_string(&license_detail).unwrap().as_str());
        log::error!("aes encrypt result:{}", result);
        let result = aes_util::aes_decrypt(&result);
        log::error!("aes decrypt result:{}", result);
    }
}
