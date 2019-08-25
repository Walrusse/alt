extern crate toml;
extern crate dialoguer;

use std::env;
use std::process;
use crate::def_file;
use crate::shim;
use crate::scan;
use crate::scan::CommandVersion;

fn prompt_versions(versions: &[CommandVersion]) -> Vec<usize> {
    let items: Vec<_> = versions.iter()
        .map(|version| format!("{} {} ({})",
            version.command, version.version, version.path.to_str().unwrap()
        ))
        .collect();

    let items_refs: Vec<_> = items.iter().map(String::as_ref).collect();

    println!("Here are the versions I found.");
    println!("  ↑/↓,j/k: move cursor");
    println!("  <space>: toggle keep");
    println!("  <enter>: confirm");
    println!();

    dialoguer::Checkboxes::new()
        .items(items_refs.as_slice())
        .clear(false)
        .interact()
        .unwrap()
}

pub fn run(command: &str) {
    let scans = vec![
        scan::path_suffix::scan(command),
        scan::homebrew::scan(command),
    ];
    let is_empty = scans.iter()
        .all(|v| v.is_empty());
    let versions: Vec<_> = scans.into_iter().flat_map(|x| x).collect();

    if is_empty {
        println!("Sorry, could not find any versions of {}", command);
        process::exit(1);
    } else {
        let choices = prompt_versions(&versions);

        if choices.is_empty() {
            println!("Looks like you didn't choose anything.");
            println!("Did you forget to select versions with <space>?");
        } else {
            let mut defs = def_file::load();
            {
                let def = defs.entry(command.to_string())
                    .or_insert_with(def_file::CommandVersions::new);

                for choice in choices {
                    let version = &versions[choice];
                    def.insert(version.version.clone(), version.path.clone());
                }
            }
            def_file::save(&defs)
                .expect("failed to save defs file");

            shim::make_shim(command, env::current_exe().unwrap().as_path())
                .unwrap_or_else(|err| panic!(
                    "failed to create shim for {}: {}",
                    command, err
                ));
        }
    }
}
