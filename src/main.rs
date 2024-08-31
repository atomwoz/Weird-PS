use std::io::Write;
use std::{fs, process, thread, time::Duration}; // Import the Write trait for file writing

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

extern crate ctrlc;

const POLL_RATE_MS: u64 = 100;

fn bytes_to_human(bytes: u64) -> String {
    let bytes = bytes as f64;
    let kb = bytes / 1024.;
    let mb = kb / 1024.;
    let gb = mb / 1024.;
    let tb = gb / 1024.;
    if tb > 0.01 {
        format!("{:.2}TB", tb)
    } else if gb > 0.01 {
        format!("{:.2}GB", gb)
    } else if mb > 0.01 {
        format!("{:.2}MB", mb)
    } else if kb > 0.01 {
        format!("{:.2}KB", kb)
    } else {
        format!("{:.2}B", bytes)
    }
}

fn processes_task(dir_to_watch: String) {
  let dir_to_watch = dir_to_watch + "\\C_C\\";
  let dir_to_watch = &dir_to_watch;
    info!("Waiting for start file in {}", dir_to_watch);
    loop {
        let dir_handle = fs::read_dir(dir_to_watch);
        let has_start_file = match dir_handle {
            Ok(mut x) => x.any(|entry| {
                if let Ok(entry) = entry {
                    entry.file_name() == "#PROCKI"
                } else {
                    false
                }
            }),
            Err(err) => {
                error!("Error reading directory {}: {}", dir_to_watch, err);
                false
            }
        };

        if has_start_file {
            break;
        }

        thread::sleep(Duration::from_millis(POLL_RATE_MS));
    }

    info!("Listing processes");

    let mut sysinfo = sysinfo::System::new();
    sysinfo.refresh_processes(sysinfo::ProcessesToUpdate::All);
    sysinfo.processes().iter().for_each(|x| {
        let x = x.1;
        let egze = if let Some(x) = x.exe() {
            x.to_string_lossy().to_string()
        } else {
            "<no exe known>".to_string()
        };
        let strr = format!(
            "Executable: {}\n CPU Usage: {}%\n RAM Usage: {}\n Disk Usage (readed): {}\n Disk Usage (writed): {}\nStatus: {}\n =====================\n Environment: {:?}\n",
            egze,
            ((x.cpu_usage() * 10000.).round()) / 100.,
            bytes_to_human(x.memory()),
            bytes_to_human(x.disk_usage().total_read_bytes),
            bytes_to_human(x.disk_usage().total_written_bytes),
            x.status(),
            x.environ()
            
        );
        fs::write(
            dir_to_watch.to_owned() + &x.name().to_string_lossy() + ".txt",
            strr,
        )
        .expect("Cannot write info file");
    });
}

///*
/// Create folder C_C on desktop
/// Create file #PROCKI in C_C
/// Program will fill C:\...\C_C\ with files with info about processes
///  */
fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    ctrlc::set_handler(|| {
        info!("Recived ctrl+c, exiting...");
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler!");

    let user_home_desktop = dirs::desktop_dir().expect("Cannot get desktop directory").to_string_lossy().to_string();

    let handle = thread::spawn(|| processes_task(user_home_desktop));
    handle.join().expect("Cannot join thread");
}
