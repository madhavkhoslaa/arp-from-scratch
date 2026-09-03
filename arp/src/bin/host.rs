use arp::utils::{build_arp_reply, get_ip, get_mac, update_kernel_arp};
use std::mem::{size_of, zeroed};
use std::net::Ipv4Addr;

fn main() {
    let my_ip = get_ip("eth0");
    let my_mac = get_mac("eth0");
    println!("my_ip: {}", my_ip);
    println!("my_mac: {:x?}", my_mac);
    let fd: i32;
    unsafe {
        // Important Learning lesson. Network bytes are in big endian and CPU's with in little endian
        // Creating a socket to send and get packets from kernel
        let proto = (libc::ETH_P_ALL as u16).to_be() as libc::c_int;
        fd = libc::socket(libc::AF_PACKET, libc::SOCK_RAW, proto);
    }
    let ifindex = unsafe { libc::if_nametoindex(b"eth0\0".as_ptr() as *const libc::c_char) };
    if ifindex == 0 {
        panic!("No interface index for eth0");
    }
    let mut buffer = [0u8; 2048];
    loop {
        let result: isize;
        unsafe {
            result = libc::recv(
                fd,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
                0,
            );
            if result < 0 {
                panic!("Unable to read the socket");
            }
        }
        if result < 14 {
            continue;
        }
        // Only process ARP frames (EtherType 0x0806 at bytes 12..14).
        let ethertype = u16::from_be_bytes([buffer[12], buffer[13]]);
        if ethertype != 0x0806 {
            continue;
        }
        // parse the source mac and the destination mac
        // ARP payload starts after the 14-byte Ethernet header; sender/target
        // protocol addresses sit after the hw/proto type fields and MACs.
        let dest_mac = &buffer[0..6]; // Ethernet destination first
        let src_mac = &buffer[6..12]; // Ethernet source second
        let sender_ip = Ipv4Addr::new(buffer[28], buffer[29], buffer[30], buffer[31]);
        let target_ip = Ipv4Addr::new(buffer[38], buffer[39], buffer[40], buffer[41]);

        println!("src_mac{:?}", src_mac);
        println!("dest_mac{:x?}", dest_mac);
        println!("send_ip {}", sender_ip);
        println!("target_ip{}", target_ip);

        // Passive learning: every ARP frame tells us sender_ip -> src_mac.
        // Learn it regardless of who the ARP is for.
        let mac = [
            src_mac[0], src_mac[1], src_mac[2], src_mac[3], src_mac[4], src_mac[5],
        ];
        update_kernel_arp(sender_ip.octets(), mac, ifindex);
        println!("learned {} -> {:x?}", sender_ip, src_mac);

        let is_broadcast = dest_mac == &[0xff; 6];
        // A broadcast ARP request asking for MY ip -> I should reply
        if is_broadcast && my_ip == target_ip {
            let frame = build_arp_reply(my_mac, my_ip, src_mac, sender_ip, 2);
            let mut sa: libc::sockaddr_ll = unsafe { zeroed() };
            sa.sll_family = libc::AF_PACKET as u16;
            sa.sll_protocol = (libc::ETH_P_ARP as u16).to_be();
            sa.sll_ifindex = ifindex as i32;
            sa.sll_halen = 6;
            sa.sll_addr[..6].copy_from_slice(src_mac);
            let n = unsafe {
                libc::sendto(
                    fd,
                    frame.as_ptr() as *const libc::c_void,
                    frame.len(),
                    0,
                    &sa as *const libc::sockaddr_ll as *const libc::sockaddr,
                    size_of::<libc::sockaddr_ll>() as libc::socklen_t,
                )
            };
            if n >= 0 {
                println!("replied to {} at {:x?}", sender_ip, src_mac);
            }
        }
    }
}
