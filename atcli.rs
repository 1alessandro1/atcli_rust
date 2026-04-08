use std::env;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::process;

// These are the exact terminators extracted from the original Compal firmware.
// The original code iterated through these to know when the modem finished replying.
const TERMINATORS: &[&str] = &[
    "+CME ERROR:",
    "+CMS ERROR:",
    "BUSY\r\n",
    "ERROR\r\n",
    "NO ANSWER\r\n",
    "NO CARRIER\r\n",
    "NO DIALTONE\r\n",
    "OK\r\n",
];

// Helper function to print POSIX-compliant usage instructions.
fn print_usage(program_name: &str) {
    println!("Usage for {}: ", program_name);
    println!("-p, --path: smd port");
    println!("-h, --help: usage help");
    println!("e.g. {} 'at cmd' (default /dev/smd11)", program_name);
    println!("     {} -p <smd port> 'at cmd'", program_name);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program_name = match args.get(0) {
        Some(name) => name,
        None => "atcmd",
    };

    // OPTIMIZATION: We avoid heavy external CLI parsing crates (like `clap`) 
    // to keep the compiled binary size as small as possible for embedded devices.
    let (device_path, at_command) = match args.len() {
        2 => {
            let arg = &args[1];
            if arg == "-h" || arg == "--help" || arg == "help" {
                print_usage(program_name);
                process::exit(0);
            }
            // Default behavior matching the original OEM utility
            ("/dev/smd11", arg.as_str())
        }
        4 => {
            // Dynamic path injection replacing the hardcoded OEM behavior
            if args[1] == "-p" || args[1] == "--path" || args[1] == "path" {
                (args[2].as_str(), args[3].as_str())
            } else {
                print_usage(program_name);
                process::exit(1);
            }
        }
        _ => {
            print_usage(program_name);
            process::exit(1);
        }
    };

    // Open the serial device. Linux TTY drivers natively handle the RTS/CTS 
    // flow control upon opening, bypassing the need for manual ioctl hacks 
    // seen in the original `atcmd` binary.
    let file_result = OpenOptions::new()
        .read(true)
        .write(true)
        .open(device_path);

    let mut file = match file_result {
        Ok(f) => f,
        Err(e) => {
            eprintln!("fopen({}) failed: {}", device_path, e);
            process::exit(1);
        }
    };

    // Send the command directly as bytes. 
    // We avoid using the `format!` macro to prevent string allocation overhead 
    // and keep the binary slim.
    if let Err(e) = file.write_all(at_command.as_bytes()) {
        eprintln!("failed to send '{}' to modem (res = {})", at_command, e);
        process::exit(1);
    }
    
    // Append the required CRLF. AT commands require this to be processed.
    if let Err(e) = file.write_all(b"\r\n") {
        eprintln!("failed to send CRLF to modem: {}", e);
        process::exit(1);
    }
    let _ = file.flush();

    // SAFETY & FIX: The original Compal `atcli` used a 4096-byte global buffer (`byte_2410`)
    // and `stpcpy`, causing buffer overflows on large responses. 
    // The original `atcmd` used `read` + `strstr`, causing serial fragmentation bugs.
    // 
    // We fix both by using a streaming `BufReader` that reads exactly up to the `\n` byte.
    // This prevents memory bloat (O(1) memory usage) and guarantees string completeness.
    let mut reader = BufReader::new(file);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                // EOF encountered. This means the device was disconnected or closed.
                eprintln!("EOF from modem");
                break;
            }
            Ok(_) => {
                // Instantly print the output to stdout. 
                // The original binary delayed this until the entire payload was buffered.
                print!("{}", line);

                // Check if the current line matches any known terminator.
                let mut break_loop = false;
                for &terminator in TERMINATORS {
                    if line.starts_with(terminator) {
                        break_loop = true;
                        break;
                    }
                }

                if break_loop {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading from modem: {}", e);
                break;
            }
        }
    }
}
