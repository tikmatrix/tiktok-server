use magic_crypt::{new_magic_crypt, MagicCryptTrait};
pub fn aes_encrypt(input: &str) -> String {
    let mc = new_magic_crypt!("0s6r2m28ns7xecmc", 256);
    mc.encrypt_str_to_base64(input)
}

pub fn aes_decrypt(input: &str) -> String {
    let mc = new_magic_crypt!("0s6r2m28ns7xecmc", 256);
    let output: Result<String, magic_crypt::MagicCryptError> = mc.decrypt_base64_to_string(input);
    if output.is_err() {
        return "".to_string();
    }
    output.unwrap()
}
