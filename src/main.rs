extern crate midir;

use enigo::{Enigo, Key, KeyboardControllable};
use std::io::stdin;
use std::error::Error;

use midir::{MidiInput, Ignore, MidiInputPort};

use std::{env, fs};
use csv::{Error as CsvError, StringRecord};

const MIDI_INPUT_NAME: &str = "kitara-midi-input";

// number of frets in each string: 22 + open string
const NUM_FRETS: usize = 23;
// number of strings: 6
const NUM_STRINGS: usize = 6;
// standard guitar tuning expressed in midi notes: E A D G B E
const TUNING_NOTES_HIGH_TO_LOW: &'static [i32] = &[64, 59, 55, 50, 45, 40];

// modifier keys
const SHIFT: &str = "SH";
const CTRL: &str = "CT";
const CMD: &str = "CM";
const ALT: &str = "AL";

// whitespace + other keys
const SPACE: &str = "SP";
const TAB: &str = "TA";
const BACKSPACE: &str = "BA";
const ENTER: &str = "EN";
const ESCAPE: &str = "ES";
const ARROW_LEFT: &str = "LE";
const ARROW_UP: &str = "UP";
const ARROW_RIGHT: &str = "RI";
const ARROW_DOWN: &str = "DO";

// midi status
const STATUS_PRESS: u8 = 9;
const STATUS_RELEASE: u8 = 8;

#[derive(Debug)]
struct Mapping {
    midi_channels: Vec<u8>,
    keymap: Vec<String>,
}

fn main() {
    // parse args
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 3, "Usage: kitara <device-name> <path/to/config/csv>");
    let device_name = &args[1];
    let csv_config_filepath = &args[2];

    // load and eval csv config file
    let csv = read_file_as_string(csv_config_filepath);
    match load_fretboard_mapping(csv) {
        Ok(m) => listen(m, device_name)
            .expect("Failed to listen to midi device"),
        Err(e) => println!("Failed to load config - {}", e),
    };
}

fn read_file_as_string(filepath: &str) -> String {
    return fs::read_to_string(filepath)
        .expect(format!("Failed to read file with path {}", filepath).as_ref());
}

fn load_fretboard_mapping(csv: String) -> Result<Mapping, CsvError> {
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    let rows: Vec<StringRecord> = reader.records()
        .map(Result::unwrap)
        .collect();

    let mut fretboard: Vec<String> = vec!["".into(); NUM_FRETS * NUM_STRINGS];
    for i in 0..NUM_STRINGS {
        for j in 0..NUM_FRETS {
            fretboard[i * NUM_FRETS + j] = rows[i][j + 1].into();
        }
    }

    return Result::Ok(Mapping {
        midi_channels: rows
            .iter()
            .map(|row| row[0].parse::<u8>().unwrap())
            .collect(),
        keymap: fretboard,
    });
}

fn listen(_mapping: Mapping, _midi_device_name: &str) -> Result<(), Box<dyn Error>> {
    print_keyboard_mapping(&_mapping);

    let mut midi_in = MidiInput::new(MIDI_INPUT_NAME)?;
    midi_in.ignore(Ignore::None);

    // filter out all midi in ports that match
    // the specified device name
    let matching_ports = midi_in.ports()
        .into_iter()
        .filter(|p|
            midi_in
                .port_name(p)
                .unwrap()
                .to_lowercase()
                .contains(&_midi_device_name.to_lowercase())
        ).collect::<Vec<MidiInputPort>>();

    // select the first port from list of matching ports
    let in_port = match matching_ports.len() {
        0 => return Err(format!("No input port found matching {}", _midi_device_name).into()),
        _ => &matching_ports[0]
    };

    // get device name before it is moved below
    let full_device_name = midi_in.port_name(in_port)?;

    let _conn_in = midi_in.connect(in_port, MIDI_INPUT_NAME, move |_, message, _| {
        if message.len() > 1 {
            // MIDI channel is encoded in the lower four bits
            // of the first byte of the message
            // one is added because midi channels are zero-based
            let channel = (message[0] & 0x0F) + 1u8;

            // MIDI Status is encoded in the higher four bits
            // of the first byte of the message
            let status = message[0] >> 4;

            // MIDI Note is encoded on the second byte of the message
            let note = i32::from(message[1]);

            // Let's only deal with Status 8 and 9 for our purpose
            if status == STATUS_PRESS || status == STATUS_RELEASE {
                // execute typing only if the message's MIDI channel
                // matches one of the channels from the mapping struct
                match _mapping.midi_channels.iter().position(|&x| x == channel) {
                    Some(gtr_string) => handle_robo_typing(
                        &_mapping,
                        channel,
                        status,
                        gtr_string,
                        note),
                    None => println!("Failed mapping channel {}", channel),
                }
            }
        }
    }, ())?;
    println!("Successfully connected to MIDI Device: {}", full_device_name);

    stdin().read_line(&mut String::new())?;
    Ok(())
}

