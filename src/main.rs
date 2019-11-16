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
use std::fs;
fn main() -> Result<(), io::Error> {
    let matches = app().get_matches();
    let input_name = matches
        .value_of_os("input")
        .expect("Input file is required.");
    // Use the supplied output name, otherwise replace the file extension of the input name.
    let output_name = matches
        .value_of_os("output")
        .map(|name| name.to_os_string())
        .unwrap_or_else(|| replace_file_ext(input_name, ".c".as_ref()));
    let code_name = strip_name(&output_name); // The name used for this thing in C source code.
    // Parse and boil down the midi file
    let tones = parse_midi(&input_name);
    let mut output_file = fs::File::create(output_name)?;
    tones.write_c_src(&code_name, &mut output_file)?;
    Ok(())
}

use std::ffi::{OsStr, OsString};
fn replace_file_ext(name: &OsStr, extension: &OsStr) -> OsString {
    let mut stripped_name: OsString = name.to_os_string().to_string_lossy().split('.').next().unwrap().into();
    stripped_name.push(extension);
    stripped_name
}

fn strip_name(name: &OsStr) -> String {
    name.to_os_string().to_string_lossy().split('.').next().unwrap().into()
}

use std::path::Path;
fn parse_midi<P: AsRef<Path>>(name: P) -> Tones {
    let mut tones = Tones::new();
    let mut reader = Reader::new(&mut tones, name.as_ref()).unwrap();
    reader.read().unwrap();
    tones
}
/// Representation of a song as series of tone settings, like a stripped-out MIDI.
use std::io;
struct Tones {
    notes: Vec<Note>,
}

impl Tones {
    pub fn new() -> Self {
        Self {
            notes: Vec::with_capacity(1000),
        }
    }

    pub fn write_c_src<W: io::Write>(&self, name: &str, f: &mut W) -> Result<(), io::Error> {
        writeln!(f, "#include <stdint.h>\n#include \"song.h\"\nconst uint32_t {}_len = {};\n\nconst struct Note {}[] = {{", 
            name, self.notes.len(), name)?;
        for note in &self.notes {
            writeln!(f, "{{ .delta_time={}, .note={} }},", note.delta_time, note.note)?;
        }
        writeln!(f, "}};")?;
        Ok(())
    }
}

use ghakuf::{messages::*, reader::*};
impl Handler for Tones {
    fn header(&mut self, format: u16, track: u16, time_base: u16) {
        println!(
            "MIDI Header Dump --
            Format Number: {}
            Number of Tracks: {}
            Raw Timebase: 0x{:X}
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
        // Write any NoteOn or NoteOff messages onto the stream, setting the MSB to indicate on / off.
        match event {
            MidiEvent::NoteOn{ note, .. } => self.notes.push(Note { delta_time, note: *note | (1 << 7)}),
            MidiEvent::NoteOff{ note, .. } => self.notes.push(Note { delta_time, note: *note & !(1 << 7)}),
            _ => (), // We don't care about the other events
        }
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
    /// Note field of MIDI spec. Remember first bit is direction (turn on/off).
    note: u8,
    /// How long from the last event?
    delta_time: u32,
}
