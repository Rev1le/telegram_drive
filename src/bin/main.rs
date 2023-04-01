use std::ffi::{c_double, CStr, CString, c_char};
use std::io::Read;
use std::path::{Path, PathBuf};
use serde_json::{Value, json};
use serde::{Serialize, Deserialize};
use telegram_drive::cloud::{Cloud, CloudError};
use telegram_drive::MockCloudBackend;

use telegram_drive_core::*;

const WAIT_TIMEOUT: f64 = 2.0;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    message_id: i64,
    content: Value
}

fn main() {
    let mut app = TDApp::new();
    app.account_auth();

    let mut cloud = Cloud::new(app);
    //println!("JSON VFS: {}", cloud.get_fs_json());

    //println!("{:?}", cloud.get_folder(Path::new("fs://")).unwrap());

    println!("{:?}", TDApp::execute_query(r#"{"@type": "setLogVerbosityLevel", "new_verbosity_level": 0}"#).unwrap());

    let _ = cloud.upload_file(Path::new(r"C:\Users\nikiy\Downloads\Бабич Роман-2.pdf"), Path::new(r"f://Бабич Роман-2.pdf"));

    loop {

    }

    panic!("\n\nПодконтрльная паника");

    println!("{:?}", TDApp::execute_query(r#"{"@type": "setLogVerbosityLevel", "new_verbosity_level": 0}"#).unwrap());

    app.account_auth();

    /*
    loop {
        if let Some(str_json) = app.receive(WAIT_TIMEOUT) {
            let json = serde_json::from_str::<Value>(&str_json).expect("Неудачный парсинг");
            parse_response(json, &mut tele_drive);
        } else {
            println!("None");
        }

        let mut input = String::new();

        std::io::stdin().read_line(&mut input).unwrap();
        input = input.trim_matches(|c: char| c.is_whitespace()).to_string();

        match input.split(' ').collect::<Vec<&str>>()[0] {
            "s" => {
                app.send_query(&json!({
                    "@type": "createNewSupergroupChat",
                    "title": "TestChannel",
                    "is_channel": true,
                    "description": "For Debug"
                }).to_string()).unwrap();
            },
            "g" => {
                app.send_query(&json!({
                    "@type": "getChat",
                    "chat_id": -1001976761155_i64
                }).to_string()).unwrap();
            },
            "u" => {
                while let  Some(str_json) = app.receive(1.0) {
                    let json = serde_json::from_str::<Value>(&str_json).expect("Неудачный парсинг");
                    println!("Обновление =>> {}", json);
                }
            },
            "h" => {
                app.send_query(&json!({
                    "@type": "getChatHistory",
                    "chat_id": -1001976761155_i64,
                    "limit": 3
                }).to_string()).unwrap();
            },
            "w" => {
                std::fs::write("messages.json", serde_json::to_value(&tele_drive).unwrap().to_string().as_bytes()).unwrap();
            },
            "m" => {
                app.send_query(&json!({
                    "@type": "sendMessage",
                    "chat_id": -1001976761155_i64,
                    "message_thread_id": 0,
                    "reply_to_message_id": 0,
                    "input_message_content": {
                        "@type": "inputMessageDocument",
                        "document": {
                            "@type": "inputFileLocal",
                            "path": "W:\\tmp\\22b257ac-cb14-4d41-b930-62ac8f99dfd0_5.part"
                        }
                    }
                }).to_string()).unwrap();
            },
            _ => {}
        }
    }
    */
}
/*
"@type": "inputMessageDocument",
                        "document": {
                            "@type": "inputFileLocal",
                            "path": "C:\\Users\\nikiy\\Downloads\\FileSeparation10x64.msi"
                        }



"input_message_content": {
                        "@type": "inputMessageText",
                        "text": {
                            "@type": "formattedText",
                            "entities": [
                                {
                                    "@type": "textEntity",
                                    "length": 61,
                                    "offset": 0,
                                    "type": {
                                        "@type": "textEntityTypeTextUrl",
                                        "url": "https://ixbt.games/articles/2023/03/29/zapad-proigryvaet-bitvu-rynok-mobilnyx-igr-zaxvatyvaet-kitai.html"
                                    }
                                }
                            ],
                            "text": "Запад проиграл Востоку. Китай захватывает рынок мобильных игр\n\nВ этом году на рынке мобильных игр ожидаются несколько крупных релизов. Среди самых ожидаемых и заметных: ролевая игра Honkai Starrail от создателей Genshin Impact; Arena Breakout — шутер в стиле Escape from Tarkov от Tencent; Seven Deadly Sins: Origin от Netmarble. И это неполный список. Примечательно то, что все эти будущие хиты издаются азиатскими компаниями! Как получилось, что крупные западные издательства остались позади и способны только разочаровывать?"
                        }
                    }
*/

/*
fn parse_response(json: Value, tele_drive: &mut TelegramDriveChat) {

    match json["@type"].as_str().unwrap() {
        "updateOption" => {
            let option_name = json["name"].as_str().unwrap();
            let option_value = &json["value"];
            println!("==> Обновление опций. {} было обновлено на: {}\n", option_name, option_value)
        },
        "messages" => {
            println!("==> Сообщения. {}\n", json);
            let messages = json["messages"].as_array().unwrap();
            for message in messages {
                tele_drive.messages.push(Message {
                    message_id: message["id"].as_i64().unwrap(),
                    content: message["content"].clone(),
                });
            }
        },
        "updateNewChat" => {

        },
        "updateFile" => {
            let file_size = json["file"]["size"].as_f64().unwrap()/1024.00/1024.0;
            let uploaded_size = json["file"]["remote"]["uploaded_size"].as_f64().unwrap()/1024.0/1024.0;
            println!("==> Загрузка файла. Размер файла: {:4} МБ, загружено: {:4} МБ\n", file_size.round(), uploaded_size.round());
        },
        "updateChatLastMessage" => {
            let chat_id = json["chat_id"].as_i64().unwrap();
            let last_message_content = &json["last_message"]["content"];
            println!("==> Обновление чата. Чат: {} обновил последнее сообщение на: {}\n", chat_id, last_message_content);
        },
        "updateDeleteMessages" => {
            let chat_id = json["chat_id"].as_i64().unwrap();
            let delete_message_ids = &json["message_ids"];
            println!("==> Обновление чата. В чате: {} удалены сообщения: {}\n", chat_id, delete_message_ids);
        },
        "updateMessageSendSucceeded" => {

        }
        _ => println!("Обновление =>> {}\n", json),
    }
}

 */
//Обновление =>> {"@client_id":1,"@type":"updateDeleteMessages","chat_id":-1001418440636,"from_cache":true,"is_permanent":false,"message_ids":[14568914944]}
