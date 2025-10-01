mod mdns_service_manager;
mod midi_io;
mod osc_io;
mod lua_processor;

use clap::{Arg, Command};
use osc_io::OscSender;
use std::io::{self, BufRead};
use std::net::SocketAddrV4;
use std::str::FromStr;

use rosc::OscPacket;
use rosc::OscType;

use once_cell::sync::OnceCell;

use std::sync::mpsc::channel;
use std::sync::Mutex;

use std::ops::DerefMut;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use std::thread;

struct MidiRoundTrip {
    midi_in: midi_io::MidiIn,
    midi_out: midi_io::MidiOut,
}

impl MidiRoundTrip {
    fn new(midi_in_port_index: usize, midi_out_port_index: usize) -> MidiRoundTrip {
        MidiRoundTrip {
            midi_in: midi_io::MidiIn::new(midi_in_port_index),
            midi_out: midi_io::MidiOut::new(midi_out_port_index),
        }
    }

    fn respond_to_midi(mut self) {
        self.midi_in.listen(
            move |_time_stamp, message, _| {
                self.midi_out.send_full(message);
            },
            (),
        );
    }
}

struct MidiEcho {
    midi_in: midi_io::MidiIn,
}

impl MidiEcho {
    fn new(midi_in_port_index: usize) -> MidiEcho {
        MidiEcho {
            midi_in: midi_io::MidiIn::new(midi_in_port_index),
        }
    }

    fn echo_midi(self) {
        let mut message_index = 0;
        self.midi_in.listen(
            move |time_stamp, message, _| {
                println!("{} {} {:?}", message_index, time_stamp, message);
                message_index = message_index + 1;
            },
            (),
        );
    }
}

struct OscToMidi {
    midi_out: midi_io::MidiOut,
    verbose: bool,
    osc_host_address: String,
    osc_path_address: String,
}

unsafe impl Sync for OscToMidi {}

impl core::fmt::Debug for OscToMidi {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OSC to MIDI")
            .field("verbose", &self.verbose)
            .field("osc_host_address", &self.osc_host_address)
            .finish()
    }
}

static INSTANCE: OnceCell<Mutex<OscToMidi>> = OnceCell::new();

impl OscToMidi {
    fn new(
        osc_host_address: &str,
        midi_out_port_index: usize,
        verbose: bool,
        osc_path_address: &str,
    ) -> OscToMidi {
        OscToMidi {
            osc_host_address: osc_host_address.to_string(),
            verbose: verbose,
            midi_out: midi_io::MidiOut::new(midi_out_port_index),
            osc_path_address: osc_path_address.to_string(),
        }
    }

    fn osc_to_midi(osc_host_address: &str,running: Arc<AtomicBool>) {
        let (send, _recv) = channel::<u32>();
        let callback = || OscToMidi::forward_osc_packet_to_midi;
        osc_io::OscServer::new(osc_host_address, callback()).listen_with_interrupt(&send,running);
    }

    fn send_midi_message(message: &[u8]) {
        INSTANCE
            .get()
            .expect("OSC to MIDI not initialized")
            .lock()
            .unwrap()
            .deref_mut()
            .midi_out
            .send_full(message);
    }

    fn osc_path_address() -> String {
        return INSTANCE
            .get()
            .expect("OSC to MIDI not initialized")
            .lock()
            .unwrap()
            .deref_mut()
            .osc_path_address
            .to_string();
    }

    fn verbose() -> bool {
        return INSTANCE
            .get()
            .expect("OSC to MIDI not initialized")
            .lock()
            .unwrap()
            .deref_mut()
            .verbose;
    }

