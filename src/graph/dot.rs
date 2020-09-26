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
use crate::graph::*;
use std::borrow::Borrow;

pub fn render(wfg: &WFGraph) -> String {
    let mut dot = vec![
        String::from("strict digraph G {"),
        String::from("label=\"WaitForGraph - Generated By hidva/waitforgraph\";"),
    ];

    for (&vert, _) in wfg.sess_vert.iter() {
        dot.push(format!("{};", vert));
    }

    for edgebox in wfg.edges.0.iter() {
        let edge: &Edge = Borrow::borrow(edgebox);
        dot.push(format!("{} -> {}", edge.waiter, edge.holder));
    }

    dot.push(String::from("}"));

    dot.push(String::from("/*大吉大利~"));
    for edgebox in wfg.edges.0.iter() {
        dot.push(wfg.desc_edge(edgebox));
    }
    dot.push(String::from("*/"));

    dot.join("\n")
}
