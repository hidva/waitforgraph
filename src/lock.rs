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
use crate::gpdbcli::*;
use crate::intern::*;
use crate::{get_or_default, Void};
use postgres::{SimpleQueryMessage, SimpleQueryRow};
use std::collections::{HashMap, HashSet};
use std::{fmt, str};

#[derive(PartialEq, Eq, Hash, Default, Clone)]
pub struct LockObj {
    locktype: Option<String>,
    gp_segment_id: Option<i64>,
    virtualxid: Option<String>,
    database: Option<i64>,
    relation: Option<i64>,
    page: Option<i64>,
    tuple: Option<i64>,
    transactionid: Option<i64>,
    classid: Option<i64>,
    objid: Option<i64>,
    objsubid: Option<i64>,
}

impl fmt::Display for LockObj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut v: Vec<String> = vec![];
        macro_rules! desc {
            ($field:ident) => {
                if let Some(ref val) = self.$field {
                    v.push(format!("{}={}", stringify!($field), val));
                }
            };
        }

        desc!(locktype);
        desc!(gp_segment_id);
        desc!(virtualxid);
        desc!(database);
        desc!(relation);
        desc!(page);
        desc!(tuple);
        desc!(transactionid);
        desc!(classid);
        desc!(objid);
        desc!(objsubid);

        write!(f, "{}", v.join(","))
    }
}

// need documentation about the behavior of Eq/Hash on LockMode.
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum LockMode {
    AccessExclusiveLock,
    AccessShareLock,
    ExclusiveLock,
    RowExclusiveLock,
    RowShareLock,
    ShareLock,
    ShareRowExclusiveLock,
    ShareUpdateExclusiveLock,
}

impl From<&str> for LockMode {
    fn from(input: &str) -> Self {
        match input {
            "AccessExclusiveLock" => LockMode::AccessExclusiveLock,
            "AccessShareLock" => LockMode::AccessShareLock,
            "ExclusiveLock" => LockMode::ExclusiveLock,
            "RowExclusiveLock" => LockMode::RowExclusiveLock,
            "RowShareLock" => LockMode::RowShareLock,
            "ShareLock" => LockMode::ShareLock,
            "ShareRowExclusiveLock" => LockMode::ShareRowExclusiveLock,
            "ShareUpdateExclusiveLock" => LockMode::ShareUpdateExclusiveLock,
            _ => panic!("UnknownLockMode={}", input),
        }
    }
}

impl fmt::Display for LockMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LockMode::RowExclusiveLock => write!(f, "RowExclusiveLock"),
            LockMode::RowShareLock => write!(f, "RowShareLock"),
            LockMode::ShareLock => write!(f, "ShareLock"),
            LockMode::ShareRowExclusiveLock => write!(f, "ShareRowExclusiveLock"),
            LockMode::ExclusiveLock => write!(f, "ExclusiveLock"),
            LockMode::AccessExclusiveLock => write!(f, "AccessExclusiveLock"),
            LockMode::AccessShareLock => write!(f, "AccessShareLock"),
            LockMode::ShareUpdateExclusiveLock => write!(f, "ShareUpdateExclusiveLock"),
        }
    }
}

impl LockMode {
    pub fn conflict_modes(&self) -> &[Self] {
        match self {
            LockMode::RowExclusiveLock => [
                LockMode::ExclusiveLock,
                LockMode::AccessExclusiveLock,
                LockMode::ShareLock,
                LockMode::ShareRowExclusiveLock,
            ]
            .as_ref(),
            LockMode::RowShareLock => {
                [LockMode::ExclusiveLock, LockMode::AccessExclusiveLock].as_ref()
            }
            LockMode::ShareLock => [
                LockMode::ExclusiveLock,
                LockMode::RowExclusiveLock,
                LockMode::AccessExclusiveLock,
                LockMode::ShareRowExclusiveLock,
                LockMode::ShareUpdateExclusiveLock,
            ]
            .as_ref(),
            LockMode::ShareRowExclusiveLock => [
                LockMode::ExclusiveLock,
                LockMode::RowExclusiveLock,
                LockMode::ShareLock,
                LockMode::ShareUpdateExclusiveLock,
                LockMode::AccessExclusiveLock,
                LockMode::ShareRowExclusiveLock,
            ]
            .as_ref(),
            LockMode::ExclusiveLock => [
                LockMode::RowShareLock,
                LockMode::ExclusiveLock,
                LockMode::RowExclusiveLock,
                LockMode::ShareLock,
                LockMode::ShareUpdateExclusiveLock,
                LockMode::AccessExclusiveLock,
                LockMode::ShareRowExclusiveLock,
            ]
            .as_ref(),
            LockMode::AccessExclusiveLock => [
                LockMode::RowShareLock,
                LockMode::ExclusiveLock,
                LockMode::AccessShareLock,
                LockMode::RowExclusiveLock,
                LockMode::ShareRowExclusiveLock,
                LockMode::ShareLock,
                LockMode::AccessExclusiveLock,
                LockMode::ShareUpdateExclusiveLock,
            ]
            .as_ref(),
            LockMode::AccessShareLock => [LockMode::AccessExclusiveLock].as_ref(),
            LockMode::ShareUpdateExclusiveLock => [
                LockMode::ExclusiveLock,
                LockMode::AccessExclusiveLock,
                LockMode::ShareLock,
                LockMode::ShareRowExclusiveLock,
                LockMode::ShareUpdateExclusiveLock,
            ]
            .as_ref(),
        }
    }
}

