// A switch has interfaces
// An interface is mapped to a mac address
// it is a passive learner. I.E it does not send any ARP broadcats.
// But it learns from all of the layer 2 packets going through it.
// That is the reason why we have a boot.rs process that runs after the host process runs

use std::collections::HashMap;
use std::mem::{size_of, zeroed};

const PORTS: [&str; 3] = ["eth0", "eth1", "eth2"];

fn main() {
    let mut mac_table: HashMap<String, String> = HashMap::new();

    // Open a raw socket per interface so we can send out the right one
    let mut fds: HashMap<String, i32> = HashMap::new();
    let mut ifindices: HashMap<String, u32> = HashMap::new();
    unsafe {
        let proto = (libc::ETH_P_ALL as u16).to_be() as libc::c_int;
        for port in PORTS {
            let cname = format!("{}\0", port);
            let fd = libc::socket(libc::AF_PACKET, libc::SOCK_RAW, proto);
            if fd < 0 {
                eprintln!("Failed to open socket for {}", port);
                continue;
            }
            let ifindex = libc::if_nametoindex(cname.as_ptr() as *const libc::c_char);

            // Put the interface into promiscuous mode so we see ALL
            // frames on the wire, not just those addressed to our own MAC.
            let mut ifr: libc::ifreq = std::mem::zeroed();
            std::ptr::copy_nonoverlapping(cname.as_ptr(), ifr.ifr_name.as_mut_ptr() as *mut u8, port.len());
            ifr.ifr_ifru.ifru_flags = libc::IFF_PROMISC as i16;
            libc::ioctl(fd, libc::SIOCGIFFLAGS, &mut ifr);
            ifr.ifr_ifru.ifru_flags |= libc::IFF_PROMISC as i16;
            libc::ioctl(fd, libc::SIOCSIFFLAGS, &ifr);

            fds.insert(port.to_string(), fd);
            ifindices.insert(port.to_string(), ifindex);
            println!("Switch: opened socket on {} (fd={}, ifindex={})", port, fd, ifindex);
        }
    }

    loop {
        // TODO: do the same loop as host
        for port in PORTS {
            let fd = fds[port];
            let mut buffer = [0u8; 2048];
            let result = unsafe {
                libc::recv(
                    fd,
                    buffer.as_mut_ptr() as *mut libc::c_void,
                    buffer.len(),
                    libc::MSG_DONTWAIT,
                )
            };
            if result <= 0 {
                continue;
            }

            let frame = &buffer[..result as usize];

            // Only forward ARP (0x0806) and IPv4 (0x0800) frames.
            let ethertype = u16::from_be_bytes([buffer[12], buffer[13]]);
            if ethertype != 0x0806 && ethertype != 0x0800 {
                continue;
            }

            // Drop multicast (bit 0 set) but keep broadcast (ff:ff:ff:ff:ff:ff).
            // Multicast frames from the docker host/kernel cause storms with ETH_P_ALL.
            if frame[0] & 0x01 == 1 && frame[0..6] != [0xff; 6] {
                continue;
            }

            // Learn: src mac is at bytes 6..12, linked to the inbound interface
            let src_mac = format!(
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                frame[6], frame[7], frame[8], frame[9], frame[10], frame[11]
            );
            let was_known = mac_table.contains_key(&src_mac);
            mac_table.insert(src_mac.clone(), port.to_string());
            if was_known {
                println!(
                    "Switch: learned {} on {} (already known)",
                    src_mac, port
                );
            } else {
                println!("Switch: learned {} on {}", src_mac, port);
            }
            println!("Switch: mac_table mutation -> {:?}", mac_table);

            let dest_mac = format!(
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                frame[0], frame[1], frame[2], frame[3], frame[4], frame[5]
            );

            let is_broadcast = frame[0..6] == [0xff; 6];

            if is_broadcast {
                // TODO: Broadcast came
                // Send flood to every interface other than where the request came from other than the
                // inbound_interface
                println!(
                    "Switch: broadcast from {} on {} -> flooding others",
                    src_mac, port
                );
                for out_port in PORTS {
                    if out_port == port {
                        continue;
                    }
                    println!("Switch:   flood to {}", out_port);
                    send_frame(
                        &fds[out_port],
                        &ifindices[out_port],
                        frame,
                    );
                }
            } else {
                // TODO: basic functionality
                // Just forward messages to interfaces based on the mac address
                match mac_table.get(&dest_mac) {
                    Some(out_port) if out_port != port => {
                        println!(
                            "Switch: unicast {} -> {} from {} (dest known on {})",
                            dest_mac, out_port, port, out_port
                        );
                        send_frame(
                            &fds[out_port],
                            &ifindices[out_port],
                            frame,
                        );
                    }
                    _ => {
                        // Unknown destination -> flood like a broadcast
                        println!(
                            "Switch: unicast to unknown {} on {} -> flooding",
                            dest_mac, port
                        );
                        for out_port in PORTS {
                            if out_port == port {
                                continue;
                            }
                            println!("Switch:   flood to {}", out_port);
                            send_frame(
                                &fds[out_port],
                                &ifindices[out_port],
                                frame,
                            );
                        }
                    }
                }
            }
        }
    }
}

fn send_frame(fd: &i32, ifindex: &u32, frame: &[u8]) {
    let mut sa: libc::sockaddr_ll = unsafe { zeroed() };
    sa.sll_family = libc::AF_PACKET as u16;
    sa.sll_protocol = (libc::ETH_P_ARP as u16).to_be();
    sa.sll_ifindex = *ifindex as i32;
    sa.sll_halen = 6;
    sa.sll_addr[..6].copy_from_slice(&frame[0..6]);
    unsafe {
        libc::sendto(
            *fd,
            frame.as_ptr() as *const libc::c_void,
            frame.len(),
            0,
            &sa as *const libc::sockaddr_ll as *const libc::sockaddr,
            size_of::<libc::sockaddr_ll>() as libc::socklen_t,
        );
    }
}
