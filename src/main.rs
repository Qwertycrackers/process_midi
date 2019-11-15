/*! MIDI-to-frequency-file Processor

  This is a utility for transforming MIDI files into a binary format suitable for use on
  a microcontroller with a simple audio output scheme -- IE: the STM32 DAC
*/
use clap::{App, Arg};

fn app() -> App<'static, 'static> {
    App::new("process-midi")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Joseph A. Gerardot")
        .about("Transforms a MIDI file into a binary array of note events (basically compressed midi) evenly over time.")
        .arg(Arg::with_name("input")
            .help("The input midi file.")
            .value_name("FILE")
            .required(true))
        .arg(Arg::with_name("output")
            .help("The output file name.")
            .short("o")
            .long("output")
            .value_name("OUTPUT"))
}

fn main() {
    let matches = app().get_matches();
    let input_name = matches
        .value_of_os("input")
        .expect("Input file is required.");
    // Use the supplied output name, otherwise replace the file extension of the input name.
    /*let output_name = matches
        .value_of_os("ouput")
        .map(|name| name.to_os_string())
        .unwrap_or_else(|| replace_file_ext(input_name, "c".as_ref()));
    */
    // Parse and boil down the midi file
    let tones = parse_midi(&input_name);
}

use std::ffi::{OsStr, OsString};
fn replace_file_ext(name: &OsStr, extension: &OsStr) -> OsString {
    unimplemented!()
}
use std::path::Path;
fn parse_midi<P: AsRef<Path>>(name: P) -> Tones {
    let stripped_name = name.as_ref().to_string_lossy().split('.').next().unwrap().into();
    let mut tones = Tones::with_name(stripped_name);
    let mut reader = Reader::new(&mut tones, name.as_ref()).unwrap();
    reader.read().unwrap();
    tones
}
/// Representation of a song as series of tone settings, like a stripped-out MIDI.
struct Tones {
    notes: Vec<Note>,
    name: String,
}

impl Tones {
    pub fn with_name(name: String) -> Self {
        Self {
            notes: Vec::with_capacity(1000),
            name,
        }
    }
}

use ghakuf::{messages::*, reader::*};
impl Handler for Tones {
    fn header(&mut self, format: u16, track: u16, time_base: u16) {
        println!(
            "MIDI Header Dump --
            Format Number: {}
            Number of Tracks: {}
            Raw Timebase: {:X}
            Time Style: {}
            Ticks Per QNote (garbage if framerate) : {}
            ---",
            format,
            track,
            time_base,
            if (time_base >> 15) != 0 { "negative SMPTE" } else { "ticks per quarter note" },
            time_base & !(1 << 15)
        )
    }

    fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
        let _ = (delta_time, event, data);
        // Ignore meta events
    }

    fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {
        let _ = (delta_time, event);
    }

    fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {
        let _ = (delta_time, event, data);
        // Ignore SysEx events
    }

    fn track_change(&mut self) {} // Don't care about track changes

    fn status(&mut self) -> HandlerStatus {
        HandlerStatus::Continue // Always continue
    }
}
/// Single segment of continued notes. Says how long it runs before a change in notes is detected.
struct Note {
    /// Note field of MIDI spec. Remember first bit is direction.
    notes: u8,
    /// How long from the last event?
    delta_time: u32,
}
