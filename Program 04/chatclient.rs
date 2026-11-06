/*
 * author: josh ramachandran (v1), Team 8 (v2)
 * date: 10/28/26
 * desc: Rust program that extracts a covert message from the delays between
 *       characters of a message transmitted by a chat server, then decrypts
 *       it with the Vigenere key recovered in the first stage.
 *
 *       v1 is kept as chatclient_v1_josh.rs. This version adds the team-name
 *       handshake the challenge server requires, learns the two delay values
 *       from the data instead of assuming a 65 ms threshold, keeps the
 *       server's prompt out of the timing data, and decrypts the result.
 *
 * Build:  rustc --edition 2024 -O chatclient.rs
 * Run:    ./chatclient
 *         ./chatclient --debug
 *         ./chatclient --host 10.7.7.105 --port 12321 --team "Deimos"
 *         ./chatclient --key VIRGO
 */

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

// Defaults so the program runs with no arguments at all.
const HOST: &str = "10.7.7.105";
const PORT: u16 = 12321;
const TEAM: &str = "Team 8";
// The covert message is Vigenere ciphertext too. It names its own key: an
// anagram of "VIGOR", which is VIRGO. ("Andromeda" was the first stage's key,
// for the ciphertext that gave us this server's address.)
const KEY: &str = "VIRGO";

const SENTINEL: &str = "EOF";
// Generous ceiling on one character's pause: far above any real delay, but
// low enough that a dead connection does not hang the program forever.
const READ_TIMEOUT: Duration = Duration::from_secs(30);
// How long to wait for the server's opening prompt before giving up on it.
const PROMPT_TIMEOUT: Duration = Duration::from_millis(1500);
/*
 * A prompt is written in one go, so its bytes arrive back to back -- well
 * under a millisecond apart. Every character of the timed overt message is
 * separated by a deliberate pause, which is far larger. Ten milliseconds sits
 * comfortably between the two and tells them apart.
 */
const BURST_GAP: Duration = Duration::from_millis(10);

struct Config {
    host: String,
    port: u16,
    team: String,
    key: String,
    debug: bool,
}

fn parse_args() -> Config {
    let mut config = Config {
        host: HOST.to_string(),
        port: PORT,
        team: TEAM.to_string(),
        key: KEY.to_string(),
        debug: false,
    };

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--debug" | "-d" => config.debug = true,
            "--host" if i + 1 < args.len() => {
                config.host = args[i + 1].clone();
                i += 1;
            }
            "--port" if i + 1 < args.len() => {
                config.port = args[i + 1].parse().unwrap_or(PORT);
                i += 1;
            }
            "--team" if i + 1 < args.len() => {
                config.team = args[i + 1].clone();
                i += 1;
            }
            "--key" if i + 1 < args.len() => {
                config.key = args[i + 1].clone();
                i += 1;
            }
            other => eprintln!("ignoring unknown argument: {other}"),
        }
        i += 1;
    }
    config
}

// A read that returned nothing because the timeout expired, rather than
// because anything went wrong.
fn timed_out(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    )
}

/*
 * What the opening read found.
 *
 * Servers differ: the challenge server greets you and waits for a team name,
 * while the lab server starts transmitting the moment you connect. Reading
 * the greeting in one bulk call would swallow the first characters of a
 * silent server's message along with the pauses that carry their bits, so
 * the greeting is read one byte at a time and the two cases are told apart
 * by how fast the bytes arrive.
 */
struct Greeting {
    // Bytes read before the overt message began.
    bytes: Vec<u8>,
    // Set when a pause revealed that `bytes` was never a prompt at all: the
    // server is already transmitting, and this is the next character along
    // with the pause before it.
    transmitting: Option<(u8, Duration)>,
}

fn read_greeting(stream: &mut TcpStream) -> io::Result<Greeting> {
    let mut bytes = Vec::new();
    let mut byte = [0u8; 1];

    loop {
        let start = Instant::now();
        match stream.read(&mut byte) {
            // The server closed on us.
            Ok(0) => break,
            Ok(_) => {
                let elapsed = start.elapsed();
                // The wait for the very first byte includes connection
                // latency, so it says nothing about the sender's pacing.
                if !bytes.is_empty() && elapsed > BURST_GAP {
                    return Ok(Greeting {
                        bytes,
                        transmitting: Some((byte[0], elapsed)),
                    });
                }
                bytes.push(byte[0]);
            }
            // Silence means the server has said its piece and is waiting.
            Err(ref e) if timed_out(e) => break,
            Err(e) => return Err(e),
        }
    }

    Ok(Greeting {
        bytes,
        transmitting: None,
    })
}