fn handle_robo_typing(_mapping: &Mapping, channel: u8, status: u8, gtr_string: usize, note: i32) {
    // guitar fret is derived by subtracting the tuning note for
    // the string played from the current midi note played
    let gtr_fret = note - TUNING_NOTES_HIGH_TO_LOW[gtr_string];
    // because the entire fretboard is encoded into a 1-dimensional vector
    // the right position for the string/fret needs to be calculated
    let keymap_position = gtr_string * NUM_FRETS + (gtr_fret as usize);
    // the key represents the keyboard key that will be invoked in this command
    let key = &_mapping.keymap[keymap_position][..];
    match key {
        // modifier keys
        SHIFT => press_release_key(status, Key::Shift),
        CTRL => press_release_key(status, Key::Control),
        ALT => press_release_key(status, Key::Alt),
        CMD => press_release_key(status, Key::Meta),
        // whitespace
        SPACE => click_key(status, Key::Space),
        TAB => click_key(status, Key::Tab),
        BACKSPACE => click_key(status, Key::Backspace),
        ENTER => click_key(status, Key::Return),
        // control keys
        ESCAPE => click_key(status, Key::Escape),
        ARROW_LEFT => click_key(status, Key::LeftArrow),
        ARROW_UP => click_key(status, Key::UpArrow),
        ARROW_RIGHT => click_key(status, Key::RightArrow),
        ARROW_DOWN => click_key(status, Key::DownArrow),
        // all other characters
        _ => {
            if !key.is_empty() {
                let ch = key.chars().next().unwrap();
                click_key(status, Key::Layout(ch));
            }
        }
    }

    println!(
        "string={}, fret={}, channel={}, note={}, key={}, action={}",
        gtr_string,
        gtr_fret,
        channel,
        note,
        match key.len() {
            0 => "<unmapped>",
            _ => key
        },
        match status {
            STATUS_PRESS => "press",
            STATUS_RELEASE => "release",
            _ => "???"
        }
    );
}

fn print_keyboard_mapping(_mapping: &Mapping) {
    //print header
    println!("\nKeyboard Mapping:");
    for _j in 0..NUM_FRETS {
        print!("{}\t", _j)
    }
    println!();

    // print a line below the header
    for _j in 0..NUM_FRETS {
        print!("----");
    }
    println!();

    //print mapping for each string
    for i in 0..NUM_STRINGS {
        print!("{}|", &_mapping.midi_channels[i]);
        for j in 0..NUM_FRETS {
            print!("{}\t", &_mapping.keymap[i * NUM_FRETS + j]);
        }
        println!();
    }
    println!();
}

fn press_release_key(status: u8, key: Key) {
    let mut enigo = Enigo::new();
    if status == STATUS_PRESS {
        enigo.key_down(key);
    } else {
        enigo.key_up(key);
    }
}

fn click_key(status: u8, key: Key) {
    let mut enigo = Enigo::new();
    if status == STATUS_PRESS {
        enigo.key_click(key);
    }
}
