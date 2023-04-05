use std::io::Write;
use serde_json::{Value, json};
use super::TDApp;

pub struct TdlibParameters {
    use_test_dc: Option<bool>,
    database_directory: Option<String>,
    files_directory: Option<String>,
    use_file_database: Option<bool>,
    use_chat_info_database: Option<bool>,
    use_message_database: Option<bool>,
    use_secret_chats: Option<bool>,
    api_id: i32,
    api_hash: String,
    system_language_code: String,
    device_model: String,
    system_version: Option<String>,
    application_version: String,
    enable_storage_optimizer: Option<bool>,
    ignore_file_names: Option<bool>
}

pub fn set_tdlib_parameters(app: &TDApp, parameters: Option<TdlibParameters>) {

    println!("Sending TdlibParameters");

    let parameters = match parameters {
        Some(_) => panic!("Unsupported custom tdlib_parameters"),
        None => {
            json!({
                "@type": "setTdlibParameters",
                    "database_directory": "td\\db\\",
                    "files_directory": "td\\file\\",
                    "use_file_database": true,
                    "use_chat_info_database": true,
                    "use_message_database": true,
                    "use_secret_chats": false,
                    "api_id": 28978068,
                    "api_hash": "ba63854dbf668b8a2c8a24330ef6fc5b",
                    "system_language_code": "ru",
                    "device_model": "Desktop",
                    "system_version": "1.0.0",
                    "application_version": "1.0",
                    "enable_storage_optimizer": true
            })

            // json!({
            //     "@type": "setTdlibParameters",
            //     "database_directory": "tdlib",
            //     "use_message_database": true,
            //     "use_secret_chats": true,
            //     "api_id": 28978068,
            //     "api_hash": "ba63854dbf668b8a2c8a24330ef6fc5b",
            //     "system_language_code": "en",
            //     "device_model": "Desktop",
            //     "application_version": "1.0",
            //     "enable_storage_optimizer": true,
            // })
        }
    };
    app.send_query(&parameters.to_string()).unwrap();
}

pub fn set_phone_number(app: &TDApp) {
    std::io::stdout().flush().unwrap();
    println!("Введите свой номер телефона");
    std::io::stdout().flush().unwrap();

    let mut phone_input = String::new();
    std::io::stdin().read_line(&mut phone_input).unwrap();

    app.send_query(&json!({
        "@type": "setAuthenticationPhoneNumber",
        "phone_number": phone_input
    }).to_string()).unwrap();
}

pub fn check_code(app: &TDApp) {
    println!("Введите код");
    std::io::stdout().flush().unwrap();

    let mut code_input = String::new();
    std::io::stdin().read_line(&mut code_input).unwrap();

    app.send_query(&json!({
        "@type": "checkAuthenticationCode",
        "code": code_input
    }).to_string()).unwrap();
}

pub fn check_password(app: &TDApp) {
    println!("Введите пароль");
    std::io::stdout().flush().unwrap();

    let mut password_input = String::new();
    std::io::stdin().read_line(&mut password_input).unwrap();

    password_input = password_input
        .trim_matches(|c: char| c.is_whitespace())
        .to_string();

    app.send_query(&json!({
        "@type": "checkAuthenticationPassword",
        "password": password_input
    }).to_string()).unwrap();
}