#[cfg(test)]
mod tests {
    use flexi_logger::Logger;

    #[test]
    fn test_db() {
        let result = crate::database::create_databases();
        match result {
            Ok(_) => {
                println!("create database success");
            }
            Err(e) => {
                println!("create database error:{}", e);
            }
        }
    }
}