/*
 * Receive the overt message, timing the pause before each character.
 *
 * The gaps are offset by one from the characters on purpose. The sender
 * writes a character and then sleeps for that character's covert bit, so the
 * pause measured between character i and character i+1 carries bit i. There
 * is no measurable pause before the first character, so gaps always holds
 * exactly one fewer entry than the message has characters.
 */
fn receive(
    stream: &mut TcpStream,
    mut overt: String,
    mut gaps: Vec<Duration>,
) -> io::Result<(String, Vec<Duration>)> {
    let mut byte = [0u8; 1];

    loop {
        if overt.ends_with(SENTINEL) {
            break;
        }

        let start = Instant::now();
        let n = match stream.read(&mut byte) {
            Ok(n) => n,
            Err(ref e) if timed_out(e) => {
                eprintln!("\n[timed out waiting for the next character]");
                break;
            }
            Err(e) => return Err(e),
        };
        let elapsed = start.elapsed();

        // Ok(0) is the real end of stream: the server closed the socket.
        if n == 0 {
            break;
        }

        // Show the overt message as it arrives rather than after the fact.
        print!("{}", byte[0] as char);
        io::stdout().flush().ok();

        if !overt.is_empty() {
            gaps.push(elapsed);
        }
        overt.push(byte[0] as char);
    }

    Ok((overt, gaps))
}

/*
 * Split the gaps into a short cluster and a long cluster, returning both
 * averages and the boundary between them.
 *
 * The lab suggests a fixed midpoint, but that only works when the server's
 * two delay constants are known in advance. Repeatedly re-averaging around
 * the current midpoint finds them from the data instead, so the client works
 * against any server and adapts to whatever latency the network adds.
 */
fn find_threshold(gaps: &[Duration]) -> (f64, f64, f64) {
    let ms: Vec<f64> = gaps.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
    let mut low = ms.iter().cloned().fold(f64::INFINITY, f64::min);
    let mut high = ms.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    for _ in 0..50 {
        let midpoint = (low + high) / 2.0;
        let below: Vec<f64> = ms.iter().cloned().filter(|v| *v <= midpoint).collect();
        let above: Vec<f64> = ms.iter().cloned().filter(|v| *v > midpoint).collect();
        if below.is_empty() || above.is_empty() {
            break;
        }
        let new_low = below.iter().sum::<f64>() / below.len() as f64;
        let new_high = above.iter().sum::<f64>() / above.len() as f64;
        if new_low == low && new_high == high {
            break;
        }
        low = new_low;
        high = new_high;
    }

    (low, high, (low + high) / 2.0)
}

// Convert a run of '0'/'1' characters into text, eight bits per character.
// A byte that is not printable ASCII becomes '?', as the assignment allows,
// so one mistimed pause does not destroy the rest of the message.
fn bits_to_string(bits: &str) -> String {
    let mut result = String::new();
    let mut i = 0;
    while i + 8 <= bits.len() {
        match u8::from_str_radix(&bits[i..i + 8], 2) {
            Ok(n) if n == b'\n' || n == b'\t' || (0x20..0x7f).contains(&n) => {
                result.push(n as char)
            }
            _ => result.push('?'),
        }
        i += 8;
    }
    result
}

/*
 * Vigenere decryption.
 *
 * Only letters advance the key; spaces, digits and punctuation pass through
 * untouched and do not consume a key character, which is the convention the
 * earlier stages of the challenge used.
 */
fn vigenere_decrypt(text: &str, key: &str) -> String {
    let shifts: Vec<u8> = key
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_lowercase() as u8 - b'a')
        .collect();
    if shifts.is_empty() {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    for ch in text.chars() {
        if ch.is_ascii_alphabetic() {
            let base = if ch.is_ascii_uppercase() { b'A' } else { b'a' };
            let offset = (ch as u8 - base + 26 - shifts[i % shifts.len()]) % 26;
            out.push((base + offset) as char);
            i += 1;
        } else {
            out.push(ch);
        }
    }
    out
}

