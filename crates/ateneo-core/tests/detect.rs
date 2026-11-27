// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashMap;
use std::time::Duration;

use ateneo_core::db::detect::{
    self, Address, DOCKER_HOSTS, ENV_PGDATABASE, ENV_PGHOST, ENV_PGPORT, ENV_PGUSER, LOCAL_HOST,
    SUGGESTED_NAME,
};

fn env(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let map: HashMap<String, String> = vars
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    move |name| map.get(name).cloned()
}

#[test]
fn without_hints_only_localhost_is_tried() {
    assert_eq!(
        detect::candidates(env(&[]), false),
        vec![Address::new(LOCAL_HOST, 5432)]
    );
}

#[test]
fn pghost_and_pgport_come_first() {
    let candidates = detect::candidates(
        env(&[(ENV_PGHOST, "db.ejemplo.edu"), (ENV_PGPORT, "6543")]),
        false,
    );

    assert_eq!(
        candidates,
        vec![
            Address::new("db.ejemplo.edu", 6543),
            Address::new(LOCAL_HOST, 6543),
            Address::new(LOCAL_HOST, 5432),
        ]
    );
}

#[test]
fn docker_adds_the_usual_service_names() {
    let candidates = detect::candidates(env(&[]), true);

    for host in DOCKER_HOSTS {
        assert!(candidates.contains(&Address::new(host, 5432)));
    }
    assert_eq!(candidates.last(), Some(&Address::new(LOCAL_HOST, 5432)));
}

#[test]
fn ignores_sockets_and_invalid_ports() {
    let candidates = detect::candidates(
        env(&[(ENV_PGHOST, "/var/run/postgresql"), (ENV_PGPORT, "cero")]),
        false,
    );

    assert_eq!(candidates, vec![Address::new(LOCAL_HOST, 5432)]);
}

#[test]
fn suggests_ateneo_or_the_pg_variables() {
    assert_eq!(detect::suggested_database(env(&[])), SUGGESTED_NAME);
    assert_eq!(detect::suggested_user(env(&[])), SUGGESTED_NAME);
    assert_eq!(
        detect::suggested_database(env(&[(ENV_PGDATABASE, "repositorio")])),
        "repositorio"
    );
    assert_eq!(
        detect::suggested_user(env(&[(ENV_PGUSER, "biblioteca")])),
        "biblioteca"
    );
    assert_eq!(
        detect::suggested_user(env(&[(ENV_PGUSER, "  ")])),
        SUGGESTED_NAME
    );
}

#[test]
fn suggests_localhost_or_the_pg_variables() {
    assert_eq!(
        detect::suggested_address(env(&[])),
        Address::new(LOCAL_HOST, 5432)
    );
    assert_eq!(
        detect::suggested_address(env(&[(ENV_PGHOST, "db"), (ENV_PGPORT, "5433")])),
        Address::new("db", 5433)
    );
}

#[test]
fn detect_returns_the_first_listening_address() {
    let url = std::env::var("DATABASE_URL").unwrap();
    let location = url.rsplit_once('@').unwrap().1;
    let (host, port) = location
        .split_once('/')
        .unwrap()
        .0
        .rsplit_once(':')
        .unwrap();
    let running = Address::new(host, port.parse().unwrap());

    let found = detect::detect(
        &[Address::new("127.0.0.1", 1), running.clone()],
        Duration::from_millis(500),
    );
    assert_eq!(found, Some(running));

    assert_eq!(
        detect::detect(&[Address::new("127.0.0.1", 1)], Duration::from_millis(300)),
        None
    );
}
