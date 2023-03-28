use rusqlite::Connection;

#[path ="database_handler.rs"]
mod database_handler;

#[derive(Debug)]
struct Pages {
    page_name: String,
    page_configuration: String,
    data: Option<Vec<u8>>,
}


#[derive(Debug)]
struct Files {
    file_name: String,
    page_name: String,
    data: Option<Vec<u8>>,
}

pub fn get_database(page_name: &str) -> Connection{
    database_handler::get_database(page_name, "CREATE TABLE pages (
        page_name TEXT PRIMARY KEY,
        page_configuration TEXT NOT NULL,
        data BLOB);
    CREATE TABLE files (
        file_name TEXT PRIMARY KEY,
        page_name TEXT NOT NULL,
        data BLOB NOT NULL;
    data BLOB)")
}

/*

    let page = Pages {
        page_name: page_name.to_string(),
        page_configuration: "{lol:0}".to_string(),
        data: None,
    };
    conn.execute(
        "INSERT INTO person (name, data) VALUES (?1, ?2)",
        (&me.name, &me.data),
    )?;

    let mut stmt = conn.prepare("SELECT id, name, data FROM person")?;
    let person_iter = stmt.query_map([], |row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            data: row.get(2)?,
        })
    })?;

    for person in person_iter {
        println!("Found person {:?}", person.unwrap());
    }
    */
