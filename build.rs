use std::collections::VecDeque;
use std::path::Path;
use std::process::Command;
use std::{env, fs};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=package-lock.json");
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=tailwind.css");
    let mut queue = VecDeque::from(
        fs::read_dir(env::current_dir().unwrap().join("templates/"))
            .unwrap()
            .map(|e| e.unwrap())
            .collect::<Vec<_>>(),
    );
    while let Some(e) = queue.pop_front() {
        if e.file_type().unwrap().is_file() {
            let name = e.file_name().into_string().unwrap();
            if name.ends_with(".html") {
                println!(
                    "cargo:rerun-if-changed={}",
                    e.file_name().into_string().unwrap()
                );
            }
        } else {
            println!(
                "cargo:rerun-if-changed={}",
                e.file_name().into_string().unwrap()
            );
            for e in fs::read_dir(e.path()).unwrap() {
                queue.push_back(e.unwrap());
            }
        }
    }

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("style.css");

    Command::new("npm")
        .args(["i"])
        .output()
        .expect("failed to compile tailwind");

    Command::new("npx")
        .args(["tailwindcss"])
        .args(["-m"])
        .args(["-c", "tailwind.config.js"])
        .args(["-i", "tailwind.css"])
        .args(["-o", dest_path.to_str().unwrap()])
        .output()
        .expect("failed to compile tailwind");
}
