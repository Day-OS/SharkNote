CREATE TABLE IF NOT EXISTS  user (
    user_id TEXT PRIMARY KEY UNIQUE,
    password TEXT NOT NULL,
    email TEXT NOT NULL,
    display_name TEXT UNIQUE,
    otp TEXT,
    configuration_json TEXT, /*CONTAINS DESCRIPTION, PRONOUNS, PROFILE PICTURE AND OTHER LESS SERIOUS STUFF*/
    is_program_admin INTEGER NOT NULL,
    account_status TEXT NOT NULL
);


/*ADMINISTRATION THINGS*/
CREATE TABLE IF NOT EXISTS page (
    owner_id TEXT,
    page_id TEXT,
    page_display_name TEXT,
    b_is_archived int NOT NULL DEFAULT 0,
    configuration_json TEXT, /*RON FILE WHERE CONFIGURATIONS ARE STORED*/
    PRIMARY KEY (page_id, owner_id)
    FOREIGN KEY (owner_id) REFERENCES user(user_id) ON DELETE CASCADE ON UPDATE CASCADE
);


CREATE TABLE IF NOT EXISTS  page_user (
    page_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    PRIMARY KEY (page_id, user_id)
    FOREIGN KEY (page_id) REFERENCES page(page_id) ON DELETE CASCADE ON UPDATE CASCADE
    FOREIGN KEY (user_id) REFERENCES user(user_id) ON DELETE CASCADE ON UPDATE CASCADE
);


CREATE TRIGGER IF NOT EXISTS default_page_display_name AFTER INSERT ON page
FOR EACH ROW WHEN NEW.page_display_name IS NULL
BEGIN 
    UPDATE page SET page_display_name = NEW.page_id WHERE rowid = NEW.rowid;
END;

/* HOW FILES ARE STORED! */

/* Each Page has 0 or more Publications */
/* A Publication has 0 or many Documents */
/* Document table points to a MarkDown file. These files are stored in the file table */


CREATE TABLE IF NOT EXISTS  file (
    page_id TEXT NOT NULL,
    file_id TEXT NOT NULL,
    file_content BLOB NOT NULL,
    PRIMARY KEY (page_id, file_id)
    FOREIGN KEY (page_id) REFERENCES page(page_id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS  last_file (
    page_id TEXT NOT NULL,
    file_id TEXT NOT NULL,
    ID INTEGER   NOT NULL,
    file_content BLOB NOT NULL,
    PRIMARY KEY (page_id, file_id, ID asc)
    FOREIGN KEY (page_id) REFERENCES page(page_id) ON DELETE CASCADE ON UPDATE CASCADE
    FOREIGN KEY (file_id) REFERENCES file(file_id) ON DELETE CASCADE ON UPDATE CASCADE
);



CREATE TRIGGER IF NOT EXISTS file_updated AFTER UPDATE ON file
BEGIN
    INSERT INTO last_file(page_id,file_id,ID,file_content) VALUES 
    (OLD.page_id, 
    OLD.file_id, 
    (SELECT ifnull(MAX(ID), 0) FROM last_file WHERE last_file.page_id = OLD.page_id AND last_file.file_id = OLD.file_id) + 1
    , OLD.file_content);
END;



CREATE TABLE IF NOT EXISTS  publication (
    page_id TEXT NOT NULL,
    publication_id TEXT NOT NULL,
    configuration_json TEXT, /*RON FILE WHERE CONFIGURATIONS ARE STORED*/
    created_at INTEGER,
    modified_at INTEGER,
    PRIMARY KEY (page_id, publication_id)
    FOREIGN KEY (page_id) REFERENCES page(page_id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS  document (
    page_id TEXT NOT NULL,
    document_id TEXT NOT NULL,
    file_id TEXT NOT NULL,
    PRIMARY KEY (page_id, document_id)
    FOREIGN KEY (page_id) REFERENCES page(page_id) ON DELETE CASCADE ON UPDATE CASCADE
    FOREIGN KEY (file_id) REFERENCES file(file_id) ON DELETE CASCADE ON UPDATE CASCADE
);


/* DEBUG REMOVE LATER!!!!!!
INSERT INTO user(user_id, password, is_program_admin) VALUES ("dayos", "ad15fb03d350b35372255f294f2b78aa6748bffd54a7b05ce85c48828122fe66", 1); 
*/

/*


You can do it in three steps (Wrapped in a transaction so it's atomic as far as other connections to the database are concerned), by first changing one of the PKs to a value that's not already in the database. Negative numbers work well, assuming your PKs are normally values >= 0, which is true of automatically generated rowid/INTEGER PRIMARY KEY values in Sqlite.

BEGIN;
UPDATE student SET Id=-Id WHERE Id=2861;
UPDATE student SET Id=2861 WHERE Id=3414;
UPDATE student SET Id=3414 WHERE Id=-2861;
COMMIT;



*/