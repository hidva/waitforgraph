use anyhow::anyhow;
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};
use waitforgraph::graph::dot;
use waitforgraph::lock::SessionId;

fn add_graph(graph: &mut HashMap<SessionId, Vec<SessionId>>, line: &str) -> anyhow::Result<()> {
    let mut spliter = line.split("->");
    let left = spliter.next().ok_or(anyhow!(""))?.trim();
    let right = spliter.next().ok_or(anyhow!(""))?.trim();
    if spliter.next().is_some() {
        return Err(anyhow!(""));
    }
    let left: SessionId = left.parse()?;
    let right: SessionId = right.parse()?;

    if let Some(vexs) = graph.get_mut(&left) {
        vexs.push(right);
    } else {
        graph.insert(left, vec![right]);
    }
    return Ok(());
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let from: SessionId = argv[1].parse().expect("invalid from session id");

    let mut graph: HashMap<SessionId, Vec<SessionId>> = HashMap::new();
    for line in io::stdin().lock().lines() {
        match line {
            Ok(line) => {
                if add_graph(&mut graph, &line).is_err() {
                    eprintln!("add_graph: invalid line: {}", line);
                }
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }

    let mut meet = HashSet::new();
    let mut queue = Vec::new();
    queue.push(from);
    meet.insert(from);
    while let Some(sessid) = queue.pop() {
        if let Some(deps) = graph.get(&sessid) {
            for &dep in deps {
                if !meet.contains(&dep) {
                    queue.push(dep);
                    meet.insert(dep);
                }
            }
        }
    }

    println!("{}", dot::render_tiny(&graph, meet.iter().map(|v| *v)));
    return;
}
