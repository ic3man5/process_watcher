use std::{env, sync::mpsc};

use sysinfo::{SystemExt, ProcessExt, ProcessStatus,};
use getopts::{Options};
use tray_item::{TrayItem};

/// Gets a "help" String to print to the console.
pub fn print_usage_string(program_name: &str, opts: &Options) {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    let brief = format!("Usage: {} PROCESS_NAME [options]", program_name);
    println!("Version: {}\n{}", VERSION, opts.usage(&brief));
}

/// Returns true if any process is found matching name and is in a run state
fn process_exists(name: &str) -> bool {
    let mut system = sysinfo::System::new();
    system.refresh_processes();
    for p in system.processes_by_name(name) {
        return p.status() == ProcessStatus::Run;
    }
    false
}

enum Message {
    ProcessUpdate(bool),
    Quit,
}

enum ThreadMessage {
    Quit,
}

fn main() {
    // Parse arguments
    let args: Vec<String> = std::env::args().collect();
    let mut opts = Options::new();
    // Help argument
    opts.optflag("h", "help", "Displays this help menu");
    // Version argument
    opts.optflag("v", "version", "Displays the version");
    // Invert argument
    opts.optflag("i", "invert", "Inverts the icons");
    // Loop delay
    opts.optopt("d", "delay", "Delay in ms before refreshing status", "2500");
    // load the arguments
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!("{}", f),
    };
    // Lets start parsing all the options here
    // version
    if matches.opt_present("v") {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        println!("Version: {}", VERSION);
        return;
    }
    // help
    if matches.opt_present("h") {
        print_usage_string(&args[0], &opts);
        return;
    }
    // Get the process name
    let process_name = if !matches.free.is_empty() {
        matches.free.join(" ")
    } else {
        print_usage_string(&args[0], &opts);
        return;
    };
    // Get the optional arguments
    let invert_icons = matches.opt_present("i");
    let duration_ms: u64 = match matches.opt_str("d") {
            Some(v) => v.parse().unwrap(),
            None => 2500,
    };
    // create the tray item
    let tray_result = TrayItem::new(format!("Process Watcher ({process_name})").as_str(), "ok");
    match &tray_result {
        Ok(t) => t,
        Err(e) => {
            println!("Error: Failed to create Tray Item: {}", e);
            return;
        },
    };
    let mut tray = tray_result.unwrap();

    // Create the channels for the tray and thread
    let (tray_tx, tray_rx) = mpsc::channel();
    let thread_tray_tx = tray_tx.clone();
    let thread_process_name = process_name.clone();
    let (thread_tx, thread_rx) = mpsc::channel();
    let thread = std::thread::spawn(move || {
        loop {
            let process_alive = process_exists(thread_process_name.as_str());
            thread_tray_tx.send(Message::ProcessUpdate(process_alive)).unwrap();
            match thread_rx.recv_timeout(std::time::Duration::from_millis(duration_ms)) {
                 Ok(ThreadMessage::Quit) => break,
                _ => {},
            }
        };
    });
    // Add a tray menu item to be able to quit
    tray.add_menu_item("Quit", move || {
        println!("Quit");
        tray_tx.send(Message::Quit).unwrap();
    })
    .unwrap();

    // actually update the tray icon here
    let mut last_result = false;
    let ok_icon_name;
    let cancel_icon_name;
    if invert_icons {
        ok_icon_name = "cancel";
        cancel_icon_name = "ok";
    } else {
        ok_icon_name = "ok";
        cancel_icon_name = "cancel";
    }
    loop {
        match tray_rx.recv() {
            Ok(Message::Quit) => break,
            Ok(Message::ProcessUpdate(b)) => {
                if !last_result && b {
                    tray.set_icon(ok_icon_name).unwrap();
                } else if last_result && !b {
                    tray.set_icon(cancel_icon_name).unwrap();
                }
                last_result = b;
            },
            Err(_) => (),
        }
    }
    // tell the thread to quit and wait for it
    thread_tx.send(ThreadMessage::Quit).unwrap();
    thread.join().expect("Process thread failed to finish!");
}

/*
{
    let mut tray = TrayItem::new("Tray Example", "my-icon-name").unwrap();

    tray.add_label("Tray Label").unwrap();

    tray.add_menu_item("Hello", || {
        println!("Hello!");
    })
    .unwrap();

    let (tx, rx) = mpsc::channel();

    tray.add_menu_item("Quit", move || {
        println!("Quit");
        tx.send(Message::Quit).unwrap();
    })
    .unwrap();

    loop {
        match rx.recv() {
            Ok(Message::Quit) => break,
            _ => {}
        }
    }
}
*/