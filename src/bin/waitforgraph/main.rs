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
use waitforgraph::gpdbcli::*;
use waitforgraph::graph::*;
use waitforgraph::lock::*;

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let mut cli = GPDBCli::new(if argv.len() >= 2 {
        argv[1].as_str()
    } else {
        ""
    });
    let wfg = WFGraph::new(LockInfo::get(&mut cli));
    println!("{}", dot::render(&wfg));
}
