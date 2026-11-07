/**
 * Name Author: Khai Tran Nguyen, Landon Carderara, Marquise Douglas
 * Team 8 : Deimos (Marquise, Josh, Landon, Noah)
 * Date: Oct 9 2026
 * Details: Program 6 XOR program
 */

 use std::fs; //read the file
 use std::io::{self, Read, Write}; // io handle stdin and stdout
 use std::process; // give you exit, if the program have error

 // constant
 const KEY_FILE:  &str = "key";
 const BUF_SIZE: usize = 4096; // process data 4KB at a time

 // read the key file
 fn main(){
    // growable array of byte, if it work store k byte at key
    let key: Vec<u8> = match fs::read(KEY_FILE){
        Ok(k) => k,
        Err (e) => {
            eprintln!("Error: can not read key file '{}':{}", KEY_FILE, e);
            process::exit(1);
        }
    };
    // be sure that key is not empty
    if key.is_empty(){
        eprintln!("Error: key file '{}' is empty",KEY_FILE);
        process::exit(1);
    }

    // set up input, output, buffer
    let stdin = io::stdin();
    let stdout = io::stdout();
    // clock stdin,std out once for whole program, not on every read and write
    let mut input = stdin.lock();
    let mut output = stdout.lock();

    let mut buf = [0u8; BUF_SIZE]; //array of 4096 bytes, all set to 0
    let mut key_pos: usize =0; // position of key

    loop{ 
        let n = match input.read(&mut buf){
            Ok(0) => break, // don't have any
            Ok(n) => n,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue, // harmless interrupted , just continue
            Err(e) => {
                eprintln!("Error reading stdin: {}",e);
                process::exit(1); 
            }
        };

        // over the first n bytes
        for byte in buf[..n].iter_mut(){
            // take the value of this refference point XOR with key position
            // if the key is shorter than the message, we repeat the key
            *byte ^= key[key_pos];
            key_pos += 1;
            if key_pos == key.len(){
                key_pos =0;
            }
        }

        //write all n bytes into stdout
        // if it fail, this function handle the error
        if let Err(e) = output.write_all(&buf[..n]){
            eprintln!("Error writting stdout: {}", e);
            process::exit(1);
        }
    }
    
    //flush force everything out, so the last byte not lost
    let result = output.flush();      // step 1: do the flush, save the result
    if let Err(e) = result {          // step 2: check the result
        eprintln!("Error flushing stdout: {}", e);
        process::exit(1);
    }
}
