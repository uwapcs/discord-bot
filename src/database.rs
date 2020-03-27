use diesel::prelude::*;
use diesel::result::Error;
use diesel::sqlite::SqliteConnection;

// TODO reuse DB connection, using r2d2 or something

use crate::ldap::*;

#[table_name = "members"]
#[derive(Queryable, AsChangeset, Insertable)]
pub struct Member {
    pub discord_id: i64,
    pub tla: Option<String>,
    pub username: String,
    pub member_since: Option<String>,
    pub name: Option<String>,
    pub biography: Option<String>,
    pub github: Option<String>,
    pub photo: Option<String>,
    pub website: Option<String>,
}

table! {
    members (discord_id) {
        discord_id -> BigInt,
        tla -> Nullable<Text>,
        username -> Text,
        member_since -> Nullable<Text>,
        name -> Nullable<Text>,
        biography -> Nullable<Text>,
        github -> Nullable<Text>,
        photo -> Nullable<Text>,
        website -> Nullable<Text>,
    }
}

pub fn db_connection() -> SqliteConnection {
    SqliteConnection::establish("state.db").expect("Failed to connect to sqlite DB")
}

pub fn add_member(discord_id: &u64, username: &str) -> Member {
    let ldap_user = ldap_search(username);
    let name = ldap_user.as_ref().map(|u| u.name.clone());
    let tla_user = tla_search(username);
    let tla = tla_user.as_ref().map(|u| u.tla.clone()).flatten();
    let new_member = Member {
        discord_id: *discord_id as i64,
        username: username.to_string(),
        name: name.clone(),
        tla: tla,
        member_since: None,
        biography: None,
        github: None,
        photo: None,
        website: None,
    };
    diesel::insert_into(members::table)
        .values(&new_member)
        .execute(&db_connection())
        .expect("Failed to add member to DB");
    info!(
        "{} added to member DB",
        name.unwrap_or(discord_id.to_string())
    );
    new_member
}

#[allow(dead_code)] // remove this if you start using it
pub fn update_member(discord_id: &u64, member: Member) -> Result<usize, Error> {
    diesel::update(members::table.find(*discord_id as i64))
        .set(&member)
        .execute(&db_connection())
}

pub fn username_exists(username: &str) -> bool {
    get_member_info_from_username(username).is_ok()
}

pub fn get_member_info(discord_id: &u64) -> Result<Member, Error> {
    members::table
        .find(*discord_id as i64)
        .first(&db_connection())
}

pub fn get_member_info_from_username(username: &str) -> Result<Member, Error> {
    members::table
        .filter(members::username.eq(username))
        .first(&db_connection())
}

pub fn get_member_info_from_tla(tla: &str) -> Result<Member, Error> {
    members::table
        .filter(members::tla.eq(tla))
        .first(&db_connection())
}

pub fn set_member_bio(discord_id: &u64, bio: &str) -> Result<usize, Error> {
    diesel::update(members::table.find(*discord_id as i64))
        .set(members::biography.eq(bio))
        .execute(&db_connection())
}

pub fn set_member_git(discord_id: &u64, git: &str) -> Result<usize, Error> {
    diesel::update(members::table.find(*discord_id as i64))
        .set(members::github.eq(git))
        .execute(&db_connection())
}

pub fn set_member_photo(discord_id: &u64, url: &str) -> Result<usize, Error> {
    diesel::update(members::table.find(*discord_id as i64))
        .set(members::photo.eq(url))
        .execute(&db_connection())
}

pub fn set_member_website(discord_id: &u64, url: &str) -> Result<usize, Error> {
    diesel::update(members::table.find(*discord_id as i64))
        .set(members::website.eq(url))
        .execute(&db_connection())
}
