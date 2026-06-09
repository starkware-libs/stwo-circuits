//! `circuit-gui` — exports the menu circuits to `viewer/data.js`, which the
//! static browser viewer (`viewer/index.html`) loads to render an interactive,
//! hierarchical, read-only view of each circuit.
//!
//! Usage:
//!   cargo run                # writes ./viewer/data.js
//!   cargo run -- <out_dir>   # writes <out_dir>/data.js

mod catalog;
mod const_registry;
mod export;
mod menu;
mod model;

use std::path::PathBuf;

use serde::Serialize;

#[derive(Serialize)]
struct Entry {
    name: String,
    graph: model::Graph,
}

fn main() {
    let out_dir = std::env::args().nth(1).unwrap_or_else(|| "viewer".to_string());
    let out_path = PathBuf::from(&out_dir).join("data.js");

    let entries: Vec<Entry> = menu::builders()
        .into_iter()
        .map(|(name, build)| {
            // Recording is thread-local: clear, build (push/pop populate it),
            // then drain this circuit's spans before the next builder runs.
            circuits::scopes::reset();
            let ctx = build();
            let spans = circuits::scopes::take_spans();
            let graph = export::export(&ctx, &spans);
            eprintln!(
                "  {name:<18} {:>6} gates  {:>4} groups  depth {}",
                graph.meta.n_gates, graph.meta.n_groups, graph.meta.max_depth
            );
            Entry { name, graph }
        })
        .collect();

    let json = serde_json::to_string(&entries).expect("serialize circuits");
    let contents = format!("window.CIRCUITS = {json};\n");
    std::fs::write(&out_path, contents)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", out_path.display()));

    println!("Wrote {} circuits to {}", entries.len(), out_path.display());

    // Print a clickable link to the viewer. The `file://` URL works directly
    // (data.js loads via a <script> tag, no server needed).
    let index = PathBuf::from(&out_dir).join("index.html");
    match std::fs::canonicalize(&index) {
        Ok(abs) => {
            println!("\nOpen in a browser:\n  file://{}", abs.display());
            println!(
                "\nOr serve over http:\n  python3 -m http.server -d {} 8731\n  then open http://localhost:8731/",
                std::fs::canonicalize(&out_dir)
                    .unwrap_or_else(|_| PathBuf::from(&out_dir))
                    .display(),
            );
        }
        Err(_) => println!("Open {} in a browser.", index.display()),
    }
}
