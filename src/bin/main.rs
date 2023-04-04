use std::ffi::{c_double, CStr, CString, c_char};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use serde_json::{Value, json};
use serde::{Serialize, Deserialize};
use telegram_drive::cloud::{Cloud, CloudError};
use telegram_drive::TelegramBackend;

use telegram_drive_core::*;

const WAIT_TIMEOUT: f64 = 2.0;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    message_id: i64,
    content: Value
}

fn main() {

    println!("{:?}", TDApp::execute_query(&json!(
        {
            "@type": "setLogVerbosityLevel",
            "new_verbosity_level": 1
        }
    ).to_string()).unwrap());

    let telegram_backend = TelegramBackend::new();
    println!("Backend has been created!");

    let mut cloud = Cloud::new(telegram_backend);

    cloud.upload_file(Path::new(r"C:\Users\nikiy\Downloads\PLvs8_Kv0hU.jpg"), Path::new("fs://")).unwrap();

    println!("\n\n\n\nСкачивание\n\n\n\n");

    thread::sleep(Duration::new(5,0));

    cloud.download_file(Path::new(r"fs://PLvs8_Kv0hU")).expect("eewewgwgw");


    loop {

    }
}

/*
    println!("NNNNNNNN\n\n\n\n{:?}\n\n\n\n", TDApp::execute_query(&json!({
        "@type": "sendMessage",
        "input_message_content": {
            "@type": "inputMessageDocument",
            "document": {
                "@type": "inputFileLocal",
                "path": "C:\\Users\\nikiy\\Downloads\\FileSeparation10x64.msi"
            }
        }

    }).to_string()).unwrap());
}


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


//Обновление =>> {"@client_id":1,"@type":"updateDeleteMessages","chat_id":-1001418440636,"from_cache":true,"is_permanent":false,"message_ids":[14568914944]}
