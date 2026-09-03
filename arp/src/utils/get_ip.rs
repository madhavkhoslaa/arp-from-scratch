use nix::ifaddrs::getifaddrs;
use std::net::Ipv4Addr;

pub fn get_ip(iface_name: &str) -> Ipv4Addr {
    getifaddrs()
        .unwrap()
        .filter_map(|iface| {
            if iface.interface_name == iface_name {
                iface
                    .address
                    .and_then(|addr| addr.as_sockaddr_in().map(|sa| Ipv4Addr::from(sa.ip())))
            } else {
                None
            }
        })
        .next()
        .unwrap_or_else(|| panic!("No IPv4 address found on {}", iface_name))
}