    fn forward_osc_packet_to_midi(packet: OscPacket) -> u32 {
        match packet {
            OscPacket::Message(msg) => {
                let path_address = OscToMidi::osc_path_address();

                if msg.addr.as_str() == path_address {
                    let mut midi_data = vec![];
                    if OscToMidi::verbose() {
                        println!("OSC msg received: {:?}", msg);
                    }

                    for osc_arg in msg.args {
                        match osc_arg.int() {
                            Some(v) => {
                                let number: i32 = v;
                                if number >= 0 && number < 256 {
                                    midi_data.push(number as u8)
                                } else {
                                    if OscToMidi::verbose() {
                                        println!(
                                            "Ignored number not fitting in byte: {:?}",
                                            number
                                        );
                                    }
                                }
                            }
                            _ => {
                                if OscToMidi::verbose() {
                                    println!("Ignored unsupported OSC type");
                                }
                            }
                        }
                    }

                    if midi_data.len() > 0 {
                        //Max 1000 bytes!
                        let mut message: [u8; 1000] = [0 as u8; 1000];
                        let mut index = 0;
                        for x in &midi_data {
                            message[index] = *x;
                            index = index + 1;
                        }
                        OscToMidi::send_midi_message(&message[0..midi_data.len()]);
                    }
                } else {
                    if OscToMidi::verbose() {
                        println!("Ignored message on OSC address: {:?}", msg.addr.as_str());
                    }
                }
            }

            OscPacket::Bundle(bundle) => {
                println!("OSC Bundle: {:?}", bundle);
            }
        }
        return 0;
    }
}

struct MidiToOsc {
    osc_sender: osc_io::OscSender,
    midi_in: midi_io::MidiIn,
    verbose: bool,
    osc_path_address: String,
}

impl MidiToOsc {
    fn new(
        osc_host_address: &str,
        midi_in_port_index: usize,
        verbose: bool,
        osc_path_address: &str,
    ) -> MidiToOsc {
        MidiToOsc {
            osc_sender: osc_io::OscSender::new(osc_host_address.to_string()),
            verbose: verbose,
            midi_in: midi_io::MidiIn::new(midi_in_port_index),
            osc_path_address: osc_path_address.to_string(),
        }
    }

    fn midi_to_osc(self) {
        let mut message_index = 0;

        self.midi_in.listen(
            move |time_stamp, message, _| {
                message_index = message_index + 1;
                if self.verbose {
                    println!("{} {:?}", time_stamp, message);
                }
                let osc_args = message
                    .iter()
                    .map(|&x| OscType::Int(x.into()))
                    .collect::<Vec<_>>();
                self.osc_sender
                    .send(self.osc_path_address.to_string(), osc_args);
            },
            (),
        );
    }
}

struct LuaMidiProcessor {
    midi_in: midi_io::MidiIn,
    midi_out: midi_io::MidiOut,
    lua_processor: lua_processor::LuaProcessor,
    verbose: bool,
}

impl LuaMidiProcessor {
    fn new(
        midi_in_port_index: usize,
        midi_out_port_index: usize,
        lua_script_path: &str,
        verbose: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(LuaMidiProcessor {
            midi_in: midi_io::MidiIn::new(midi_in_port_index),
            midi_out: midi_io::MidiOut::new(midi_out_port_index),
            lua_processor: lua_processor::LuaProcessor::new(lua_script_path)?,
            verbose,
        })
    }

    fn process_midi(mut self) {
        let mut message_count = 0;
        
        self.midi_in.listen(
            move |_timestamp, message, _| {
                message_count += 1;
                
                if self.verbose {
                    println!("Received MIDI message #{}: {:?}", message_count, message);
                }
                
                // Process through Lua script
                match self.lua_processor.process_message(message) {
                    Ok(Some(processed_message)) => {
                        if self.verbose {
                            println!("Sending processed message: {:?}", processed_message);
                        }
                        self.midi_out.send_full(&processed_message);
                    }
                    Ok(None) => {
                        if self.verbose {
                            println!("Message filtered by Lua script");
                        }
                    }
                    Err(e) => {
                        eprintln!("Lua processing error: {}", e);
                    }
                }
            },
            (),
        );
    }
}

fn is_host_with_port(v: &str) -> Result<String, String> {
    let addr = SocketAddrV4::from_str(v);
    match addr {
        Ok(_addr) => return Ok(String::from(v)),
        Err(_e) => Err(String::from(
            "Expects a valid IPv4 address with UDP port: xxx.xxx.xxx.xxx:port",
        )),
    }
}