impl str::FromStr for LockMode {
    type Err = Void;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(LockMode::from(s))
    }
}

pub type LockObjId = *const LockObj;
pub type SessionId = i64;

#[derive(Clone, Copy)]
pub struct Lock {
    pub objid: LockObjId,
    pub mode: LockMode,
}

#[derive(Default)]
pub struct LockInfo {
    objs: Internment<LockObj>,
    pub granted_table: HashMap<LockObjId, HashMap<LockMode, HashSet<SessionId>>>,
    pub waiter: HashMap<SessionId, Vec<Lock>>,
}

impl LockInfo {
    pub fn get_obj(&self, objid: LockObjId) -> &LockObj {
        unsafe { &*objid }
    }
    fn get_objid(&mut self, lockobj: LockObj) -> LockObjId {
        self.objs.intern(lockobj)
    }

    fn getlock(row: &SimpleQueryRow) -> (LockObj, LockMode, SessionId, bool) {
        let mut obj = LockObj::default();

        macro_rules! fill {
            ($field: ident) => {
                obj.$field = row.get(stringify!($field)).map(|v| v.parse().unwrap())
            };
        }

        fill!(locktype);
        fill!(gp_segment_id);
        fill!(virtualxid);
        fill!(database);
        fill!(relation);
        fill!(page);
        fill!(tuple);
        fill!(transactionid);
        fill!(classid);
        fill!(objid);
        fill!(objsubid);

        (
            obj,
            row.get("mode").unwrap().parse().unwrap(),
            row.get("mppsessionid").unwrap().parse().unwrap(),
            row.get("granted").unwrap().parse().unwrap(),
        )
    }

    fn add_waiter(&mut self, objid: LockObjId, mode: LockMode, sessid: SessionId) {
        get_or_default(&mut self.waiter, sessid).push(Lock { objid, mode });
    }

    fn add_granted(&mut self, objid: LockObjId, mode: LockMode, sessid: SessionId) {
        get_or_default(get_or_default(&mut self.granted_table, objid), mode).insert(sessid);
    }

    fn process_row(&mut self, row: &SimpleQueryRow) {
        let (lockobj, mode, sessid, granted) = LockInfo::getlock(row);
        let objid = self.get_objid(lockobj);
        if !granted {
            self.add_waiter(objid, mode, sessid);
        } else {
            self.add_granted(objid, mode, sessid);
        }
    }

    fn process(qres: Vec<SimpleQueryMessage>) -> LockInfo {
        let mut lockinfo = LockInfo::default();
        for rowres in qres.iter() {
            if let SimpleQueryMessage::Row(row) = rowres {
                lockinfo.process_row(row);
            }
        }
        lockinfo
    }

    pub fn get(cli: &mut GPDBCli) -> LockInfo {
        let qstr = if cli.ver.semver.major <= 4 {
            "select null as virtualxid,\
            gp_segment_id,locktype,database,relation,page,tuple,\
            transactionid,classid,objid,objsubid,mode,\
            case when granted = 't' then 'true' else 'false' end as granted,\
            mppsessionid from pg_locks"
        } else {
            "select virtualxid,\
            gp_segment_id,locktype,database,relation,page,tuple,\
            transactionid,classid,objid,objsubid,mode,\
            case when granted = 't' then 'true' else 'false' end as granted,\
            mppsessionid from pg_locks"
        };
        LockInfo::process(cli.query(qstr))
    }

    pub fn get_holders(&self, lockmode: LockMode, objid: LockObjId) -> Option<&HashSet<SessionId>> {
        self.granted_table
            .get(&objid)
            .and_then(|v| v.get(&lockmode))
    }
}
