use std::io::{self, Read};

fn try_decode(bin: &str, bit_size: usize) -> Option<String> {
    let mut result = String::new();

    for chunk in bin.as_bytes().chunks(bit_size) {
        if chunk.len() < bit_size {
            return None;
        }
        if let Ok(chunk_str) = std::str::from_utf8(chunk) {
            if let Ok(num) = u8::from_str_radix(chunk_str, 2) {
                // If checking 7-bit mode, all numbers must be valid 7-bit ASCII (<= 127)
                if bit_size == 7 && num > 127 {
                    return None;
                }
                
                let character = num as char;
                if character == '\x08' {
                    result.pop();
                } else {
                    result.push(character);
                }
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    Some(result)
}

fn decode(bin: &str) {
    let len = bin.len();

    // 1. Try 7-bit first if length is divisible by 7
    if len % 7 == 0 {
        if let Some(decoded) = try_decode(bin, 7) {
            println!("{}", decoded);
            return;
        }
    }

    // 2. Try 8-bit if length is divisible by 8
    if len % 8 == 0 {
        if let Some(decoded) = try_decode(bin, 8) {
            println!("{}", decoded);
            return;
        }
    }

    // 3. Fallback to 8-bit if neither exact match succeeded
    if let Some(decoded) = try_decode(bin, 8) {
        println!("{}", decoded);
    }
}

fn main() -> io::Result<()> {
    let mut bin_msg = String::new();
    io::stdin().read_to_string(&mut bin_msg)?;

    let clean_msg = bin_msg.replace("\x1b", "").replace("\r", "").replace("\n", "");
    decode(clean_msg.trim());

    Ok(())
}
