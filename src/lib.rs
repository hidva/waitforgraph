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
use std::collections::HashMap;

// LockMode is public, so Void must also be pub...
#[derive(Debug)]
pub enum Void {}

fn get_or_default<K, V, S>(map: &mut HashMap<K, V, S>, k: K) -> &mut V
where
    K: std::hash::Hash + Eq + Copy,
    S: std::hash::BuildHasher,
    V: Default,
{
    get_or_default_ex(map, k, <V as Default>::default)
}

fn get_or_default_ex<K, V, S, F>(map: &mut HashMap<K, V, S>, k: K, new: F) -> &mut V
where
    K: std::hash::Hash + Eq + Copy,
    S: std::hash::BuildHasher,
    F: FnOnce() -> V,
{
    if let Some(v) = map.get_mut(&k) {
        let vp = v as *mut V;
        unsafe { &mut *vp }
    } else {
        map.insert(k, new());
        map.get_mut(&k).unwrap()
    }
}

mod intern;

pub mod lock;

pub mod gpdbcli;

pub mod graph;
