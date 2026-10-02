use std::env;
use std::io::{self, BufRead, Write};


fn main () {
    let args: Vec<String> =env::args().collect();
    // args[0] is the program name; args[1] is "-e"/"-d"; args[2] is the key
    if args.len() <3{
        println!("Usage: ./vigenere -e|-d <key>");
        std::process::exit(1);
    }
    let encrypt = args[1] == "-e"; // true = encrypt (-e), false = decrypt (-d)
    
    let raw_key = &args[2];
    let mut key: Vec<u8> = Vec:: new();

    for c in raw_key.chars(){
        if c.is_ascii_alphabetic(){
            // key uppercase doesn't matter , then put it into 0-25 range alphabet
            let lower = c.to_ascii_lowercase();
            let value = (lower as u8) - b'a';
            key.push(value);
        }
        // else if not the char, just skip
    }

    // read ONE line at a time so each result prints right after Enter (matches the demo)
    let stdin = io::stdin();
    let mut out = io::stdout();
    let mut k = 0; // outside the line loop: the key continues across lines

    for line in stdin.lock().lines(){
        let line = line.unwrap();
        for c in line.chars(){
            if c.is_ascii_alphabetic(){
                let key_char = key[k % key.len()];
                let result = shift_letter(c,key_char,encrypt);
                print!("{}",result);
                k += 1;
            }else{
                print!("{}",c);
            }
        }
        println!();           // re-add the newline .lines() stripped off
        out.flush().unwrap(); // show this line now, before reading the next
    }
}


fn shift_letter(letter:char, key: u8, encrypt: bool) ->char {
    // 1. remember if plain text was uppercase
    let uppercase = letter.is_ascii_uppercase();   

    // 2. base = b'A' or b'a' depending on case
    let base: u8;
    if uppercase {
        base = b'A';
    }
    else{
        base = b'a';
    }
    // 3. p = (c as u8) - base            // 0..=25
    let letter_convert = (letter as u8) - base;

    // 4. apply the lecture formula with k
    let result: u8;
    if encrypt{
        result = (letter_convert + key )%26;
    }else{
        result = (26 +letter_convert - key )%26;
    }
    
    // 5. return (base + result) as char

    return (result + base) as char;
}