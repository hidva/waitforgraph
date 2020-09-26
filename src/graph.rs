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
use crate::get_or_default;
use crate::intern::*;
use crate::lock::*;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct Vertex {
    edges_in: HashSet<EdgeId>,
    edges_out: HashSet<EdgeId>,
}

type EdgeId = *const Edge;

// waiter -> holder
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct Edge {
    waiter: SessionId,
    holder: SessionId,
    wait: LockMode,
    hold: LockMode,
    obj: LockObjId,
}

#[derive(Default)]
pub struct WFGraph {
    data: LockInfo,
    edges: Internment<Edge>,
    sess_vert: HashMap<SessionId, Vertex>,
}

impl WFGraph {
    fn get_edgeid(&mut self, edge: Edge) -> EdgeId {
        self.edges.intern(edge)
    }

    fn get_vert(&mut self, sessid: SessionId) -> &mut Vertex {
        get_or_default(&mut self.sess_vert, sessid)
    }

    fn add_dependency(
        &mut self,
        waiter: SessionId,
        holder: SessionId,
        obj: LockObjId,
        wait: LockMode,
        hold: LockMode,
    ) {
        let edgeid = self.get_edgeid(Edge {
            waiter,
            holder,
            wait,
            hold,
            obj,
        });
        self.get_vert(waiter).edges_out.insert(edgeid);
        self.get_vert(holder).edges_in.insert(edgeid);
    }

    fn process_waiter(&mut self, waiter: SessionId, locks: &std::vec::Vec<Lock>, info: &LockInfo) {
        for &lock in locks {
            for &conflict_mode in lock.mode.conflict_modes() {
                if let Some(holders) = info.get_holders(conflict_mode, lock.objid) {
                    for &holder in holders {
                        if waiter == holder {
                            continue;
                        }
                        self.add_dependency(waiter, holder, lock.objid, lock.mode, conflict_mode);
                    }
                }
            }
        }
    }

    pub fn new(info: LockInfo) -> WFGraph {
        let mut ret = WFGraph::default();
        for (&waiter, locks) in info.waiter.iter() {
            ret.process_waiter(waiter, locks, &info);
        }
        ret.data = info;
        ret
    }

    fn get_obj(&self, objid: LockObjId) -> &LockObj {
        self.data.get_obj(objid)
    }

    fn desc_edge(&self, edge: &Edge) -> String {
        format!(
            "session {} waits for {} on {}; blocked by session {}(granted {});",
            edge.waiter,
            edge.wait,
            self.get_obj(edge.obj),
            edge.holder,
            edge.hold
        )
    }
}

pub mod dot;
