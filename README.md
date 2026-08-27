# ARP from Scratch

# Topology
1 Switch
2. 3 Hosts

# What does a host have
1. IP
2. Mac Address(6 byte address)
3. It runs linux(has arp tables)



# Plan of action
* Disable the standard implmentation of ARP
* Run my own implementation(Binary that runs on userspace)
* Update the ARP tables

# Partiies involved
1. Host A(alice)
2. Host B(bob)
3. Host C (Carol)
4. Switch

# Links in out topology
1. Switch to A
2. Switch to B
3. Switch to C

# Responsibilites of our implementation
1. To be able to send out a ARP broadcast(ff:ff:ff:ff:ff:ff). What mac address has this IP
2.To be able to response to a ARP broadcast(Hey, me i.e my macaddress has this IP)
3. For a switch to be able to map interfaces with macaddr(mac table)

# What I've ignored
1. CAM memory and stuff
2. My switch does not have VLANs

# The docker spin up
1. Host A(alice)
2. Host B(bob)
3. Host C (Carol)
4. Switch
All of them are alpine linux. And I've added tcpdump binaries on them

A, B and C are isolated from each other. They can only communicate through the switch

# How we plan on reading the packets
1. AF_PACKET
2. RAW_SOCKET
3. ARP traffic
ARP should be wrapped in ethernet frame
# arp-from-scratch
