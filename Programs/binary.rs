use std::io::{self, Read}; 

fn decode(bin: &str, bit_size: usize){
    let mut decoded_text = String::new();

    for chunk in bin.as_bytes().chunks(bit_size){
        if let Ok(chunk_str) = std::str::from_utf8(chunk){
            if let Ok(num) = u8::from_str_radix(chunk_str, 2){
                let ch = num as char;

                if ch == '\x08' {
                    decoded_text.pop();
                }
                else {
                    decoded_text.push(ch);
                }
            }
        }
    }
    print!("{}\n", decoded_text);
}

fn main() -> io::Result<()>{

    // takes in an input
    let mut bin_mes = String::new();
    io::stdin().read_to_string(&mut bin_mes)?;

    // gets rid of any escape characters
    let clean_bin_mes: String= bin_mes
        .chars()
        .filter(|c| *c == '0' || *c == '1')
        .collect();
    print!("-----7 bit---------\n");
    decode(&clean_bin_mes, 7);
    print!("-----8 bit---------\n");
    decode(&clean_bin_mes, 8);    

    Ok(())
}