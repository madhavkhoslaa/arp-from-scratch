use nix::ifaddrs::getifaddrs;

pub fn get_mac(iface_name: &str) -> [u8; 6] {
    getifaddrs()
        .unwrap()
        .filter_map(|iface| {
            if iface.interface_name == iface_name {
                iface
                    .address
                    .and_then(|addr| addr.as_link_addr().and_then(|la| la.addr()))
            } else {
                None
            }
        })
        .next()
        .unwrap_or_else(|| panic!("No MAC address found on {}", iface_name))
}
