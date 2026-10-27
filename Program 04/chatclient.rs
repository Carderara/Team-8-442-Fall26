/*
 * author: josh ramachandran
 * date: 10/26/26
 * desc: Rust language program that can extract a covert message from the delays in between characters of a message transmitted by a chat server.
 */



use std::env;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Instant;

fn main() -> io::Result<()> {
    // Default server configuration (IP: 138.47.99.29, Port: 31337)
    let args: Vec<String> = env::args().collect();
    let server_ip = if args.len() > 1 { &args[1] } else { "138.47.99.29" };
    let server_port = if args.len() > 2 { &args[2] } else { "31337" };
    let address = format!("{}:{}", server_ip, server_port);

    let mut stream = TcpStream::connect(&address)?;
    
    let mut buffer = [0u8; 1];
    let mut overt_bytes = Vec::new();
    let mut delays = Vec::new();
    
    let mut last_time: Option<Instant> = None;

    // Read character by character to accurately measure inter-character delays
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, // Connection closed by server
            Ok(_) => {
                let now = Instant::now();
                let byte = buffer[0];

                if let Some(prev) = last_time {
                    let duration = now.duration_since(prev);
                    let delay_ms = duration.as_secs_f64() * 1000.0;
                    delays.push(delay_ms);
                }
                last_time = Some(now);

                // Print the overt message character to stdout as it arrives
                print!("{}", byte as char);
                io::stdout().flush()?;

                overt_bytes.push(byte);

                // Check if the overt message ends with "EOF"
                if overt_bytes.ends_with(b"EOF") {
                    break;
                }
            }
            Err(e) => {
                eprintln!("\nError reading from stream: {}", e);
                break;
            }
        }
    }

    println!("\n[Disconnecting from the chat server]");

    // Print measured delays with 3 decimal places to inspect clusters
    println!("\n--- Raw Inter-Character Delays ---");
    for (i, &d) in delays.iter().enumerate() {
        print!("{:.3}ms ", d);
        if (i + 1) % 10 == 0 { 
            println!(); 
        }
    }
    println!("\n----------------------------------");

    if !delays.is_empty() {
        
        let threshold = 65.0; 

        let mut covert_bits = Vec::new();
        for &d in &delays {
            // Map delays to '0' or '1' based on the static threshold
            let bit = if d > threshold { '1' } else { '0' };
            covert_bits.push(bit);
        }

        // Convert bits (8 bits per character) into ASCII bytes
        let mut covert_message_bytes = Vec::new();
        for chunk in covert_bits.chunks(8) {
            if chunk.len() == 8 {
                let s: String = chunk.iter().collect();
                if let Ok(byte_val) = u8::from_str_radix(&s, 2) {
                    covert_message_bytes.push(byte_val);
                }
            }
        }
        
        let covert_string = String::from_utf8_lossy(&covert_message_bytes);
        
        // Parse up to "EOF" for the covert message[cite: 2]
        let clean_message = if let Some(idx) = covert_string.find("EOF") {
            &covert_string[..idx]
        } else {
            &covert_string
        };

        println!("\nCovert message: {}", clean_message.trim());
    }

    Ok(())
}
chatclient.rs