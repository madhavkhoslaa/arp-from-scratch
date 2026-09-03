use std::net::Ipv4Addr;

pub fn build_arp_reply(
    my_mac: [u8; 6],
    my_ip: Ipv4Addr,
    requester_mac: &[u8],
    requester_ip: Ipv4Addr,
    opcode: u16,
) -> [u8; 42] {
    let mut f = [0u8; 42];
    f[0..6].copy_from_slice(requester_mac); // dest MAC = requester
    f[6..12].copy_from_slice(&my_mac); // src MAC = me
    f[12..14].copy_from_slice(&0x0806u16.to_be_bytes()); // EtherType ARP
    f[14..16].copy_from_slice(&1u16.to_be_bytes()); // htype Ethernet
    f[16..18].copy_from_slice(&0x0800u16.to_be_bytes()); // ptype IPv4
    f[18] = 6; // hlen (mac address is 6 bytes. IANA name + Unique ID)
    f[19] = 4; // plen (ipv4 is 4 octets/bytes)
    f[20..22].copy_from_slice(&opcode.to_be_bytes()); // opcode: 1=request, 2=reply
    f[22..28].copy_from_slice(&my_mac); // sender hw
    f[28..32].copy_from_slice(&my_ip.octets()); // sender proto
    f[32..38].copy_from_slice(requester_mac); // target hw
    f[38..42].copy_from_slice(&requester_ip.octets()); // target proto
    f
}