// Print the covert message as recovered and as decrypted, so the raw bits
// stay visible if the key ever turns out to be wrong.
fn report(covert: &str, key: &str) {
    println!("\nCovert message (as received): {covert}");
    println!("Covert message (Vigenere key \"{key}\"): {}", vigenere_decrypt(covert, key));
}

fn main() {
    let config = parse_args();
    let address = format!("{}:{}", config.host, config.port);

    println!("Connecting to {address} ...");
    let mut stream = match TcpStream::connect(&address) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error connecting to {address}: {e}");
            std::process::exit(1);
        }
    };

    // Each character must travel in its own packet; batching would smear the
    // pauses together and destroy the covert signal.
    let _ = stream.set_nodelay(true);

    let _ = stream.set_read_timeout(Some(PROMPT_TIMEOUT));
    let greeting = match read_greeting(&mut stream) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Error reading from server: {e}");
            std::process::exit(1);
        }
    };

    // Seed the message with anything the greeting turned out to already be
    // part of the transmission.
    let (overt, gaps) = match greeting.transmitting {
        Some((next, gap)) => {
            let mut text = String::from_utf8_lossy(&greeting.bytes).into_owned();
            text.push(next as char);
            print!("{text}");
            io::stdout().flush().ok();
            (text, vec![gap])
        }
        None => {
            print!("{}", String::from_utf8_lossy(&greeting.bytes));
            io::stdout().flush().ok();
            (String::new(), Vec::new())
        }
    };

    // The challenge server expects a team name before it will transmit. A
    // server that never asked for one simply ignores this.
    if let Err(e) = stream.write_all(format!("{}\n", config.team).as_bytes()) {
        eprintln!("Error sending team name: {e}");
        std::process::exit(1);
    }
    if greeting.transmitting.is_none() {
        println!("\n[sent team name: {}]\n", config.team);
    }

    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    let gaps = match receive(&mut stream, overt, gaps) {
        Ok((_overt, gaps)) => gaps,
        Err(e) => {
            eprintln!("\nError reading from server: {e}");
            std::process::exit(1);
        }
    };
    println!("\n\n[disconnected -- {} delays measured]", gaps.len());

    if gaps.len() < 8 {
        eprintln!("Too few delays to recover a covert message.");
        std::process::exit(1);
    }

    let (low, high, threshold) = find_threshold(&gaps);

    if config.debug {
        println!("\nMeasured delays (ms):");
        for (i, gap) in gaps.iter().enumerate() {
            println!("{:5}  {:9.3}", i, gap.as_secs_f64() * 1000.0);
        }
    }
    println!("\nShort delay ~{low:.3} ms, long delay ~{high:.3} ms, threshold {threshold:.3} ms");
    if high - low < 2.0 {
        println!("WARNING: the two delays are nearly identical -- run again.");
    }

    // Which pause stands for 0 is a convention the server never states, so
    // decode both ways and keep whichever one produces the covert sentinel.
    let short_is_zero: String = gaps
        .iter()
        .map(|d| {
            if d.as_secs_f64() * 1000.0 >= threshold {
                '1'
            } else {
                '0'
            }
        })
        .collect();
    let short_is_one: String = short_is_zero
        .chars()
        .map(|c| if c == '0' { '1' } else { '0' })
        .collect();

    if config.debug {
        println!("\nBits (short delay = 0): {short_is_zero}");
    }

    for (label, bits) in [
        ("short delay = 0", &short_is_zero),
        ("short delay = 1", &short_is_one),
    ] {
        let decoded = bits_to_string(bits);
        match decoded.find(SENTINEL) {
            Some(at) => {
                report(&decoded[..at], &config.key);
                return;
            }
            None if config.debug => println!("\n[{label}, no sentinel] {decoded}"),
            None => {}
        }
    }

    // No sentinel either way, so show both readings and let the user judge.
    println!("\nNo \"{SENTINEL}\" sentinel found. Both readings:");
    for (label, bits) in [
        ("short delay = 0", &short_is_zero),
        ("short delay = 1", &short_is_one),
    ] {
        let decoded = bits_to_string(bits);
        println!("  {label}: {decoded}");
        println!("    decrypted: {}", vigenere_decrypt(&decoded, &config.key));
    }
    println!("\nRe-run to collect cleaner timings.");
}
