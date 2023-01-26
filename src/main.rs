mod midi_io;
mod osc_io;

use clap::{Arg, Command};
use std::str::FromStr;
use std::net::{SocketAddrV4};

use rosc::OscType;

use std::sync::mpsc::channel;


struct MidiRoundTrip {
    midi_in: midi_io::MidiIn,
    midi_out: midi_io::MidiOut
}

impl MidiRoundTrip{

    fn new(midi_in_port_index: usize,midi_out_port_index: usize) -> MidiRoundTrip {
        MidiRoundTrip{
            midi_in: midi_io::MidiIn::new(midi_in_port_index),
            midi_out: midi_io::MidiOut::new(midi_out_port_index)
        }
    }

    fn respond_to_midi(mut self){
        self.midi_in.listen(move |_time_stamp, message, _| {
            self.midi_out.send_full(message);
        },());
    }
}


struct MidiEcho {
    midi_in: midi_io::MidiIn
}

impl MidiEcho{

    fn new(midi_in_port_index: usize, ) -> MidiEcho {
        MidiEcho{
            midi_in: midi_io::MidiIn::new(midi_in_port_index)
        }
    }

    fn echo_midi(self){
        let mut message_index = 0;
        self.midi_in.listen(move |time_stamp, message, _| {
            println!("{} {} {:?}",message_index, time_stamp, message);
            message_index = message_index + 1;
        },());
    }
}


struct MidiToOsc {
    osc_sender: osc_io::OscSender,
    midi_in: midi_io::MidiIn,
    verbose: bool,
    address: String
}

impl MidiToOsc{

    fn new(osc_target_address: &str,midi_in_port_index: usize, verbose: bool, address: &str) -> MidiToOsc {
        MidiToOsc{
            osc_sender: osc_io::OscSender::new(osc_target_address.to_string()),
            verbose: verbose,
            midi_in: midi_io::MidiIn::new(midi_in_port_index),
            address: address.to_string()
        }
    }

    fn midi_to_osc(self){
        let mut message_index = 0;

        self.midi_in.listen(move |time_stamp, message, _| {
            message_index = message_index + 1;        
            if self.verbose {println!("{} {:?}",time_stamp, message);}
            let osc_args = message.iter().map(|&x| OscType::Int(x.into())).collect::<Vec<_>>();
            self.osc_sender.send(self.address.to_string(), osc_args);            
        },());
    }
}


fn is_host_with_port(v: &str) -> Result<String,String>{
    let addr = SocketAddrV4::from_str(v);
    match addr {
        Ok(_addr) => return Ok(String::from(v)),
        Err(_e) => Err(String::from("Expects a valid IPv4 address with UDP port: xxx.xxx.xxx.xxx:port")),
    }
}

fn main() {
    let matches = Command::new("mot")
        .bin_name("mot")
        .version("0.1.0")
        .author("Joren Six. <joren.six@ugent.be>")
        .about("mot - Midi and OSC Tools")
        .subcommand_required(true)
        .subcommand(Command::new("midi_to_osc")
            .about("Transport MIDI over OSC")
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
        ).get_matches();


    if let Some(sub_matches) = matches.subcommand_matches("midi_echo") {
        if sub_matches.value_source("list") == Some(clap::parser::ValueSource::CommandLine) {
            println!{"Listing MIDI input devices:"}
            midi_io::MidiIn::list_midi_input_ports();
        }else{
            let midi_input_index: usize = *sub_matches.get_one("midi_input_index").expect("`midi_input_index` is required");
            //let midi_input_index = usize::from_str(sub_matches.value_of("midi_input_index").unwrap()).unwrap();
            if midi_io::MidiIn::check_midi_input_port_index(midi_input_index) {
                 MidiEcho::new(midi_input_index).echo_midi();
            }  
        }
    }

    if let Some(sub_matches) = matches.subcommand_matches("midi_to_osc") {
        if sub_matches.value_source("list") == Some(clap::parser::ValueSource::CommandLine) {
            midi_io::MidiIn::list_midi_input_ports();
        }else{
            let midi_input_index: usize = *sub_matches.get_one("midi_input_index").expect("`midi_input_index` is required");
            let verbose = sub_matches.value_source("verbose") == Some(clap::parser::ValueSource::CommandLine);
            let osc_target_host_address = sub_matches.get_one::<String>("host:port").unwrap();
            let osc_target_osc_address = sub_matches.get_one::<String>("osc_address").unwrap();
           
            if midi_io::MidiIn::check_midi_input_port_index(midi_input_index) {
                MidiToOsc::new(osc_target_host_address,midi_input_index,verbose,osc_target_osc_address).midi_to_osc();
            }
        }
    }

    if let Some(sub_matches) = matches.subcommand_matches("osc_echo") {
        let addr = sub_matches.get_one::<String>("host:port").unwrap();
        let (send, _recv) = channel::<u32>();
        osc_io::OscServer::new(addr,osc_io::OscServer::echo_osc_packet).listen(&send);    
    }

    if let Some(sub_matches) = matches.subcommand_matches("midi_roundtrip_latency") {
        if sub_matches.value_source("list") == Some(clap::parser::ValueSource::CommandLine) {
            midi_io::MidiIn::list_midi_input_ports();
            midi_io::MidiOut::list_midi_output_ports();
        }else{
            let midi_input_index: usize = *sub_matches.get_one("midi_input_index").expect("`midi_input_index` is required");
            let midi_output_index: usize = *sub_matches.get_one("midi_output_index").expect("`midi_output_index` is required");
            println!{"MIDI roundtrip latency application."}
            if midi_io::MidiIn::check_midi_input_port_index(midi_input_index) && midi_io::MidiOut::check_midi_output_port_index(midi_output_index)   {
                 MidiRoundTrip::new(midi_input_index,midi_output_index).respond_to_midi();
            }  
        }
    }
}
