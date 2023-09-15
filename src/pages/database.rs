use dirs;
use filetime::FileTime;
use std::{fs, path::Path};
static PROGRAM_NAME: &str = "daytheipc-com";
use serde::{Deserialize, Serialize};

//THIS GETS A DATABASE AND CHECKS IF IT EXISTS
//pub fn get_database(rules: Vec<&str>) -> Connection {
//    let db_name = "database";
//    let dir = format!("{}/db/{}.db", get_program_files_location(), db_name.clone());
//
//    let file = File::open(dir.clone());
//    let connection = Connection::open(dir).expect("Database to be get or created");
//    match file {
//        Ok(_) => {
//            println!("{} already exists! Skipping this step.", db_name.clone());
//        }
//        Err(_) => {
//            println!(
//                "{} does not exists! Creating database file.",
//                db_name.clone()
//            );
//            for rule in rules {
//                match connection.execute(rule, params![]) {
//                    Ok(_) => {}
//                    Err(e) => {
//                        println!(
//                            "Error happened when trying to create {} tables: \n{}",
//                            db_name.clone(),
//                            e
//                        )
//                    }
//                }
//            }
//        }
//    }
//    connection
//}




pub fn get_program_files_location() -> String {
    format!(
        "{}/{}",
        dirs::home_dir().unwrap().to_str().unwrap().to_string(),
        PROGRAM_NAME
    )
}

#[derive(Serialize)]
pub struct Directory {
    pub(crate) name: String,
    pub(crate) directories: Vec<Directory>,
    pub(crate) files: Vec<FileInfo>,
}

#[derive(Serialize)]
pub struct FileInfo {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) modified: String,
}

pub fn get_directory_file_names(root: String, name: Option<String>) -> Result<Directory, String> {
    let mut dir: Directory = Directory {
        name: name.unwrap_or("root".into()),
        directories: vec![],
        files: vec![],
    };

    let read_dir = match fs::read_dir(root) {
        Ok(read_dir) => read_dir,
        Err(_) => return Err("Not found!".into()),
    };

    for path in read_dir.filter_map(|p| p.ok()) {
        let metadata = path.metadata().unwrap();
        if metadata.is_dir() {
            dir.directories.push(
                get_directory_file_names(
                    path.path().display().to_string(),
                    Some(path.file_name().into_string().unwrap()),
                )
                .unwrap(),
            );
        } else if metadata.is_file() {
            let file = FileInfo {
                name: path.file_name().into_string().unwrap(),
                path: path.path().display().to_string(),
                modified: FileTime::from_last_modification_time(&metadata)
                    .unix_seconds()
                    .to_string(),
            };

            dir.files.push(file);
        }
    }
    Ok(dir)
}