fn osc_send(osc_target_host_address: &str, verbose: bool) {
    let stdin = io::stdin();

    let osc_sender = OscSender::new(osc_target_host_address.to_string());
    for line in stdin.lock().lines() {
        let line = line.unwrap(); // Handle potential error
        let mut tokens: Vec<&str> = line.split_whitespace().collect();

        // do not send empty messages
        if tokens.len() == 0 {
            if verbose {
                print!("Empty message; nothing send\n");
            }
            continue;
        }

        // else

        // check if the first token is a valid osc method (starts with '/')
        let osc_method = match tokens.get(0) {
            Some(v) => {
                if v.starts_with('/') {
                    v.to_string()
                } else {
                    format!("/{}", v)
                }
            }
            None => {
                println!("No OSC method provided");
                continue;
            }
        };

        // remove the first token
        tokens.remove(0);

        let mut osc_args: Vec<OscType> = Vec::new();

        for token in tokens {
            if let Ok(int) = token.parse::<i32>() {
                osc_args.push(OscType::Int(int));
            } else if let Ok(float) = token.parse::<f32>() {
                osc_args.push(OscType::Float(float));
            } else {
                osc_args.push(OscType::String(token.to_string()));
            }
        }

        osc_sender.send(osc_method.to_string(), osc_args.clone());

        if verbose {
            print!(
                "Sent OSC message to {} with args {:?}\n",
                osc_target_host_address, osc_args
            );
        }
    }
}

fn setup_interrupt_handler() -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        #[cfg(debug_assertions)]
        println!("\nReceived Ctrl+C, shutting down gracefully...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    
    running
}

