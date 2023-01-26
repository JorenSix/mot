use midir::{MidiOutput,MidiInput, Ignore,MidiOutputConnection};

use std::io::stdin;

pub struct MidiIn {
	midi_in: MidiInput,
	midi_in_index: usize,
}

impl MidiIn {

	pub fn new(midi_in_index: usize) -> MidiIn {

		let mut midi_in = MidiInput::new("midir reading input").unwrap();
		midi_in.ignore(Ignore::None);

        MidiIn{
        	midi_in: midi_in,
            midi_in_index: midi_in_index,
        }
    }

    pub fn listen<F,T: 'static +  Send>(self, callback: F,data: T,)  where F: FnMut(u64, &[u8], &mut T) + Send + 'static {
    	let in_ports = self.midi_in.ports();
    	let in_port = &in_ports[self.midi_in_index];

    	println!("#Receiving MIDI from {:?} ",self.midi_in.port_name(&in_port).unwrap());
    	// Get an input port (read from console if multiple are available)
    	   // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
        let _conn_in = self.midi_in.connect(&in_port, "midir-read-input", callback,data);
    	
        
    	let mut input = String::new();
        input.clear();
        stdin().read_line(&mut input).unwrap(); // wait for next enter key press
        println!("Closing MIDI port");
        
        
    }


	pub fn list_midi_input_ports(){
	    let mut midi_in = MidiInput::new("midir reading input").unwrap();
	    midi_in.ignore(Ignore::None);
	    
	    let in_ports = midi_in.ports();
	    println!("\nAvailable MIDI input ports:");
	    for (i, p) in in_ports.iter().enumerate() {
	        println!("{}: {}", i, midi_in.port_name(p).unwrap());
	    }
	    println!("");
	    
	}

	pub fn check_midi_input_port_index(midi_in_index: usize) -> bool{
		let mut midi_in = MidiInput::new("midir reading input").unwrap();
	    midi_in.ignore(Ignore::None);
	    
	    if midi_in_index >= midi_in.ports().len() {
	    	println!("{} is an invalid MIDI input port index.", midi_in_index);
	    	println!("Choose one of the following:");
	    	MidiIn::list_midi_input_ports();
	    }
	    midi_in_index < midi_in.ports().len()
	}



}

pub struct MidiOut {
	conn_out: MidiOutputConnection
}

impl MidiOut {

	pub fn new(midi_out_index: usize) -> MidiOut {
		let midi_out = MidiOutput::new("midir reading output").unwrap();
    	let out_ports = midi_out.ports();
    	let out_port = &out_ports[midi_out_index];

        MidiOut{
            conn_out:  midi_out.connect(out_port, "aika-out").unwrap()
        }
    }

    //pub fn send(&mut self, midi_cmd: u8,  data1:u8 , data2: u8){
    //	self.conn_out.send(&[midi_cmd, data1, data2]).unwrap();
    //}

    pub fn send_full(&mut self, message: &[u8]){
    	self.conn_out.send(message).unwrap();
    }


	pub fn list_midi_output_ports(){
	    let midi_out = MidiOutput::new("midir reading input").unwrap();
	    
	    let out_ports = midi_out.ports();
	    println!("\nAvailable MIDI output ports:");
	    for (i, p) in out_ports.iter().enumerate() {
	        println!("{}: {}", i, midi_out.port_name(p).unwrap());
	    }
	    println!("");
	}

	pub fn check_midi_output_port_index(midi_out_index: usize) -> bool{
		let midi_out = MidiOutput::new("midir reading input").unwrap();
	    
	    if midi_out_index >= midi_out.ports().len() {
	    	println!("{} is an invalid MIDI output port index.", midi_out_index);
	    	println!("Choose one of the following:");
	    	MidiOut::list_midi_output_ports();
	    }
	    midi_out_index < midi_out.ports().len()
	}
}