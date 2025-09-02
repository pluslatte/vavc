use rusqlite::Connection;
use vrchatapi::models::Avatar;

pub fn create_avatar_db() -> Result<(), rusqlite::Error> {
    let conn = Connection::open("./avatars.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS avatars (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT,
        version INTEGER,
        thumbnail_image_url TEXT,
        created_at TEXT,
        updated_at TEXT
      )",
        [],
    )?;

    Ok(())
}

pub fn create_alias_db() -> Result<(), rusqlite::Error> {
    let conn = Connection::open("./avatars.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS aliases (
      name TEXT PRIMARY KEY,
      avatar_id TEXT NOT NULL
    )",
        [],
    )?;

    Ok(())
}

pub fn rebuild_avatar_db(avatars: Vec<Avatar>) -> Result<(), rusqlite::Error> {
    let conn = Connection::open("./avatars.db")?;

    conn.execute("DROP TABLE IF EXISTS avatars", [])?;
    create_avatar_db()?;
    insert_avatars(avatars)?;

    println!("Rebuilt avatar database.");
    Ok(())
}

fn insert_avatars(avatars: Vec<Avatar>) -> Result<(), rusqlite::Error> {
    let conn = Connection::open("./avatars.db")?;

    for avatar in avatars {
        if let Err(e) = conn.execute(
      "INSERT INTO avatars (id, name, description, version, thumbnail_image_url, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
          [
            &avatar.id,
            &avatar.name,
            &avatar.description,
            &avatar.version.to_string(),
            &avatar.thumbnail_image_url,
            &avatar.created_at,
            &avatar.updated_at,
        ]) {
          eprintln!("Failed to insert avatar {}: {}", avatar.id, e);
        };
    }

    Ok(())
}

pub fn register_alias(alias: &str, avatar_id: &str) -> Result<(), rusqlite::Error> {
    let conn = Connection::open("./avatars.db")?;

    let result = conn.execute(
        "INSERT INTO aliases (name, avatar_id) VALUES (?1, ?2)
         ON CONFLICT(name) DO NOTHING",
        [alias, avatar_id],
    )?;

    if result == 0 {
        println!("Alias '{}' already exists. No changes made.", alias);
    } else {
        println!("Made new alias {} -> {}", alias, avatar_id);
    }

    Ok(())
}

pub fn remove_alias(alias: &str) -> Result<(), rusqlite::Error> {
    let conn = Connection::open("./avatars.db")?;

    let result = conn.execute("DELETE FROM aliases WHERE name = ?1", [alias])?;

    if result == 0 {
        println!("Alias '{}' does not exist. No changes made.", alias);
    } else {
        println!("Removed alias '{}'.", alias);
    }

    Ok(())
}

pub fn get_avatar_id_by_alias(alias: &str) -> Result<String, rusqlite::Error> {
    let conn = Connection::open("./avatars.db")?;

    let mut stmt = conn.prepare("SELECT avatar_id FROM aliases WHERE alias = ?1")?;
    let mut rows = stmt.query([alias])?;

    if let Some(row) = rows.next()? {
        let avatar_id: String = row.get(0)?;
        Ok(avatar_id)
    } else {
        Err(rusqlite::Error::QueryReturnedNoRows)
    }
}