fn main() {
    let matches = Command::new("mot")
        .bin_name("mot")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Joren Six. <joren.six@ugent.be>")
        .about("mot - Midi and OSC Tools")
        .subcommand_required(true)
        .subcommand(Command::new("midi_to_osc")
            .about("Send MIDI over OSC")
            .arg(Arg::new("verbose")
                .short('v')
                .num_args(0)
                .required(false)
                .help("print debug information verbosely"))
            .arg(Arg::new("host:port")
                .default_value("127.0.0.1:1234")
                .help("The host:port to send OSC data to")
                .value_name("host:port")
                .value_parser(is_host_with_port))
            .arg(Arg::new("osc_address")
                .default_value("/midi")
                .help("The OSC address to send MIDI to.")
                .value_name("OSC_address"))
            .arg(Arg::new("midi_input_index")
                .default_value("0") 
                .value_parser(clap::value_parser!(usize))
                .help("MIDI input device index. List the devices to get the correct index."))
            .arg(Arg::new("list")
                .short('l')
                .num_args(0)
                .required(false)
                .help("list midi input devices")))
        .subcommand(Command::new("osc_to_midi")
            .about("Translate OSC to MIDI")
            .arg(Arg::new("verbose")
                .short('v')
                .num_args(0)
                .required(false)
                .help("print debug information verbosely"))
            .arg(Arg::new("host:port")
                .default_value("127.0.0.1:1234")
                .help("The host:port to receive OSC data from")
                .value_name("host:port")
                .value_parser(is_host_with_port))
            .arg(Arg::new("osc_address")
                .default_value("/midi")
                .help("The OSC address to receive data from.")
                .value_name("OSC_address"))
            .arg(Arg::new("midi_output_index")
                .default_value("0") 
                .value_parser(clap::value_parser!(usize))
                .help("MIDI output device index. List the devices to get the correct index."))
            .arg(Arg::new("list")
                .short('l')
                .num_args(0)
                .required(false)
                .help("list MIDI output devices")))
        .subcommand(Command::new("midi_echo")
            .about("Print incoming MIDI messages.")
            .arg(Arg::new("list")
                .short('l')
                .num_args(0)
                .required(false)
                .help("list midi input devices"))
            .arg(Arg::new("midi_input_index")
                .default_value("0")
                .value_parser(clap::value_parser!(usize))
                .help("MIDI input device index. List the devices to get the correct index."))
            )
        .subcommand(Command::new("osc_echo")
            .about("Print incoming OSC messages.")
            .arg(Arg::new("verbose")
                .short('v')
                .help("print verbose information"))
            .arg(Arg::new("host:port")
                .default_value("0.0.0.0:1234")
                .help("the host:port to receive OSC data")
                .value_parser(is_host_with_port))
            )
        .subcommand(Command::new("osc_send")
            .about("Send OSC messages from STDIN. The first token of each line is the OSC method, the rest are the arguments. Only floats, ints and strings are converted to OSC types.")
            .arg(Arg::new("verbose")
                .short('v')
                .help("print verbose information"))
            .arg(Arg::new("host:port")
                .default_value("127.0.0.1:1234")
                .help("the host:port to send OSC data to")
                .value_parser(is_host_with_port))
            )
        .subcommand(Command::new("midi_roundtrip_latency")
            .about("Test MIDI roundtrip latency")
            .arg(Arg::new("list")
                .short('l')
                .num_args(0)
                .required(false)
                .help("list midi input and output devices"))
            .arg(Arg::new("midi_input_index")
                .default_value("0")
                .value_parser(clap::value_parser!(usize))
                .help("MIDI input device index. List the devices to get the correct index."))
            .arg(Arg::new("midi_output_index")
                .default_value("0")
                .value_parser(clap::value_parser!(usize))
                .help("MIDI output device index. List the devices to get the correct index."))
        )
        .subcommand(Command::new("midi_processor")
            .about("Process MIDI messages through a Lua script")
            .arg(Arg::new("verbose")
                .short('v')
                .num_args(0)
                .required(false)
                .help("print debug information verbosely"))
            .arg(Arg::new("list")
                .short('l')
                .num_args(0)
                .required(false)
                .help("list midi input and output devices"))
            .arg(Arg::new("midi_input_index")
                .default_value("0")
                .value_parser(clap::value_parser!(usize))
                .help("MIDI input device index. List the devices to get the correct index."))
            .arg(Arg::new("midi_output_index")
                .default_value("0")
                .value_parser(clap::value_parser!(usize))
                .help("MIDI output device index. List the devices to get the correct index."))
            .arg(Arg::new("script")
                .short('s')
                .long("script")
                .required_unless_present("list")
                .help("Path to the Lua script file"))
        ).get_matches();
    
    let running = setup_interrupt_handler();

    if let Some(sub_matches) = matches.subcommand_matches("midi_echo") {
        if sub_matches.value_source("list") == Some(clap::parser::ValueSource::CommandLine) {
            println! {"Listing MIDI input devices:"}
            midi_io::MidiIn::list_midi_input_ports();
        } else {
            let midi_input_index: usize = *sub_matches
                .get_one("midi_input_index")
                .expect("`midi_input_index` is required");
            //let midi_input_index = usize::from_str(sub_matches.value_of("midi_input_index").unwrap()).unwrap();
            if midi_io::MidiIn::check_midi_input_port_index(midi_input_index) {
                MidiEcho::new(midi_input_index).echo_midi();
            }
        }
    }

    if let Some(sub_matches) = matches.subcommand_matches("osc_send") {
        let osc_target_host_address = sub_matches.get_one::<String>("host:port").unwrap();
        let verbose =
            sub_matches.value_source("verbose") == Some(clap::parser::ValueSource::CommandLine);
        osc_send(osc_target_host_address, verbose);
    }

    if let Some(sub_matches) = matches.subcommand_matches("midi_to_osc") {
        if sub_matches.value_source("list") == Some(clap::parser::ValueSource::CommandLine) {
            midi_io::MidiIn::list_midi_input_ports();
        } else {
            let midi_input_index: usize = *sub_matches
                .get_one("midi_input_index")
                .expect("`midi_input_index` is required");
            let verbose =
                sub_matches.value_source("verbose") == Some(clap::parser::ValueSource::CommandLine);
            let osc_target_host_address = sub_matches.get_one::<String>("host:port").unwrap();
            let osc_target_osc_address = sub_matches.get_one::<String>("osc_address").unwrap();

            if midi_io::MidiIn::check_midi_input_port_index(midi_input_index) {
                MidiToOsc::new(
                    osc_target_host_address,
                    midi_input_index,
                    verbose,
                    osc_target_osc_address,
                )
                .midi_to_osc();
            }
        }
    }

    if let Some(sub_matches) = matches.subcommand_matches("osc_to_midi") {
        if sub_matches.value_source("list") == Some(clap::parser::ValueSource::CommandLine) {
            midi_io::MidiOut::list_midi_output_ports();
        } else {
            let midi_output_index: usize = *sub_matches
                .get_one("midi_output_index")
                .expect("`midi_output_index` is required");
            let verbose =
                sub_matches.value_source("verbose") == Some(clap::parser::ValueSource::CommandLine);
            let osc_host_address = sub_matches.get_one::<String>("host:port").unwrap();
            let osc_method_address = sub_matches.get_one::<String>("osc_address").unwrap();

            if midi_io::MidiOut::check_midi_output_port_index(midi_output_index) {
                let osc_to_midi = OscToMidi::new(
                    osc_host_address,
                    midi_output_index,
                    verbose,
                    osc_method_address,
                );
                INSTANCE.set(Mutex::new(osc_to_midi)).unwrap();

                // Register mDNS service to indicate that this is an OSC receiver
                let mut mdns = mdns_service_manager::MdnsService::new().unwrap();

                // Register a simple OSC service
                let port = osc_host_address
                    .split(':')
                    .last()
                    .unwrap_or("8080")
                    .parse::<u16>()
                    .unwrap_or(8080);
                mdns.register("mot-osc-listener", "_osc._udp", port)
                    .unwrap();

                let mdns_running = running.clone();
                thread::spawn(move || {
                    mdns.run_with_interrupt(mdns_running).unwrap();
                });

                OscToMidi::osc_to_midi(osc_host_address,running.clone());
            }
        }
    }

    if let Some(sub_matches) = matches.subcommand_matches("osc_echo") {
        let addr = sub_matches.get_one::<String>("host:port").unwrap();
        
        // Register mDNS service to indicate that this is an OSC receiver
        let mut mdns = mdns_service_manager::MdnsService::new().unwrap();

        // Register a simple OSC service
        let port = addr
            .split(':')
            .last()
            .unwrap_or("8080")
            .parse::<u16>()
            .unwrap_or(8080);
        mdns.register("mot-osc-echo", "_osc._udp", port)
            .unwrap();

        let mdns_running = running.clone();
        thread::spawn(move || {
            mdns.run_with_interrupt(mdns_running).unwrap();
        });

        let (send, _recv) = channel::<u32>();
        osc_io::OscServer::new(addr, osc_io::OscServer::echo_osc_packet).listen_with_interrupt(&send, running);
    }

    if let Some(sub_matches) = matches.subcommand_matches("midi_roundtrip_latency") {
        if sub_matches.value_source("list") == Some(clap::parser::ValueSource::CommandLine) {
            midi_io::MidiIn::list_midi_input_ports();
            midi_io::MidiOut::list_midi_output_ports();
        } else {
            let midi_input_index: usize = *sub_matches
                .get_one("midi_input_index")
                .expect("`midi_input_index` is required");
            let midi_output_index: usize = *sub_matches
                .get_one("midi_output_index")
                .expect("`midi_output_index` is required");
            println! {"MIDI roundtrip latency application."}
            if midi_io::MidiIn::check_midi_input_port_index(midi_input_index)
                && midi_io::MidiOut::check_midi_output_port_index(midi_output_index)
            {
                MidiRoundTrip::new(midi_input_index, midi_output_index).respond_to_midi();
            }
        }
    }

    if let Some(sub_matches) = matches.subcommand_matches("midi_processor") {
        if sub_matches.value_source("list") == Some(clap::parser::ValueSource::CommandLine) {
            midi_io::MidiIn::list_midi_input_ports();
            midi_io::MidiOut::list_midi_output_ports();
        } else {
            let midi_input_index: usize = *sub_matches
                .get_one("midi_input_index")
                .expect("`midi_input_index` is required");
            let midi_output_index: usize = *sub_matches
                .get_one("midi_output_index")
                .expect("`midi_output_index` is required");
            let script_path = sub_matches.get_one::<String>("script").unwrap();
            let verbose = 
                sub_matches.value_source("verbose") == Some(clap::parser::ValueSource::CommandLine);
            
            println!("MIDI Processor");
            println!("Loading Lua script: {}", script_path);
            
            if midi_io::MidiIn::check_midi_input_port_index(midi_input_index)
                && midi_io::MidiOut::check_midi_output_port_index(midi_output_index)
            {
                match LuaMidiProcessor::new(midi_input_index, midi_output_index, script_path, verbose) {
                    Ok(processor) => {
                        println!("Lua script loaded successfully. Processing MIDI...");
                        processor.process_midi();
                    }
                    Err(e) => {
                        eprintln!("Error loading Lua script: {}", e);
                    }
                }
            }
        }
    }
}
