#[macro_use] extern crate log;

use std::fs;
use std::fs::{DirEntry, File};
use std::path::PathBuf;
use std::process::Command;
use std::io::{stdin, stdout, Read, Write};

use simplelog::*;


#[derive(PartialEq)]
enum GameRegion {
    Us,
    Eu,
    Jp,
    Unknown,
}

fn main() {
    if !init_fixer() {
        return;
    }

    let game_id = {
        if PathBuf::from("../NPUB30910").exists() {
            GameRegion::Us
        }
        else if PathBuf::from("../NPEB01202").exists() {
            GameRegion::Eu
        }
        else if PathBuf::from("../NPJA00102").exists() {
            GameRegion::Jp
        }
        else {
            GameRegion::Unknown
        }
    };

    if game_id != GameRegion::Unknown {
        let elapsed_time = std::time::Instant::now();
        let dec_path = match game_id {
            GameRegion::Us => "../NPUB30910/USRDIR",
            GameRegion::Eu => "../NPEB01202/USRDIR",
            GameRegion::Jp => "../NPJA00102/USRDIR",
            GameRegion::Unknown => "",
        };

        info!("Game found. Starting process.");
        info!("Copying game files. Please wait...");
        copy(dec_path.split_at(12).0);

        info!("Starting decryption. This can take a while....");
        decrypt(dec_path);

        println!("\n");
        info!("Game decrypted in {:?} seconds.", elapsed_time.elapsed().as_secs());
    }
    else {
        error!("No known PSN DeS game files detected");
        pause();
        return;
    }

    println!("\n");
    info!("Good to go.");
    info!("RPCS3 should pick it up after a game list refresh, or a restart");
    pause();
}

fn init_fixer() -> bool {
    if PathBuf::from("./log_fixer.log").exists() {
        fs::remove_file("./log_fixer.log").unwrap();
    }

    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed),
        WriteLogger::new(LevelFilter::Info, Config::default(), File::create("log_fixer.log").unwrap()),
    ]).unwrap();

    if !PathBuf::from("./resources/make_npdata.exe").exists() {
        error!("Can't find make_npdata.exe. Execution can't continue.");
        pause();
        false
    }
    else {
        true
    }
}

fn copy(path: &str) {
    let dir_entries = fs::read_dir(path).unwrap();

    fs::create_dir_all("../../disc/DeS-Converted/PS3_GAME").unwrap();

    for entry in dir_entries {
        if entry.is_ok() {
            let entry: DirEntry = entry.unwrap();
            let entry_path = entry.path();
            let split_path = entry_path.to_str().unwrap().split_at(12);
            let target_path = format!("../../disc/DeS-Converted/PS3_GAME{}", split_path.1);

            if entry_path.is_dir() {
                if entry_path.file_name().unwrap() == "LICDIR" || entry_path.file_name().unwrap() == "MANUAL" {
                    continue;
                }

                fs::create_dir_all(target_path).unwrap();
                copy(entry_path.to_str().unwrap());
            }
            else {
                if entry_path.file_name().unwrap() == "ISO2PKG.DAT" {
                    continue;
                }

                fs::copy(entry_path, target_path).unwrap();
            }
        }
    }
}

fn decrypt(path: &str) {
    let entries = fs::read_dir(path).unwrap();
    for entry in entries {
        if entry.is_ok() {
            let entry: DirEntry = entry.unwrap();
            if entry.path().is_dir() {
                decrypt(entry.path().to_str().unwrap());
            }
            else {
                if entry.path().file_name().unwrap() == "EBOOT.BIN" {
                    continue;
                }
                decrypt_file(entry.path());
            }
        }
    }
}

fn decrypt_file(path: PathBuf) {
    let split_path = path.to_str().unwrap().split_at(12);
    let target_path = format!("../../disc/DeS-Converted/PS3_GAME{}", split_path.1);
    
    info!("Decrypting file {:?} to target '{}'...", path.to_str().unwrap(), target_path);

    let mut process = Command::new("./resources/make_npdata.exe").args(&["-d", path.to_str().unwrap(), target_path.as_str(), "0"])
        .stdout(std::process::Stdio::null())
        .spawn().expect("Failed to launch process");
    
    process.wait().unwrap();
}

fn pause() {
    stdout().write(b"Press Enter to continue...").unwrap();
    stdout().flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}