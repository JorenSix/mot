use std::sync::mpsc::Sender;

use std::net::UdpSocket;
use std::net::SocketAddrV4;
use std::str::FromStr;

use rand::Rng; // 0.8.0

use rosc::OscPacket;
use rosc::OscMessage;
use rosc::encoder;
use rosc::OscType;


pub struct OscSender{
	sock:UdpSocket,
	to_addr:SocketAddrV4
}

impl OscSender{
	pub fn new(osc_target_address: String)-> OscSender{
		let num = rand::thread_rng().gen_range(12000..13000);
		let sock = UdpSocket::bind("0.0.0.0:".to_owned() + &num.to_string()).unwrap();
        let to_addr = SocketAddrV4::from_str(&osc_target_address).unwrap();
        OscSender{
        	sock: sock,
        	to_addr: to_addr
        }
	}

	pub fn send(&self,addr: String, osc_args: Vec<OscType>){
		 let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
                    addr: addr,
                    args: osc_args,
                })).unwrap();
		 self.sock.send_to(&msg_buf, self.to_addr).unwrap();
	}
}

pub struct OscServer {
    sock_addr: String,
    packet_handler: fn(OscPacket) -> u32,
    pub last_returned: u32,
    pub is_listening: bool,
}

impl OscServer{

    pub fn new(sock_addr: &str, packet_handler: fn(OscPacket) -> u32) -> OscServer {
        OscServer{
            sock_addr: sock_addr.to_string(),
            packet_handler: packet_handler,
            last_returned: 0,
            is_listening: false,
        }
    }

    pub fn listen(&mut self,sender: &Sender<u32> )  -> u32{
        self.listen_and_stop(-1,sender) 
    }

    pub fn listen_and_stop(&mut self,max_nr_of_messages: i32,sender: &Sender<u32>) -> u32{
        self.is_listening = true;

        let addr = match SocketAddrV4::from_str(&self.sock_addr.to_string()) {
            Ok(addr) => addr,
            Err(_) => panic!("Invalid OSC ip address addr"),
        };

        let sock = UdpSocket::bind(addr).unwrap();
        println!("Listening to {}", addr);

        let mut buf = [0u8; rosc::decoder::MTU];
        let mut return_value: u32 = 0;

        let mut msg_counter = 0;
        while max_nr_of_messages == -1 || msg_counter < max_nr_of_messages {
            match sock.recv_from(&mut buf) {
                Ok((size, _addr)) => {
                    //println!("Received packet with size {} from: {}", size, addr);
                    let (_i, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                   	return_value = (self.packet_handler)(packet);
                    self.last_returned = return_value;
                    sender.send(return_value).unwrap();
                    msg_counter = msg_counter + 1;
                }
                Err(e) => {
                    println!("Error receiving from socket: {}", e);
                    break;
                }
            }
        }

        return_value
    }

    pub fn echo_osc_packet(packet: OscPacket) -> u32 {
        match packet {
            OscPacket::Message(msg) => {
                match msg.addr.as_str(){
                    _ => {
                        println!("msg: {:?}", msg);
                    }
                }
            }

            OscPacket::Bundle(bundle) => {
                println!("OSC Bundle: {:?}", bundle);
            }
        }

        0
    }
}
