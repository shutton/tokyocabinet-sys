extern crate pkg_config;
extern crate vergen;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use vergen::*;

#[cfg(target_os = "macos")]
const PKG_CONFIG_PATH: &str = "PKG_CONFIG_PATH";
#[cfg(target_os = "macos")]
const MACOS_TC_PC_PATH: &str = "/usr/local/Cellar/tokyo-cabinet/1.4.48/lib/pkgconfig";

fn main() {
    // This works on macOS if TC is installed via brew
    // Since TC is not longer maintained, the version is unlikely to change
    #[cfg(target_os = "macos")]
    match env::var(PKG_CONFIG_PATH) {
        Ok(v) => env::set_var(PKG_CONFIG_PATH, format!("{}:{}", v, MACOS_TC_PC_PATH)),
        Err(_) => env::set_var(PKG_CONFIG_PATH, MACOS_TC_PC_PATH),
    }

    let mut flags = OutputFns::all();
    flags.toggle(NOW);
    assert!(vergen(flags).is_ok());

    let latest = env::var("CARGO_FEATURE_LATEST").is_ok();
    if latest {
        build_tokyocabinet();
    } else {
        let has_pkgconfig = Command::new("pkg-config").output().is_ok();

        if has_pkgconfig && pkg_config::find_library("libtokyocabinet").is_ok() {
            return;
        } else {
            build_tokyocabinet();
        }
    }
}

fn build_tokyocabinet() {
    let manifest_dir = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(d) => d,
        None => panic!("Unable to read manifest dir"),
    };
    let out_dir = match env::var_os("OUT_DIR") {
        Some(d) => d,
        None => panic!("Unable to read output dir"),
    };
    let src = PathBuf::from(&manifest_dir).join("tokyocabinet");
    let dst = PathBuf::from(&out_dir).join("build");
    let _ = fs::create_dir(&dst);

    run(Command::new("./configure")
            .args(&["--enable-fastest", "--disable-bzip"]) // bzip2-sys has some trouble playing nicely.
            .env("CFLAGS", "-fPIC")
            .env("CPPFLAGS", "-fPIC")
            .current_dir(&src));
    run(Command::new("make").current_dir(&src));
    let _ = fs::copy(&src.join("libtokyocabinet.a"), &dst.join("libtokyocabinet.a"));

    println!("cargo:rustc-link-lib=static=tokyocabinet");
    println!("cargo:rustc-link-search=native=/usr/lib64");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-flags=-L {}", dst.display());
}

fn run(cmd: &mut Command) {
    assert!(cmd.stdout(Stdio::inherit())
               .stderr(Stdio::inherit())
               .status()
               .unwrap()
               .success());
}
