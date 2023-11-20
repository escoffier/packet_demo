extern crate af_packet;
extern crate num_cpus;
extern crate pktparse;
extern crate nom;

use std::env;
use std::thread;
use nom::IResult;
use pktparse::ip::IPProtocol;
use pktparse::{ethernet, ipv4, tcp};

// use af_packet::tpacket3::TpacketStatsV3;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut fds = Vec::<i32>::new();

    for _ in 0..num_cpus::get() {
        let interface = args[1].clone();

        let mut ring = af_packet::rx::Ring::from_if_name(&interface).unwrap();

        fds.push(ring.socket.fd);

        thread::spawn(move || {
            loop {
                let mut block = ring.get_block();
                for packet in block.get_raw_packets() {

                //    let _res = ethernet::parse_ethernet_frame(&packet.data[82..]).and_then(|(remainder, _frame)|  {
                //         if _frame.ethertype == ethernet::EtherType::IPv4 {
                //             let _ip_data=  ipv4::parse_ipv4_header(&remainder).and_then(|(remainder, iphdr)| {
                //                 if iphdr.protocol == IPProtocol::TCP {
                //                     let _tcp = tcp::parse_tcp_header(&remainder).and_then(|(remainder, tcphdr)| {
                //                         Ok(())
                //                     });
                //                 }
                //                 Ok((remainder, iphdr))
                //             });
                //         }
                //         Ok((remainder, _frame))
                //     }); 

                    if let IResult::Ok((remainder_, frame)) = ethernet::parse_ethernet_frame(&packet.data[82..]) {
                        if frame.ethertype == ethernet::EtherType::IPv4 {
                            if let IResult::Ok((remainder_, iphdr)) = ipv4::parse_ipv4_header(&remainder_) {
                                if iphdr.protocol == IPProtocol::TCP {
                                    if let IResult::Ok((remainder_, tcphdr)) = tcp::parse_tcp_header(&remainder_) {
                                        println!("{:?}", tcphdr);
                                        print!("payload len: {}", remainder_.len());
                                    }

                                }
                            }
                        }  
                    }
                }
                block.mark_as_consumed();
            }
        });
    }

    let mut packets: u64 = 0;
    let mut drops: u64 = 0;

    loop {
        let mut stats: (u64, u64) = (0, 0);
        for fd in &fds {
            let ring_stats = af_packet::rx::get_rx_statistics(*fd).unwrap();
            stats.0 += ring_stats.tp_drops as u64;
            stats.1 += ring_stats.tp_packets as u64;

        }
    }
    println!("hello world");
}