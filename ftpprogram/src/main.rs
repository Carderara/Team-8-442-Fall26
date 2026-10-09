/**
 * Name Author: Khai Tran Nguyen
 * Team8 : Deimos (Marquise, Josh, Landon, Noah)
 * Date: Oct 9 2026
 * Details: FTP Storage Convert Channel
 */

use std::process; // exit with error

use suppaftp::FtpStream; //ftp connection object
use suppaftp::Mode; //choose passive mode

const HOST: &str = "138.47.99.21";
const PORT: u16 = 21;
const USERNAME: &str = "anonymous";
const PASSWORD: &str = "nothing@example.com";
const DIRECTORY: &str = "10"; // 7 for 7 bits, 10 for 10 bits
const METHOD: Method = Method::TenBit;  

// Two supported decoding schema
enum Method{
    SevenBit,
    TenBit,
}

fn main(){
    // 1. connect server, fetch, listing, and disconnect imediately
    let listing = match fetch_listing(){
        Ok(lines) => lines,
        Err(e) =>{
            eprintln!("FTP error: {e}");
            process::exit(1);
        }
    };

    // 2. Decode the message after disconnecting
    let message = decode(&listing, METHOD);

    // 3. Print out the convert message
    println!("{message}");
}

//FTP part
fn fetch_listing() -> Result<Vec<String>,Box<dyn std::error::Error>>{
    let address = format!("{HOST}:{PORT}"); // create new string

    let mut ftp = FtpStream::connect(address)?; // the state will be change so mutable
    ftp.login(USERNAME,PASSWORD)?;
    ftp.set_mode(Mode::Passive);// only you connect to server, active in opposite
    ftp.cwd(DIRECTORY)?; // Change directory

    let listing = ftp.list(None)?;

    ftp.quit()?; // disconnect before decode

    Ok(listing)

}

struct Entry{
    name: String, // file name
    bits: [u8; 10], // need 8bits for 10 numbers is enough
}

fn parse_line(line: &str) -> Option<Entry>{ // might return struct Entry or nNone
    let fields: Vec<&str> = line.split_whitespace().collect(); // vac store multiple elemnet

    let permission = fields.first()?;
    if permission.chars().count() <10{
        return None;
    }
    if fields.len() < 9{
        return None;
    }
    let name = fields[8..].join(" "); //remove the space in file name
    if name == "." || name == ".."{
        return None;
    }

    let mut bits = [0u8;10];
    for (slot, char) in bits.iter_mut().zip(permission.chars()){ // pair positio with character
        *slot = if char == '-' {0} else {1};
    }

    Some(Entry {name,bits})
}

fn decode(listing: &[String], method: Method) -> String{
    let mut entries: Vec<Entry> = listing.iter().filter_map(|line| parse_line(line)).collect();

    entries.sort_by(|a,b| a.name.cmp(&b.name)); // comepare 2 name in entries

    match method{
        Method::SevenBit => decode_7bit(&entries),
        Method::TenBit => decode_10bit(&entries),
    }
}

// 2 decoder 

fn decode_7bit(entries: &[Entry]) -> String{
    let mut out = String::new();
    for entry in entries{
        // skip the noise, have bit in Type, owner-read, or owner-write 
        if entry.bits[0] == 1|| entry.bits[1] == 1|| entry.bits[2] == 1{
            continue;
        }
        // 7 right most bit = 1 char
        let value = bits_to_u8(&entry.bits[3..10]);
        out.push(value as char);
    }
    out
}

fn decode_10bit(entries: &[Entry]) -> String{
    let mut stream: Vec<u8> = Vec::new();
    for entry in entries {
        stream.extend_from_slice(&entry.bits);   // // all 10bits into 1 stream
    } 

    let mut out = String::new();
    for chunk in stream.chunks(7){
        if chunk.len() < 7 {
            break;                                // leftover padding, ignore
        }
        out.push(bits_to_u8(chunk) as char);
    }

    while out.ends_with('\0') {                   // strip trailing padding
        out.pop();
    }
    out

}

fn bits_to_u8 (bits: &[u8]) -> u8{
    bits.iter().fold(0u8, |acc, &b| (acc << 1) | (b & 1)) // 0 at first, then accumulated, shift left to multiple by 2 and then add b
}