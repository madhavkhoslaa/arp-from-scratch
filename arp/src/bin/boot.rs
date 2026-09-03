use arp::utils::{build_arp_reply, get_ip, get_mac};
use std::mem::{size_of, zeroed};

fn main() {
    let my_mac = get_mac("eth0");
    let my_ip = get_ip("eth0");

    println!("Boot: my_ip={}, my_mac={:x?}", my_ip, my_mac);

    let fd: i32;
    unsafe {
        let proto = (libc::ETH_P_ARP as u16).to_be() as libc::c_int;
        fd = libc::socket(libc::AF_PACKET, libc::SOCK_RAW, proto);
    }

    let ifindex = unsafe { libc::if_nametoindex(b"eth0\0".as_ptr() as *const libc::c_char) };
    if ifindex == 0 {
        panic!("No interface index for eth0");
    }

    let broadcast_mac = [0xffu8; 6];

    // Gratuitous ARP: announce our own IP/MAC to the network
    let frame = build_arp_reply(my_mac, my_ip, &broadcast_mac, my_ip, 1);

    let mut sa: libc::sockaddr_ll = unsafe { zeroed() };
    sa.sll_family = libc::AF_PACKET as u16;
    sa.sll_protocol = (libc::ETH_P_ARP as u16).to_be();
    sa.sll_ifindex = ifindex as i32;
    sa.sll_halen = 6;
    sa.sll_addr[..6].copy_from_slice(&broadcast_mac);

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
        println!("Boot: gratuitous ARP sent");
    } else {
        eprintln!(
            "Boot: failed to send ARP (errno {})",
            std::io::Error::last_os_error()
        );
    }

    unsafe {
        libc::close(fd);
    }
}
