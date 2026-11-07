use chrono::{Local, LocalResult, NaiveDateTime, TimeZone};
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read epoch from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        return Err("Epoch time not provided on stdin".into());
    }

    // Parse epoch time in format YYYY MM DD HH mm SS
    let epoch_naive = NaiveDateTime::parse_from_str(input, "%Y %m %d %H %M %S")?;

    // Convert to local time, accounting for daylight saving time transitions
    let epoch_local = match Local.from_local_datetime(&epoch_naive) {
        LocalResult::Single(dt) => dt,
        LocalResult::Ambiguous(dt1, _) => dt1,
        LocalResult::None => {
            return Err("Invalid local time for epoch (possibly during a DST gap)".into())
        }
    };

    // Get current system time
    let current_local = Local::now();

    // Calculate elapsed seconds between current time and epoch
    let elapsed = current_local.timestamp() - epoch_local.timestamp();

    // Round down to the beginning of the 60-second interval
    let interval = (elapsed / 60) * 60;

    // Compute MD5(MD5(time elapsed))
    let hash1 = md5::compute(interval.to_string().as_bytes());
    let hash1_str = format!("{:x}", hash1);
    let hash2 = md5::compute(hash1_str.as_bytes());
    let hash2_str = format!("{:x}", hash2);

    // Extract first two letters ([a-f]) from left-to-right
    let mut letters = String::new();
    for c in hash2_str.chars() {
        if c >= 'a' && c <= 'f' {
            letters.push(c);
            if letters.len() == 2 {
                break;
            }
        }
    }

    // Extract first two single-digit integers ([0-9]) from right-to-left
    let mut digits = String::new();
    for c in hash2_str.chars().rev() {
        if c >= '0' && c <= '9' {
            digits.push(c);
            if digits.len() == 2 {
                break;
            }
        }
    }

    // Output the final 4-character code to stdout
    let code = format!("{}{}", letters, digits);
    println!("{}", code);

    Ok(())
}
