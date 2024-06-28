extern crate af_packet;
extern crate nom;
extern crate num_cpus;
extern crate pktparse;

use nom::IResult;
use smoltcp::wire::EthernetProtocol;
use smoltcp::wire::IpAddress;
use std::env;
use std::thread;
// use pktparse::ip;
// use pktparse::ip::IPProtocol;
// use pktparse::{ethernet, ipv4, tcp};
use smoltcp::phy::ChecksumCapabilities;
use smoltcp::wire::{IpProtocol,EthernetFrame,EthernetRepr, Ipv4Packet, Ipv4Repr, TcpPacket, TcpRepr};

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

                    // if let IResult::Ok((remainder_, frame)) = ethernet::parse_ethernet_frame(&packet.data[82..]) {
                    //     if frame.ethertype == ethernet::EtherType::IPv4 {
                    //         if let IResult::Ok((remainder_, iphdr)) = ipv4::parse_ipv4_header(&remainder_) {
                    //             if iphdr.protocol == IPProtocol::TCP {
                    //                 if let IResult::Ok((remainder_, tcphdr)) = tcp::parse_tcp_header(&remainder_) {
                    //                     println!("{:?}", tcphdr);
                    //                     println!("payload len: {}", remainder_.len());
                    //                 }

                    //             }
                    //         }
                    //     }
                    // }
                    let frame = EthernetFrame::new_checked(&packet.data).expect("eth frame");
                    let parsedFrame = EthernetRepr::parse(&frame);
                    let mut parsedFrame = match parsedFrame {
                        Ok(f) => f,
                        Err(e) => return,
                    }; 
                    println!("eth header {:?}", parsedFrame);

                    if frame.ethertype() == EthernetProtocol::Ipv4 {
                        let paylaod = frame.payload();
                        let ip_packet = Ipv4Packet::new_checked(&paylaod).expect("truncated packet");
                        println!("protocol: {}", ip_packet.next_header());
                        let parsed: Result<Ipv4Repr, smoltcp::wire::Error> = Ipv4Repr::parse(&ip_packet, &ChecksumCapabilities::default());
                        println!("ip header {:?}", parsed);
                        let payload = ip_packet.payload();

                        let tcp = TcpPacket::new_checked(&payload).expect("truncated packet");
                        let src_addr =  IpAddress::v4(0,0,0,0);
                        let dst_addr =  IpAddress::v4(0,0,0,0);
    
                        let parsed_tcp = TcpRepr::parse(&tcp, &src_addr, &dst_addr, &ChecksumCapabilities::default());
                        match parsed_tcp {
                            Ok(seg) => {
                                println!("tcp flag: {:?}", seg.control);
                            }
                            Err(_) => todo!(),
                        }
                    }

                    // Ipv4Packet::new_checked(packet.data).and_then(|ipv4_packet| Ok({
                    //     ipv4_packet.payload();
                    //     ipv4_packet.next_header();
                    //     let p = Ipv4Repr::parse(&ipv4_packet, &ChecksumCapabilities::default());
                    //     p.and_then(|ipv4_repr: Ipv4Repr|Ok({
                    //         if ipv4_repr.next_header == IpProtocol::Tcp {

                    //         }
                    //     }));
                    // }));
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
}
