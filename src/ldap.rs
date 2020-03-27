use ldap3::{LdapConn, LdapConnSettings, Scope, SearchEntry};
use crate::config::CONFIG;

#[derive(Debug)]
pub struct LDAPUser {
    pub username: String,
    pub name: String,
    pub when_created: String,
}

pub fn ldap_search(username: &str) -> Option<LDAPUser> {
    let settings = LdapConnSettings::new().set_no_tls_verify(true);
    let ldap = LdapConn::with_settings(settings, &CONFIG.bind_address)
        .expect("Unable to connect to LDAP");
    ldap.simple_bind(
        "cn=ucc-discord-bot,cn=Users,dc=ad,dc=ucc,dc=gu,dc=uwa,dc=edu,dc=au",
        &CONFIG.ldap_pass,
    )
    .expect("Unable to attempt to bind to LDAP")
    .success()
    .expect("Unable to bind to LDAP");
    let (rs, _res) = ldap
        .search(
            "cn=Users,dc=ad,dc=ucc,dc=gu,dc=uwa,dc=edu,dc=au",
            Scope::Subtree,
            &format!("(cn={})", username),
            vec!["when_created", "displayName", "name"],
        )
        .expect("LDAP error")
        .success()
        .expect("LDAP search error");
    if rs.len() != 1 {
        return None;
    }
    let result = SearchEntry::construct(rs[0].clone()).attrs;
    Some(LDAPUser {
        username: result
            .get("name")
            .expect("LDAP failed to get 'name' field")
            .join(""),
        name: result
            .get("displayName")
            .expect("LDAP failed to get 'displayName' field")
            .join(""),
        when_created: "".to_string() // result
            // .get("whenCreated")
            // .expect("LDAP failed to get 'whenCreated' field")
            // .join(""),
    })
}

pub fn ldap_exists(username: &str) -> bool {
    match ldap_search(username) {
        Some(_) => true,
        None => false,
    }
}

#[derive(Debug)]
pub struct TLA {
    pub tla: Option<String>,
    pub name: String,
    pub username: String,
}

pub fn tla_search(term: &str) -> Option<TLA> {
    let tla_search = String::from_utf8(
        std::process::Command::new("tla")
            .arg(term)
            .output()
            .expect("failed to execute tla")
            .stdout,
    )
    .expect("unable to parse stdout to String");
    let tla_results = tla_search.split("\n").collect::<Vec<&str>>();
    if tla_results.len() != 4 {
        return None;
    }
    let mut the_tla = Some(tla_results[0].replace("TLA: ", "")[1..4].to_string());
    if the_tla == Some(String::from("???")) {
        the_tla = None;
    }
    Some(TLA {
        tla: the_tla,
        name: tla_results[1].replace("Name: ", ""),
        username: tla_results[2].replace("Login: ", ""),
    })
}
