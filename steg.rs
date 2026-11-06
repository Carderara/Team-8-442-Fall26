use std::env;
use std::fs;
use std::io::{self, Write};

// mark end of file
const SENTINEL: [u8; 6] = [0x00, 0xFF, 0x00, 0x00, 0xFF, 0x00];

fn main()
{
    if let Err(error) = run()
    {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn store_bits( wrapper: &mut Vec<u8>, hidden: &[u8], offset: usize, interval: usize)-> Result <(), String>
{
    let mut current_offset = offset;

    // checks if interval is zero
    if interval == 0
    {
       return Err("Interval cannot be zero".to_string());
    }

    // loops through hidden data and sentinel
    for byte in hidden.iter().chain(SENTINEL.iter()) 
    {
        

        // eight wrapper bytes for one hidden byte
        for j in 0..8
        {
            // checks if file is to big for wrapper
            if current_offset >= wrapper.len()
            {
                return Err("Hidden file is too large for wrapper".to_string());
            }

            wrapper[current_offset] &= 0xFE;

            // extracts one bit at a time
            let bit = (byte >> (7-j)) & 1;
            
            //stores bit in least significant bit
            wrapper[current_offset] |= bit;

            // increase offset
            current_offset += interval;
        }
    }

    Ok(())
}

fn retrieve_bits( wrapper: &[u8], offset: usize, interval: usize,) -> Result<Vec<u8>, String>
{
    // check if interval is -
    if interval == 0
    {
        return Err("Interval cannot be zero".to_string());
    }

    let mut current_offset = offset;
    let mut hidden = Vec::new();

    while current_offset < wrapper.len()
    {
        let mut byte: u8 = 0;

        for _j in 0..8
        {
            // check if offset is bigger than the wrapper
            if current_offset >= wrapper.len()
            {
                return Err("Incomplete hidden byte".to_string());
            }

            //shift bits left to make room for new bit
            byte <<= 1;

            // extract lsb and add it to byte
            byte |= wrapper[current_offset] & 1;

            current_offset += interval;
        }
        hidden.push(byte);

        // check if message is as long as SENTINEL
        if hidden.len() >= SENTINEL.len()
        {
            // get the last six bytes
            let start = hidden.len() - SENTINEL.len();

            // check if last six bytes are the SENTINEL
            if hidden[start..] == SENTINEL
            {
                // remove the SENTINEL
                hidden.truncate(start);
                return Ok(hidden);
            }
        }
    }

    Err("Sentinel not found".to_string())
}

fn run() -> Result<(), String> 
{
    let args: Vec<String> = env::args().collect();

    // mode variables
    let mut store = false;
    let mut retrieve = false;
    let mut bit_mode = false;
    let mut byte_mode = false;

    // number variables
    let mut offset: usize = 0;
    let mut interval: usize = 1;

    // file variables
    let mut wrapper_file = String::new();
    let mut hidden_file = String::new();

    let mut i = 1;

    while i < args.len()
    {
        match args[i].as_str()
        {
            "-s" => store = true,
            "-r" => retrieve = true,
            "-b" => bit_mode = true,
            "-B" => byte_mode = true,

            "-o" => 
            {
                i += 1;
                if i >= args.len() 
                {
                    return Err("Missing offset value".to_string());
                }
                offset = args[i].parse().map_err(|_| "Invalid offset".to_string())?;
            }

            "-i" => 
            {
                i+= 1;
                if i >= args.len()
                {
                    return Err("Missing interval value".to_string());
                }
                interval = args[i].parse().map_err(|_| "Invalid interval".to_string())?;
            }

             "-w" =>
            {
                i += 1;
                if i >= args.len() 
                {
                    return Err("Missing wrapper filename".to_string());
                }
                wrapper_file = args[i].clone();
            }

            "-h" => 
            {
                i += 1;
                if i >= args.len() 
                {
                    return Err("Missing hidden filename".to_string());
                }
                hidden_file = args[i].clone();
            }

            _ => return Err("Unknown argument".to_string()),
        }

        i += 1;
    }

    // validation checks
    if store == retrieve
    {
        return Err("Choose either -s or -r".to_string());
    }

    if bit_mode == byte_mode
    {
        return Err("Choose either -b or -B".to_string());
    }
    
    if wrapper_file.is_empty() 
    {
        return Err("Wrapper file is required".to_string());
    }

    if store && hidden_file.is_empty() 
    {
        return Err("Hidden file is required for storing".to_string());
    }

    if interval == 0 
    {
        return Err("Interval cannot be zero".to_string());
    }

    let mut wrapper = fs::read(&wrapper_file).map_err(|_| "Could not read wrapper file".to_string())?;

    if store {
        let hidden = fs::read(&hidden_file).map_err(|_| "Could not read hidden file".to_string())?;

        if bit_mode     
        {
            store_bits(&mut wrapper, &hidden, offset, interval)?;
        } 
        
        else 
        {
            return Err("Byte storage is not implemented yet".to_string());   
        }

        io::stdout().write_all(&wrapper).map_err(|_| "Could not write output".to_string())?;

    } 
    
    else 
    {
        let hidden = if bit_mode 
        {
            retrieve_bits(&wrapper, offset, interval)?
        } 
        
        else 
        {
            return Err("Byte retrieval is note implemented yet". to_string());
        };

        io::stdout().write_all(&hidden).map_err(|_| "Could not write output".to_string())?;
    }
    Ok(())
}

