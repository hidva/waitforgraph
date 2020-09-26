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
use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::Hash;

// implement an Iterator to iterate Internment.0. for now, just make Internment.0 public.
pub struct Internment<T>(pub HashSet<Box<T>>);

impl<T> Default for Internment<T> {
    fn default() -> Self {
        Self(HashSet::new())
    }
}

impl<T> Internment<T>
where
    T: Eq + Hash,
{
    // How to define a newtype Id for *const T? like: type Id = *const T;
    pub fn intern(&mut self, t: T) -> *const T {
        if let Some(v) = self.0.get(&t) {
            v.borrow() as *const T
        } else {
            let b = Box::new(t);
            let ret = b.borrow() as *const T;
            self.0.insert(b);
            ret
        }
    }
}
