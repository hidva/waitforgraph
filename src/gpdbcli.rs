/*
Copyright 2020 <盏一 w@hidva.com>
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/
use postgres::{Client, Config, NoTls, SimpleQueryMessage};

#[derive(Debug)]
pub struct GPDBVersion {
    verstr: String,
    pub semver: semver::Version,
}

impl GPDBVersion {
    // str: (Greenplum Database 6.3.0 build dev)
    fn new(gpver: &str) -> GPDBVersion {
        let marker = "(Greenplum Database ";
        let errstr = format!("UnexpectedVersion={}", gpver);
        let ver_start = gpver.find(marker).expect(errstr.as_str()) + marker.len();
        let ver_end = gpver.find(')').expect(errstr.as_str());
        let verstr = gpver[ver_start..ver_end].to_string();
        let semverstr = if let Some(ver_end) = verstr.find(' ') {
            &verstr[..ver_end]
        } else {
            verstr.as_str()
        };
        // semverstr may be 4.3.99.00
        let semverpart: Vec<&str> = semverstr.split('.').collect();
        let semverstr = semverpart[..3].join(".");
        let semver = semver::Version::parse(semverstr.as_str()).expect(errstr.as_str());
        GPDBVersion { verstr, semver }
    }
}

fn autofill_opt(
    cfg: &mut Config,
    exists: fn(&Config) -> bool,
    set: fn(&mut Config, &str),
    optenv: &str,
    optdef: &str,
) {
    if exists(cfg) {
        return;
    }

    let val: String;
    if let Ok(envval) = std::env::var(optenv) {
        val = envval;
    } else {
        val = String::from(optdef);
    }

    set(cfg, val.as_str());
}

fn get_config(connstr: &str) -> Config {
    let mut cfg: Config = connstr.parse().unwrap();
    autofill_opt(
        &mut cfg,
        |c| c.get_hosts().len() > 0,
        |c, v| {
            c.host(v);
        },
        "PGHOST",
        "127.0.0.1",
    );
    autofill_opt(
        &mut cfg,
        |c| c.get_ports().len() > 0,
        |c, v| {
            c.port(v.parse().unwrap());
        },
        "PGPORT",
        "5432",
    );
    autofill_opt(
        &mut cfg,
        |c| c.get_user().is_some(),
        |c, v| {
            c.user(v);
        },
        "PGUSER",
        std::env::var("USER").unwrap().as_str(),
    );
    let user = String::from(cfg.get_user().unwrap());
    autofill_opt(
        &mut cfg,
        |c| c.get_dbname().is_some(),
        |c, v| {
            c.dbname(v);
        },
        "PGDATABASE",
        user.as_str(),
    );
    autofill_opt(
        &mut cfg,
        |c| c.get_application_name().is_some(),
        |c, v| {
            c.application_name(v);
        },
        "PGAPPNAME",
        "hidva/wait-for-graph",
    );
    cfg
}

pub struct GPDBCli {
    cli: Client,
    pub ver: GPDBVersion,
}

impl GPDBCli {
    fn query_val(cli: &mut Client, query: &str) -> String {
        let ret = &cli.simple_query(query).unwrap()[0];
        if let SimpleQueryMessage::Row(row) = ret {
            String::from(row.get(0).unwrap())
        } else {
            panic!("UnexpectedResult; query={} ret=CommandComplete", query)
        }
    }

    pub fn new(connstr: &str) -> GPDBCli {
        let cfg = get_config(connstr);
        let mut cli = cfg.connect(NoTls).unwrap();
        let ver =
            GPDBVersion::new(GPDBCli::query_val(&mut cli, "select pg_catalog.version()").as_str());
        GPDBCli { cli, ver }
    }

    pub fn query(&mut self, query: &str) -> Vec<SimpleQueryMessage> {
        self.cli.simple_query(query).unwrap()
    }
}
