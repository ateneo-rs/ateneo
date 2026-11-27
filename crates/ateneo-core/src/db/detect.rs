// SPDX-FileCopyrightText: 2026 James Paul Vizuete Davila
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::time::Duration;

use crate::config::DEFAULT_DATABASE_PORT;

use super::is_listening;

pub const ENV_PGHOST: &str = "PGHOST";
pub const ENV_PGPORT: &str = "PGPORT";
pub const ENV_PGDATABASE: &str = "PGDATABASE";
pub const ENV_PGUSER: &str = "PGUSER";

pub const LOCAL_HOST: &str = "localhost";
pub const DOCKER_HOSTS: &[&str] = &["postgres", "db"];
pub const SUGGESTED_NAME: &str = "ateneo";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    pub host: String,
    pub port: u16,
}

impl Address {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
        }
    }
}

fn env_port(env: &impl Fn(&str) -> Option<String>) -> Option<u16> {
    env(ENV_PGPORT)
        .and_then(|value| value.trim().parse().ok())
        .filter(|port| *port != 0)
}

fn env_host(env: &impl Fn(&str) -> Option<String>) -> Option<String> {
    env(ENV_PGHOST)
        .map(|host| host.trim().to_string())
        .filter(|host| !host.is_empty() && !host.starts_with('/') && !host.contains(','))
}

fn env_name(env: &impl Fn(&str) -> Option<String>, var: &str) -> String {
    env(var)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| SUGGESTED_NAME.to_string())
}

pub fn candidates(env: impl Fn(&str) -> Option<String>, docker: bool) -> Vec<Address> {
    let port = env_port(&env).unwrap_or(DEFAULT_DATABASE_PORT);
    let mut list = Vec::new();
    if let Some(host) = env_host(&env) {
        list.push(Address::new(&host, port));
    }
    if docker {
        for host in DOCKER_HOSTS {
            list.push(Address::new(host, DEFAULT_DATABASE_PORT));
        }
    }
    list.push(Address::new(LOCAL_HOST, port));
    list.push(Address::new(LOCAL_HOST, DEFAULT_DATABASE_PORT));
    let mut unique: Vec<Address> = Vec::new();
    for address in list {
        if !unique.contains(&address) {
            unique.push(address);
        }
    }
    unique
}

pub fn detect(candidates: &[Address], timeout: Duration) -> Option<Address> {
    candidates
        .iter()
        .find(|address| is_listening(&address.host, address.port, timeout))
        .cloned()
}

pub fn suggested_address(env: impl Fn(&str) -> Option<String>) -> Address {
    Address {
        host: env_host(&env).unwrap_or_else(|| LOCAL_HOST.to_string()),
        port: env_port(&env).unwrap_or(DEFAULT_DATABASE_PORT),
    }
}

pub fn suggested_database(env: impl Fn(&str) -> Option<String>) -> String {
    env_name(&env, ENV_PGDATABASE)
}

pub fn suggested_user(env: impl Fn(&str) -> Option<String>) -> String {
    env_name(&env, ENV_PGUSER)
}
