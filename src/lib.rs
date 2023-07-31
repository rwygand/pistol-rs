use anyhow::Result;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::time::Duration;
use subnetwork::Ipv4Pool;

mod scan;
mod utils;

/// ARP scanning.
/// This will sends ARP packets to hosts on the local network and displays any responses that are received.
/// The network interface to use can be specified with the `interface` option.
/// If this option is not present, program will search the system interface list for `subnet` user provided, configured up interface (excluding loopback).
/// By default, the ARP packets are sent to the Ethernet broadcast address, ff:ff:ff:ff:ff:ff, but that can be changed with the `destaddr` option.
/// When `threads_num` is 0, means that automatic threads pool mode is used.
pub fn arp_scan_subnet(
    subnet: &str,
    dstaddr: Option<&str>,
    interface: Option<&str>,
    threads_num: usize,
    print_result: bool,
) -> Result<scan::ArpScanResults> {
    let subnet = Ipv4Pool::new(subnet).unwrap();
    scan::run_arp_scan_subnet(subnet, dstaddr, interface, threads_num, print_result)
}

/// TCP connect() scanning.
/// This is the most basic form of TCP scanning.
/// The connect() system call provided by your operating system is used to open a connection to every interesting port on the machine.
/// If the port is listening, connect() will succeed, otherwise the port isn't reachable.
/// One strong advantage to this technique is that you don't need any special privileges.
/// Any user on most UNIX boxes is free to use this call.
/// Another advantage is speed.
/// While making a separate connect() call for every targeted port in a linear fashion would take ages over a slow connection,
/// you can hasten the scan by using many sockets in parallel.
/// Using non-blocking I/O allows you to set a low time-out period and watch all the sockets at once.
/// This is the fastest scanning method supported by nmap, and is available with the -t (TCP) option.
/// The big downside is that this sort of scan is easily detectable and filterable.
/// The target hosts logs will show a bunch of connection and error messages for the services which take the connection and then have it immediately shutdown.
pub fn tcp_connect_scan_single_port(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    dst_ipv4: Ipv4Addr,
    dst_port: u16,
    interface: Option<&str>,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<scan::TcpScanResults> {
    scan::run_tcp_connect_scan_single_port(
        src_ipv4,
        src_port,
        dst_ipv4,
        dst_port,
        interface,
        print_result,
        timeout,
        max_loop,
    )
}

pub fn tcp_connect_scan_range_port(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    dst_ipv4: Ipv4Addr,
    start_port: u16,
    end_port: u16,
    interface: Option<&str>,
    threads_num: usize,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<scan::TcpScanResults> {
    scan::run_tcp_connect_scan_range_port(
        src_ipv4,
        src_port,
        dst_ipv4,
        start_port,
        end_port,
        interface,
        threads_num,
        print_result,
        timeout,
        max_loop,
    )
}

pub fn tcp_connect_scan_subnet(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    subnet: Ipv4Pool,
    start_port: u16,
    end_port: u16,
    interface: Option<&str>,
    threads_num: usize,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<HashMap<Ipv4Addr, scan::TcpScanResults>> {
    scan::run_tcp_connect_scan_subnet(
        src_ipv4,
        src_port,
        subnet,
        start_port,
        end_port,
        interface,
        threads_num,
        print_result,
        timeout,
        max_loop,
    )
}

/// TCP SYN scanning.
/// This technique is often referred to as "half-open" scanning, because you don't open a full TCP connection.
/// You send a SYN packet, as if you are going to open a real connection and wait for a response.
/// A SYN|ACK indicates the port is listening.
/// A RST is indicative of a non-listener.
/// If a SYN|ACK is received, you immediately send a RST to tear down the connection (actually the kernel does this for us).
/// The primary advantage to this scanning technique is that fewer sites will log it.
/// Unfortunately you need root privileges to build these custom SYN packets.
/// SYN scan is the default and most popular scan option for good reason.
/// It can be performed quickly,
/// scanning thousands of ports per second on a fast network not hampered by intrusive firewalls.
/// SYN scan is relatively unobtrusive and stealthy, since it never completes TCP connections.
pub fn tcp_syn_scan_single_port(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    dst_ipv4: Ipv4Addr,
    dst_port: u16,
    interface: Option<&str>,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<scan::TcpScanResults> {
    scan::run_tcp_syn_scan_single_port(
        src_ipv4,
        src_port,
        dst_ipv4,
        dst_port,
        interface,
        print_result,
        timeout,
        max_loop,
    )
}

pub fn tcp_syn_scan_range_port(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    dst_ipv4: Ipv4Addr,
    start_port: u16,
    end_port: u16,
    interface: Option<&str>,
    threads_num: usize,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<scan::TcpScanResults> {
    scan::run_tcp_syn_scan_range_port(
        src_ipv4,
        src_port,
        dst_ipv4,
        start_port,
        end_port,
        interface,
        threads_num,
        print_result,
        timeout,
        max_loop,
    )
}

pub fn tcp_syn_scan_subnet(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    subnet: Ipv4Pool,
    start_port: u16,
    end_port: u16,
    interface: Option<&str>,
    threads_num: usize,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<HashMap<Ipv4Addr, scan::TcpScanResults>> {
    scan::run_tcp_syn_scan_subnet(
        src_ipv4,
        src_port,
        subnet,
        start_port,
        end_port,
        interface,
        threads_num,
        print_result,
        timeout,
        max_loop,
    )
}

/// TCP FIN scanning.
/// There are times when even SYN scanning isn't clandestine enough.
/// Some firewalls and packet filters watch for SYNs to an unallowed port,
/// and programs like synlogger and Courtney are available to detect these scans.
/// FIN packets, on the other hand, may be able to pass through unmolested.
/// This scanning technique was featured in detail by Uriel Maimon in Phrack 49, article 15.
/// The idea is that closed ports tend to reply to your FIN packet with the proper RST.
/// Open ports, on the other hand, tend to ignore the packet in question.
/// This is a bug in TCP implementations and so it isn't 100% reliable
/// (some systems, notably Micro$oft boxes, seem to be immune).
/// When scanning systems compliant with this RFC text,
/// any packet not containing SYN, RST, or ACK bits will result in a returned RST if the port is closed and no response at all if the port is open.
/// As long as none of those three bits are included, any combination of the other three (FIN, PSH, and URG) are OK.
pub fn tcp_fin_scan_single_port(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    dst_ipv4: Ipv4Addr,
    dst_port: u16,
    interface: Option<&str>,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<scan::TcpScanResults> {
    scan::run_tcp_fin_scan_single_port(
        src_ipv4,
        src_port,
        dst_ipv4,
        dst_port,
        interface,
        print_result,
        timeout,
        max_loop,
    )
}

pub fn tcp_fin_scan_range_port(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    dst_ipv4: Ipv4Addr,
    start_port: u16,
    end_port: u16,
    interface: Option<&str>,
    threads_num: usize,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<scan::TcpScanResults> {
    scan::run_tcp_fin_scan_range_port(
        src_ipv4,
        src_port,
        dst_ipv4,
        start_port,
        end_port,
        interface,
        threads_num,
        print_result,
        timeout,
        max_loop,
    )
}

pub fn tcp_fin_scan_subnet(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    subnet: Ipv4Pool,
    start_port: u16,
    end_port: u16,
    interface: Option<&str>,
    threads_num: usize,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<HashMap<Ipv4Addr, scan::TcpScanResults>> {
    scan::run_tcp_fin_scan_subnet(
        src_ipv4,
        src_port,
        subnet,
        start_port,
        end_port,
        interface,
        threads_num,
        print_result,
        timeout,
        max_loop,
    )
}

/// TCP ACK scanning.
/// This scan is different than the others discussed so far in that it never determines open (or even open|filtered) ports.
/// It is used to map out firewall rulesets, determining whether they are stateful or not and which ports are filtered.
/// When scanning unfiltered systems, open and closed ports will both return a RST packet.
/// We then labels them as unfiltered, meaning that they are reachable by the ACK packet, but whether they are open or closed is undetermined.
/// Ports that don't respond, or send certain ICMP error messages back, are labeled filtered.
pub fn tcp_ack_scan_single_port(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    dst_ipv4: Ipv4Addr,
    dst_port: u16,
    interface: Option<&str>,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<scan::TcpScanResults> {
    scan::run_tcp_ack_scan_single_port(
        src_ipv4,
        src_port,
        dst_ipv4,
        dst_port,
        interface,
        print_result,
        timeout,
        max_loop,
    )
}

pub fn tcp_ack_scan_range_port(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    dst_ipv4: Ipv4Addr,
    start_port: u16,
    end_port: u16,
    interface: Option<&str>,
    threads_num: usize,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<scan::TcpScanResults> {
    scan::run_tcp_ack_scan_range_port(
        src_ipv4,
        src_port,
        dst_ipv4,
        start_port,
        end_port,
        interface,
        threads_num,
        print_result,
        timeout,
        max_loop,
    )
}

pub fn tcp_ack_scan_subnet(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    subnet: Ipv4Pool,
    start_port: u16,
    end_port: u16,
    interface: Option<&str>,
    threads_num: usize,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<HashMap<Ipv4Addr, scan::TcpScanResults>> {
    scan::run_tcp_ack_scan_subnet(
        src_ipv4,
        src_port,
        subnet,
        start_port,
        end_port,
        interface,
        threads_num,
        print_result,
        timeout,
        max_loop,
    )
}

/// TCP Null scanning.
/// Does not set any bits (TCP flag header is 0).
/// When scanning systems compliant with this RFC text,
/// any packet not containing SYN, RST, or ACK bits will result in a returned RST if the port is closed and no response at all if the port is open.
/// As long as none of those three bits are included, any combination of the other three (FIN, PSH, and URG) are OK.
pub fn tcp_null_scan_single_port(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    dst_ipv4: Ipv4Addr,
    dst_port: u16,
    interface: Option<&str>,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<scan::TcpScanResults> {
    scan::run_tcp_null_scan_single_port(
        src_ipv4,
        src_port,
        dst_ipv4,
        dst_port,
        interface,
        print_result,
        timeout,
        max_loop,
    )
}

pub fn tcp_null_scan_range_port(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    dst_ipv4: Ipv4Addr,
    start_port: u16,
    end_port: u16,
    interface: Option<&str>,
    threads_num: usize,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<scan::TcpScanResults> {
    scan::run_tcp_null_scan_range_port(
        src_ipv4,
        src_port,
        dst_ipv4,
        start_port,
        end_port,
        interface,
        threads_num,
        print_result,
        timeout,
        max_loop,
    )
}

pub fn tcp_null_scan_subnet(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    subnet: Ipv4Pool,
    start_port: u16,
    end_port: u16,
    interface: Option<&str>,
    threads_num: usize,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<HashMap<Ipv4Addr, scan::TcpScanResults>> {
    scan::run_tcp_null_scan_subnet(
        src_ipv4,
        src_port,
        subnet,
        start_port,
        end_port,
        interface,
        threads_num,
        print_result,
        timeout,
        max_loop,
    )
}

/// TCP Xmas scanning.
/// Sets the FIN, PSH, and URG flags, lighting the packet up like a Christmas tree.
/// When scanning systems compliant with this RFC text,
/// any packet not containing SYN, RST, or ACK bits will result in a returned RST if the port is closed and no response at all if the port is open.
/// As long as none of those three bits are included, any combination of the other three (FIN, PSH, and URG) are OK.
pub fn tcp_xmas_scan_single_port(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    dst_ipv4: Ipv4Addr,
    dst_port: u16,
    interface: Option<&str>,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<scan::TcpScanResults> {
    scan::run_tcp_xmas_scan_single_port(
        src_ipv4,
        src_port,
        dst_ipv4,
        dst_port,
        interface,
        print_result,
        timeout,
        max_loop,
    )
}

pub fn tcp_xmas_scan_range_port(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    dst_ipv4: Ipv4Addr,
    start_port: u16,
    end_port: u16,
    interface: Option<&str>,
    threads_num: usize,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<scan::TcpScanResults> {
    scan::run_tcp_xmas_scan_range_port(
        src_ipv4,
        src_port,
        dst_ipv4,
        start_port,
        end_port,
        interface,
        threads_num,
        print_result,
        timeout,
        max_loop,
    )
}

pub fn tcp_xmas_scan_subnet(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    subnet: Ipv4Pool,
    start_port: u16,
    end_port: u16,
    interface: Option<&str>,
    threads_num: usize,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<HashMap<Ipv4Addr, scan::TcpScanResults>> {
    scan::run_tcp_xmas_scan_subnet(
        src_ipv4,
        src_port,
        subnet,
        start_port,
        end_port,
        interface,
        threads_num,
        print_result,
        timeout,
        max_loop,
    )
}

/// UDP scanning.
/// While most popular services on the Internet run over the TCP protocol, UDP services are widely deployed.
/// DNS, SNMP, and DHCP (registered ports 53, 161/162, and 67/68) are three of the most common.
/// Because UDP scanning is generally slower and more difficult than TCP, some security auditors ignore these ports.
/// This is a mistake, as exploitable UDP services are quite common and attackers certainly don't ignore the whole protocol.
/// UDP scan works by sending a UDP packet to every targeted port.
/// For most ports, this packet will be empty (no payload), but for a few of the more common ports a protocol-specific payload will be sent.
/// Based on the response, or lack thereof, the port is assigned to one of four states.
pub fn udp_scan_single_port(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    dst_ipv4: Ipv4Addr,
    dst_port: u16,
    interface: Option<&str>,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<scan::UdpScanResults> {
    scan::run_udp_scan_single_port(
        src_ipv4,
        src_port,
        dst_ipv4,
        dst_port,
        interface,
        print_result,
        timeout,
        max_loop,
    )
}

pub fn udp_scan_range_port(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    dst_ipv4: Ipv4Addr,
    start_port: u16,
    end_port: u16,
    interface: Option<&str>,
    threads_num: usize,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<scan::UdpScanResults> {
    scan::run_udp_scan_range_port(
        src_ipv4,
        src_port,
        dst_ipv4,
        start_port,
        end_port,
        interface,
        threads_num,
        print_result,
        timeout,
        max_loop,
    )
}

pub fn udp_scan_subnet(
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    subnet: Ipv4Pool,
    start_port: u16,
    end_port: u16,
    interface: Option<&str>,
    threads_num: usize,
    print_result: bool,
    timeout: Option<Duration>,
    max_loop: Option<usize>,
) -> Result<HashMap<Ipv4Addr, scan::UdpScanResults>> {
    scan::run_udp_scan_subnet(
        src_ipv4,
        src_port,
        subnet,
        start_port,
        end_port,
        interface,
        threads_num,
        print_result,
        timeout,
        max_loop,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_arp_scan_subnet() {
        // let interface = Some("ens33");
        let interface = None;
        let rets = arp_scan_subnet("192.168.1.0/24", None, interface, 0, true).unwrap();
        println!("{:?}", rets);
    }
    #[test]
    fn test_connect_scan_single_port() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let dst_ipv4: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 1);
        let dst_port: u16 = 80;
        // let interface: Option<&str> = Some("eno1");
        let interface: Option<&str> = None;
        let print_result: bool = true;
        let ret = tcp_connect_scan_single_port(
            src_ipv4,
            src_port,
            dst_ipv4,
            dst_port,
            interface,
            print_result,
            None,
            None,
        )
        .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_connect_scan_range_port() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let dst_ipv4: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 3);
        let start_port: u16 = 1;
        let end_port: u16 = 100;
        // let interface: Option<&str> = Some("eno1");
        let interface = None;
        let threads_num = 0;
        let print_result: bool = true;
        let max_loop = Some(8);
        let ret = tcp_connect_scan_range_port(
            src_ipv4,
            src_port,
            dst_ipv4,
            start_port,
            end_port,
            interface,
            threads_num,
            print_result,
            None,
            max_loop,
        )
        .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_connect_scan_subnet() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let subnet: Ipv4Pool = Ipv4Pool::new("192.168.1.0/24").unwrap();
        let start_port: u16 = 80;
        let end_port: u16 = 82;
        // let interface: Option<&str> = Some("eno1");
        let interface = None;
        let threads_num: usize = 0;
        let print_result: bool = true;
        let max_loop = Some(8);
        let ret = tcp_connect_scan_subnet(
            src_ipv4,
            src_port,
            subnet,
            start_port,
            end_port,
            interface,
            threads_num,
            print_result,
            None,
            max_loop,
        )
        .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_tcp_connect_scan_single_port() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let dst_ipv4: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 1);
        let dst_port: u16 = 80;
        // let interface: Option<&str> = Some("eno1");
        let interface = None;
        let print_result: bool = true;
        let max_loop = Some(64);
        let ret = tcp_connect_scan_single_port(
            src_ipv4,
            src_port,
            dst_ipv4,
            dst_port,
            interface,
            print_result,
            None,
            max_loop,
        )
        .unwrap();
        println!("{:?}", ret);
        let dst_port: u16 = 88;
        let ret = tcp_connect_scan_single_port(
            src_ipv4,
            src_port,
            dst_ipv4,
            dst_port,
            interface,
            print_result,
            None,
            max_loop,
        )
        .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_syn_scan_single_port() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let dst_ipv4 = Ipv4Addr::new(192, 168, 1, 1);
        // let i = Some("eno1");
        let i = None;
        let max_loop = Some(64);
        let ret =
            tcp_syn_scan_single_port(src_ipv4, src_port, dst_ipv4, 80, i, true, None, max_loop)
                .unwrap();
        println!("{:?}", ret);
        let ret =
            tcp_syn_scan_single_port(src_ipv4, src_port, dst_ipv4, 9999, i, true, None, max_loop)
                .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_syn_scan_range_port() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let dst_ipv4 = Ipv4Addr::new(192, 168, 1, 1);
        // let i = Some("eno1");
        let i = None;
        let ret =
            tcp_syn_scan_range_port(src_ipv4, src_port, dst_ipv4, 22, 90, i, 0, true, None, None)
                .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_syn_scan_subnet() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let subnet = Ipv4Pool::new("192.168.1.0/24").unwrap();
        // let i = Some("eno1");
        let i = None;
        let max_loop = Some(64);
        let ret = tcp_syn_scan_subnet(
            src_ipv4, src_port, subnet, 80, 82, i, 0, true, None, max_loop,
        )
        .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_fin_scan_single_port() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let dst_ipv4 = Ipv4Addr::new(192, 168, 1, 1);
        // let i = Some("eno1");
        let i = None;
        let ret = tcp_fin_scan_single_port(src_ipv4, src_port, dst_ipv4, 80, i, true, None, None)
            .unwrap();
        println!("{:?}", ret);
        let ret = tcp_fin_scan_single_port(src_ipv4, src_port, dst_ipv4, 9999, i, true, None, None)
            .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_fin_scan_range_port() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let dst_ipv4 = Ipv4Addr::new(192, 168, 1, 1);
        // let i = Some("eno1");
        let i = None;
        let ret =
            tcp_fin_scan_range_port(src_ipv4, src_port, dst_ipv4, 22, 90, i, 0, true, None, None)
                .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_fin_scan_subnet() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let subnet = Ipv4Pool::new("192.168.1.0/24").unwrap();
        // let i = Some("eno1");
        let i = None;
        let max_loop = Some(64);
        let ret = tcp_fin_scan_subnet(
            src_ipv4, src_port, subnet, 80, 82, i, 0, true, None, max_loop,
        )
        .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_ack_scan_single_port() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let dst_ipv4 = Ipv4Addr::new(192, 168, 1, 1);
        // let i = Some("eno1");
        let i = None;
        let ret = tcp_ack_scan_single_port(src_ipv4, src_port, dst_ipv4, 80, i, true, None, None)
            .unwrap();
        println!("{:?}", ret);
        let ret = tcp_ack_scan_single_port(src_ipv4, src_port, dst_ipv4, 9999, i, true, None, None)
            .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_ack_scan_range_port() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let dst_ipv4 = Ipv4Addr::new(192, 168, 1, 1);
        // let i = Some("eno1");
        let i = None;
        let ret =
            tcp_ack_scan_range_port(src_ipv4, src_port, dst_ipv4, 22, 90, i, 0, true, None, None)
                .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_ack_scan_subnet() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let subnet = Ipv4Pool::new("192.168.1.0/24").unwrap();
        // let i = Some("eno1");
        let i = None;
        let max_loop = Some(64);
        let ret = tcp_ack_scan_subnet(
            src_ipv4, src_port, subnet, 80, 82, i, 0, true, None, max_loop,
        )
        .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_null_scan_single_port() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let dst_ipv4 = Ipv4Addr::new(192, 168, 1, 1);
        // let i = Some("eno1");
        let i = None;
        let ret = tcp_null_scan_single_port(src_ipv4, src_port, dst_ipv4, 81, i, true, None, None)
            .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_null_scan_range_port() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let dst_ipv4 = Ipv4Addr::new(192, 168, 1, 1);
        // let i = Some("eno1");
        let i = None;
        let ret =
            tcp_null_scan_range_port(src_ipv4, src_port, dst_ipv4, 22, 90, i, 0, true, None, None)
                .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_null_scan_subnet() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let subnet = Ipv4Pool::new("192.168.1.0/24").unwrap();
        // let i = Some("eno1");
        let i = None;
        let max_loop = Some(64);
        let ret = tcp_null_scan_subnet(
            src_ipv4, src_port, subnet, 80, 82, i, 0, true, None, max_loop,
        )
        .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_udp_scan_single_port() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let dst_ipv4 = Ipv4Addr::new(192, 168, 1, 1);
        // let i = Some("eno1");
        let i = None;
        let ret =
            udp_scan_single_port(src_ipv4, src_port, dst_ipv4, 53, i, true, None, None).unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_udp_scan_range_port() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let dst_ipv4 = Ipv4Addr::new(192, 168, 1, 1);
        // let i = Some("eno1");
        let i = None;
        let ret = udp_scan_range_port(src_ipv4, src_port, dst_ipv4, 22, 90, i, 0, true, None, None)
            .unwrap();
        println!("{:?}", ret);
    }
    #[test]
    fn test_udp_scan_subnet() {
        let src_ipv4 = Some(Ipv4Addr::new(192, 168, 1, 211));
        let src_port = None;
        let subnet = Ipv4Pool::new("192.168.1.0/24").unwrap();
        // let i = Some("eno1");
        let i = None;
        let max_loop = Some(64);
        let ret = udp_scan_subnet(
            src_ipv4, src_port, subnet, 80, 82, i, 0, true, None, max_loop,
        )
        .unwrap();
        println!("{:?}", ret);
    }
}
