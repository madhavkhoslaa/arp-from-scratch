use std::mem;
use std::net::Ipv4Addr;
use std::os::fd::AsRawFd;

use libc::{arpreq, sockaddr_in, AF_INET, ATF_COM, ATF_PERM, SIOCSARP};
use nix::ioctl_write_ptr_bad;
use nix::sys::socket::{socket, AddressFamily, SockFlag, SockType};

ioctl_write_ptr_bad!(set_arp_entry, SIOCSARP, arpreq);

/// Updates or creates a static ARP table entry via the SIOCSARP ioctl.
/// Requires root privileges (CAP_NET_ADMIN).
/// `ifindex` is accepted for signature compatibility but SIOCSARP keys off
/// the interface name, so we always target eth0.
pub fn update_kernel_arp(ip: [u8; 4], mac: [u8; 6], _ifindex: u32) {
    let interface = "eth0";

    let fd = match socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), None) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("ARP update: socket failed: {}", e);
            return;
        }
    };

    let mut req: arpreq = unsafe { mem::zeroed() };

    // Protocol address (IPv4) field.
    let sin_ip: sockaddr_in = unsafe {
        let mut s: sockaddr_in = mem::zeroed();
        s.sin_family = AF_INET as u16;
        s.sin_addr.s_addr = u32::from_ne_bytes(ip);
        s
    };
    unsafe {
        std::ptr::copy_nonoverlapping(
            &sin_ip as *const sockaddr_in as *const u8,
            &mut req.arp_pa as *mut _ as *mut u8,
            mem::size_of::<sockaddr_in>(),
        );
    }

    // Hardware address (MAC) field.
    req.arp_ha.sa_family = libc::ARPHRD_ETHER as u16;
    for i in 0..6 {
        req.arp_ha.sa_data[i] = mac[i] as i8;
    }

    // Interface name.
    let bytes = interface.as_bytes();
    if bytes.len() >= req.arp_dev.len() {
        eprintln!("ARP update: interface name too long");
        return;
    }
    for (i, &byte) in bytes.iter().enumerate() {
        req.arp_dev[i] = byte as i8;
    }

    req.arp_flags = ATF_COM | ATF_PERM;

    unsafe {
        if let Err(e) = set_arp_entry(fd.as_raw_fd(), &req) {
            eprintln!("ARP update: ioctl SIOCSARP failed for {}: {}", Ipv4Addr::from(ip), e);
        } else {
            println!("ARP: {} -> {:02x?} via {}", Ipv4Addr::from(ip), mac, interface);
        }
    }
}