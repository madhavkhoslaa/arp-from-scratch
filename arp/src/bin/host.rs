use libc;
use std::net::Ipv4Addr;

fn main() {
    let fd: i32;
    unsafe {
        let proto = (libc::ETH_P_ARP as u16).to_be() as libc::c_int;
        fd = libc::socket(libc::AF_PACKET, libc::SOCK_RAW, proto);
    }
    let mut buffer = [0u8; 2048];
    loop {
        unsafe {
            let result: isize = libc::recv(
                fd,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
                0,
            );
            if result < 0 {
                panic!("Unable to read the socket");
            }
        }
        // dbg!(buffer);
        // parse the source mac and the destination mac
        // parse the source and destination IP if there is any(How do we check that if there is  any IPv4 packet)
        let sender_mac = &buffer[0..6];
        let target_mac = &buffer[6..12];
        // ARP payload starts after the 14-byte Ethernet header; sender/target
        // protocol addresses sit after the hw/proto type fields and MACs.
        let sender_ip = Ipv4Addr::new(buffer[28], buffer[29], buffer[30], buffer[31]);
        let target_ip = Ipv4Addr::new(buffer[38], buffer[39], buffer[40], buffer[41]);

        println!("sender_mac {:x?}", sender_mac);
        println!("target_mac{:x?}", target_mac);
        println!("send_ip {}", sender_ip);
        println!("target_ip{}", target_ip);
    }
}
