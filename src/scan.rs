/* Scan */
use anyhow::Result;
use pnet::datalink::MacAddr;
use pnet::packet::ip::IpNextHeaderProtocol;
use std::collections::HashMap;
use std::fmt;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::net::IpAddr;

pub mod arp;
pub mod ip;
pub mod tcp;
pub mod tcp6;
pub mod udp;
pub mod udp6;

use crate::errors::CanNotFoundInterface;
use crate::errors::CanNotFoundMacAddress;
use crate::errors::CanNotFoundSourceAddress;
use crate::utils::find_interface_by_ipv4;
use crate::utils::find_source_ipv4;
use crate::utils::find_source_ipv6;
use crate::utils::get_default_timeout;
use crate::utils::get_threads_pool;
use crate::utils::random_port;
use crate::TargetType;

use super::errors::NotSupportIpTypeForArpScan;
use super::Target;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetScanStatus {
    Open,
    Closed,
    Filtered,
    OpenOrFiltered,
    Unfiltered,
    Unreachable,
    ClosedOrFiltered,
}

#[derive(Debug, Clone, Copy)]
pub struct IdleScanResults {
    pub zombie_ip_id_1: u16,
    pub zombie_ip_id_2: u16,
}

#[derive(Debug, Clone)]
pub struct ArpAliveHosts {
    pub mac_addr: MacAddr,
    pub ouis: String,
}

#[derive(Debug, Clone)]
pub struct ArpScanResults {
    pub alive_hosts: HashMap<Ipv4Addr, ArpAliveHosts>,
}

impl fmt::Display for ArpScanResults {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result_str = String::new();
        let s = format!("Alive hosts: {}", self.alive_hosts.len());
        result_str += &s;
        result_str += "\n";
        for (ip, aah) in &self.alive_hosts {
            let s = format!("{}: {} ({})", ip, aah.mac_addr, aah.ouis);
            result_str += &s;
            result_str += "\n";
        }
        write!(f, "{}", result_str)
    }
}

#[derive(Debug, Clone)]
pub struct PortStatus {
    pub status: HashMap<u16, TargetScanStatus>,
    pub rtt: Option<Duration>,
}

impl PortStatus {
    pub fn new() -> PortStatus {
        PortStatus {
            status: HashMap::new(),
            rtt: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TcpUdpScanResults {
    pub results: HashMap<IpAddr, PortStatus>,
}

impl TcpUdpScanResults {
    pub fn new() -> TcpUdpScanResults {
        TcpUdpScanResults {
            results: HashMap::new(),
        }
    }
}

impl fmt::Display for TcpUdpScanResults {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result_str = String::new();
        for (ip, ps) in &self.results {
            for (port, status) in &ps.status {
                let status_str = match status {
                    TargetScanStatus::Open => format!("{ip} {port} open"),
                    TargetScanStatus::OpenOrFiltered => format!("{ip} {port} open|filtered"),
                    TargetScanStatus::Filtered => format!("{ip} {port} filtered"),
                    TargetScanStatus::Unfiltered => format!("{ip} {port} unfiltered"),
                    TargetScanStatus::Closed => format!("{ip} {port} closed"),
                    TargetScanStatus::Unreachable => format!("{ip} {port} unreachable"),
                    TargetScanStatus::ClosedOrFiltered => format!("{ip} {port} closed|filtered"),
                };
                result_str += &status_str;
                result_str += "\n";
            }
        }
        write!(f, "{}", result_str)
    }
}

#[derive(Debug, Clone)]
pub struct ProtocolStatus {
    pub status: HashMap<IpNextHeaderProtocol, TargetScanStatus>,
    pub rtt: Option<Duration>,
}

impl ProtocolStatus {
    pub fn new() -> ProtocolStatus {
        ProtocolStatus {
            status: HashMap::new(),
            rtt: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IpScanResults {
    pub results: HashMap<IpAddr, ProtocolStatus>,
}

impl IpScanResults {
    pub fn new() -> IpScanResults {
        IpScanResults {
            results: HashMap::new(),
        }
    }
}

impl fmt::Display for IpScanResults {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result_str = String::new();
        for (ip, ps) in &self.results {
            for (protocol, status) in &ps.status {
                let status_str = match status {
                    TargetScanStatus::Open => format!("{ip} {protocol} open"),
                    TargetScanStatus::OpenOrFiltered => format!("{ip} {protocol} open|filtered"),
                    TargetScanStatus::Filtered => format!("{ip} {protocol} filtered"),
                    TargetScanStatus::Unfiltered => format!("{ip} {protocol} unfiltered"),
                    TargetScanStatus::Closed => format!("{ip} {protocol} closed"),
                    TargetScanStatus::Unreachable => format!("{ip} {protocol} unreachable"),
                    TargetScanStatus::ClosedOrFiltered => {
                        format!("{ip} {protocol} closed|filtered")
                    }
                };
                result_str += &status_str;
                result_str += "\n";
            }
        }
        write!(f, "{}", result_str)
    }
}

#[derive(Debug, Clone)]
pub struct NmapMacPrefix {
    pub prefix: String,
    pub ouis: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScanMethod {
    Connect,
    Syn,
    Fin,
    Ack,
    Null,
    Xmas,
    Window,
    Maimon,
    Idle, // need ipv4 ip id
    Udp,
    IpProcotol,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScanMethod6 {
    Connect,
    Syn,
    Fin,
    Ack,
    Null,
    Xmas,
    Window,
    Maimon,
    Udp,
}

fn get_nmap_mac_prefixes() -> Vec<NmapMacPrefix> {
    let nmap_mac_prefixes_file = include_str!("./db/nmap-mac-prefixes");
    let mut nmap_mac_prefixes = Vec::new();
    for l in nmap_mac_prefixes_file.lines() {
        nmap_mac_prefixes.push(l.to_string());
    }

    let mut ret = Vec::new();
    for p in nmap_mac_prefixes {
        if !p.contains("#") {
            let p_split: Vec<String> = p.split(" ").map(|s| s.to_string()).collect();
            if p_split.len() >= 2 {
                let ouis_slice = p_split[1..].to_vec();
                let n = NmapMacPrefix {
                    prefix: p_split[0].to_string(),
                    ouis: ouis_slice.join(" "),
                };
                ret.push(n);
            }
        }
    }
    ret
}

pub fn arp_scan(
    target: Target,
    src_ipv4: Option<Ipv4Addr>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<ArpScanResults> {
    match target.target_type {
        TargetType::Ipv4 => {
            let nmap_mac_prefixes = get_nmap_mac_prefixes();
            let mut ret = ArpScanResults {
                alive_hosts: HashMap::new(),
            };

            // println!("{:?}", bi_vec);
            let pool = get_threads_pool(threads_num);
            let timeout = match timeout {
                Some(t) => t,
                None => get_default_timeout(),
            };

            let dst_mac = MacAddr::broadcast();
            let (tx, rx) = channel();
            let mut recv_size = 0;
            for host in target.hosts {
                recv_size += 1;
                let dst_ipv4 = host.addr;
                let src_ipv4 = match find_source_ipv4(src_ipv4, dst_ipv4)? {
                    Some(s) => s,
                    None => return Err(CanNotFoundSourceAddress::new().into()),
                };
                let interface = match find_interface_by_ipv4(src_ipv4) {
                    Some(i) => i,
                    None => return Err(CanNotFoundInterface::new().into()),
                };
                let src_mac = match interface.mac {
                    Some(m) => m,
                    None => return Err(CanNotFoundMacAddress::new().into()),
                };
                let tx = tx.clone();
                pool.execute(move || {
                    let scan_ret = arp::send_arp_scan_packet(
                        dst_ipv4, dst_mac, src_ipv4, src_mac, interface, timeout,
                    );
                    match tx.send(Ok((dst_ipv4, scan_ret))) {
                        _ => (),
                    }
                });
            }
            let iter = rx.into_iter().take(recv_size);
            for v in iter {
                match v {
                    Ok((target_ipv4, target_mac)) => match target_mac? {
                        (Some(m), Some(_rtt)) => {
                            let mut ouis = String::new();
                            let mut mac_prefix = String::new();
                            let m0 = format!("{:X}", m.0);
                            let m1 = format!("{:X}", m.1);
                            let m2 = format!("{:X}", m.2);
                            // m0
                            let i = if m0.len() < 2 { 2 - m0.len() } else { 0 };
                            if i > 0 {
                                for _ in 0..i {
                                    mac_prefix += "0";
                                }
                            }
                            mac_prefix += &m0;
                            // m1
                            let i = if m1.len() < 2 { 2 - m1.len() } else { 0 };
                            if i > 0 {
                                for _ in 0..i {
                                    mac_prefix += "0";
                                }
                            }
                            mac_prefix += &m1;
                            // m2
                            let i = if m2.len() < 2 { 2 - m2.len() } else { 0 };
                            if i > 0 {
                                for _ in 0..i {
                                    mac_prefix += "0";
                                }
                            }
                            mac_prefix += &m2;
                            // println!("{}", mac_prefix);
                            for p in &nmap_mac_prefixes {
                                if mac_prefix == p.prefix {
                                    ouis = p.ouis.to_string();
                                }
                            }
                            let aah = ArpAliveHosts { mac_addr: m, ouis };
                            ret.alive_hosts.insert(target_ipv4, aah);
                        }
                        (_, _) => (),
                    },
                    Err(e) => return Err(e),
                }
            }
            Ok(ret)
        }
        _ => Err(NotSupportIpTypeForArpScan::new(target.target_type).into()),
    }
}

fn run_scan(
    method: ScanMethod,
    src_ipv4: Ipv4Addr,
    src_port: u16,
    dst_ipv4: Ipv4Addr,
    dst_port: u16,
    zombie_ipv4: Option<Ipv4Addr>,
    zombie_port: Option<u16>,
    protocol: Option<IpNextHeaderProtocol>,
    timeout: Duration,
) -> Result<(
    Ipv4Addr,
    u16,
    Option<IpNextHeaderProtocol>,
    TargetScanStatus,
    Option<Duration>,
)> {
    let (scan_ret, rtt) = match method {
        ScanMethod::Connect => {
            tcp::send_connect_scan_packet(src_ipv4, src_port, dst_ipv4, dst_port, timeout)?
        }
        ScanMethod::Syn => {
            tcp::send_syn_scan_packet(src_ipv4, src_port, dst_ipv4, dst_port, timeout)?
        }
        ScanMethod::Fin => {
            tcp::send_fin_scan_packet(src_ipv4, src_port, dst_ipv4, dst_port, timeout)?
        }
        ScanMethod::Ack => {
            tcp::send_ack_scan_packet(src_ipv4, src_port, dst_ipv4, dst_port, timeout)?
        }
        ScanMethod::Null => {
            tcp::send_null_scan_packet(src_ipv4, src_port, dst_ipv4, dst_port, timeout)?
        }
        ScanMethod::Xmas => {
            tcp::send_xmas_scan_packet(src_ipv4, src_port, dst_ipv4, dst_port, timeout)?
        }
        ScanMethod::Window => {
            tcp::send_window_scan_packet(src_ipv4, src_port, dst_ipv4, dst_port, timeout)?
        }
        ScanMethod::Maimon => {
            tcp::send_maimon_scan_packet(src_ipv4, src_port, dst_ipv4, dst_port, timeout)?
        }
        ScanMethod::Idle => {
            let zombie_ipv4 = zombie_ipv4.unwrap();
            let zombie_port = zombie_port.unwrap();
            match tcp::send_idle_scan_packet(
                src_ipv4,
                src_port,
                dst_ipv4,
                dst_port,
                zombie_ipv4,
                zombie_port,
                timeout,
            ) {
                Ok((status, _idel_rets, rtt)) => (status, rtt),
                Err(e) => return Err(e.into()),
            }
        }
        ScanMethod::Udp => {
            udp::send_udp_scan_packet(src_ipv4, src_port, dst_ipv4, dst_port, timeout)?
        }
        ScanMethod::IpProcotol => {
            ip::send_ip_procotol_scan_packet(src_ipv4, dst_ipv4, protocol.unwrap(), timeout)?
        }
    };

    Ok((dst_ipv4, dst_port, protocol, scan_ret, rtt))
}

fn run_scan6(
    method: ScanMethod6,
    src_ipv6: Ipv6Addr,
    src_port: u16,
    dst_ipv6: Ipv6Addr,
    dst_port: u16,
    timeout: Duration,
) -> Result<(Ipv6Addr, u16, TargetScanStatus, Option<Duration>)> {
    let (scan_ret, rtt) = match method {
        ScanMethod6::Connect => {
            tcp6::send_connect_scan_packet(src_ipv6, src_port, dst_ipv6, dst_port, timeout)?
        }
        ScanMethod6::Syn => {
            tcp6::send_syn_scan_packet(src_ipv6, src_port, dst_ipv6, dst_port, timeout)?
        }
        ScanMethod6::Fin => {
            tcp6::send_fin_scan_packet(src_ipv6, src_port, dst_ipv6, dst_port, timeout)?
        }
        ScanMethod6::Ack => {
            tcp6::send_ack_scan_packet(src_ipv6, src_port, dst_ipv6, dst_port, timeout)?
        }
        ScanMethod6::Null => {
            tcp6::send_null_scan_packet(src_ipv6, src_port, dst_ipv6, dst_port, timeout)?
        }
        ScanMethod6::Xmas => {
            tcp6::send_xmas_scan_packet(src_ipv6, src_port, dst_ipv6, dst_port, timeout)?
        }
        ScanMethod6::Window => {
            tcp6::send_window_scan_packet(src_ipv6, src_port, dst_ipv6, dst_port, timeout)?
        }
        ScanMethod6::Maimon => {
            tcp6::send_maimon_scan_packet(src_ipv6, src_port, dst_ipv6, dst_port, timeout)?
        }
        ScanMethod6::Udp => {
            udp6::send_udp_scan_packet(src_ipv6, src_port, dst_ipv6, dst_port, timeout)?
        }
    };

    Ok((dst_ipv6, dst_port, scan_ret, rtt))
}

pub fn scan(
    target: Target,
    method: ScanMethod,
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    zombie_ipv4: Option<Ipv4Addr>,
    zombie_port: Option<u16>,
    protocol: Option<IpNextHeaderProtocol>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<(TcpUdpScanResults, IpScanResults)> {
    let pool = get_threads_pool(threads_num);
    let (tx, rx) = channel();
    let mut recv_size = 0;
    let timeout = match timeout {
        Some(t) => t,
        None => get_default_timeout(),
    };

    for host in target.hosts {
        let dst_ipv4 = host.addr;
        let src_ipv4 = match find_source_ipv4(src_ipv4, dst_ipv4)? {
            Some(s) => s,
            None => return Err(CanNotFoundSourceAddress::new().into()),
        };
        let src_port = match src_port {
            Some(s) => s,
            None => random_port(),
        };
        for dst_port in host.ports {
            let tx = tx.clone();
            recv_size += 1;
            pool.execute(move || {
                let scan_ret = run_scan(
                    method,
                    src_ipv4,
                    src_port,
                    dst_ipv4,
                    dst_port,
                    zombie_ipv4,
                    zombie_port,
                    protocol,
                    timeout,
                );
                match tx.send(scan_ret) {
                    _ => (),
                }
            });
        }
    }

    let iter = rx.into_iter().take(recv_size);
    let mut tcpudp_ret = TcpUdpScanResults::new();
    let mut ip_ret = IpScanResults::new();

    for v in iter {
        match v {
            Ok((dst_ipv4, dst_port, procotol, scan_ret, rtt)) => match procotol {
                Some(p) => {
                    if ip_ret.results.contains_key(&dst_ipv4.into()) {
                        let ps = ip_ret.results.get_mut(&dst_ipv4.into()).unwrap();
                        ps.status.insert(p, scan_ret);
                        match ps.rtt {
                            Some(_) => (),
                            None => ps.rtt = rtt,
                        }
                    } else {
                        let mut ps = ProtocolStatus::new();
                        ps.status.insert(p, scan_ret);
                        match ps.rtt {
                            Some(_) => (),
                            None => ps.rtt = rtt,
                        }
                        ip_ret.results.insert(dst_ipv4.into(), ps);
                    }
                }
                _ => {
                    if tcpudp_ret.results.contains_key(&dst_ipv4.into()) {
                        let ps = tcpudp_ret.results.get_mut(&dst_ipv4.into()).unwrap();
                        ps.status.insert(dst_port, scan_ret);
                        match ps.rtt {
                            Some(_) => (),
                            None => ps.rtt = rtt,
                        }
                    } else {
                        let mut ps = PortStatus::new();
                        ps.status.insert(dst_port, scan_ret);
                        match ps.rtt {
                            Some(_) => (),
                            None => ps.rtt = rtt,
                        }
                        tcpudp_ret.results.insert(dst_ipv4.into(), ps);
                    }
                }
            },
            Err(e) => return Err(e),
        }
    }
    Ok((tcpudp_ret, ip_ret))
}

pub fn scan6(
    target: Target,
    method: ScanMethod6,
    src_ipv6: Option<Ipv6Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    let pool = get_threads_pool(threads_num);
    let (tx, rx) = channel();
    let mut recv_size = 0;
    let timeout = match timeout {
        Some(t) => t,
        None => get_default_timeout(),
    };

    for host in target.hosts6 {
        let dst_ipv6 = host.addr;
        let src_ipv6 = match find_source_ipv6(src_ipv6, dst_ipv6)? {
            Some(s) => s,
            None => return Err(CanNotFoundSourceAddress::new().into()),
        };
        let src_port = match src_port {
            Some(s) => s,
            None => random_port(),
        };
        for dst_port in host.ports {
            let tx = tx.clone();
            recv_size += 1;
            pool.execute(move || {
                let scan_ret = run_scan6(method, src_ipv6, src_port, dst_ipv6, dst_port, timeout);
                match tx.send(scan_ret) {
                    _ => (),
                }
            });
        }
    }

    let iter = rx.into_iter().take(recv_size);
    let mut tcpudp_ret = TcpUdpScanResults::new();

    for v in iter {
        match v {
            Ok((dst_ipv6, dst_port, scan_ret, rtt)) => {
                if tcpudp_ret.results.contains_key(&dst_ipv6.into()) {
                    let ps = tcpudp_ret.results.get_mut(&dst_ipv6.into()).unwrap();
                    ps.status.insert(dst_port, scan_ret);
                    match ps.rtt {
                        Some(_) => (),
                        None => ps.rtt = rtt,
                    }
                } else {
                    let mut ps = PortStatus::new();
                    ps.status.insert(dst_port, scan_ret);
                    match ps.rtt {
                        Some(_) => (),
                        None => ps.rtt = rtt,
                    }
                    tcpudp_ret.results.insert(dst_ipv6.into(), ps);
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(tcpudp_ret)
}

pub fn tcp_connect_scan(
    target: Target,
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    let (ret, _) = scan(
        target,
        ScanMethod::Connect,
        src_ipv4,
        src_port,
        None,
        None,
        None,
        threads_num,
        timeout,
    )?;
    Ok(ret)
}

pub fn tcp_connect_scan6(
    target: Target,
    src_ipv6: Option<Ipv6Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    scan6(
        target,
        ScanMethod6::Connect,
        src_ipv6,
        src_port,
        threads_num,
        timeout,
    )
}

pub fn tcp_syn_scan(
    target: Target,
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    let (ret, _) = scan(
        target,
        ScanMethod::Syn,
        src_ipv4,
        src_port,
        None,
        None,
        None,
        threads_num,
        timeout,
    )?;
    Ok(ret)
}

pub fn tcp_syn_scan6(
    target: Target,
    src_ipv6: Option<Ipv6Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    scan6(
        target,
        ScanMethod6::Syn,
        src_ipv6,
        src_port,
        threads_num,
        timeout,
    )
}

pub fn tcp_fin_scan(
    target: Target,
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    let (ret, _) = scan(
        target,
        ScanMethod::Fin,
        src_ipv4,
        src_port,
        None,
        None,
        None,
        threads_num,
        timeout,
    )?;
    Ok(ret)
}

pub fn tcp_fin_scan6(
    target: Target,
    src_ipv6: Option<Ipv6Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    scan6(
        target,
        ScanMethod6::Fin,
        src_ipv6,
        src_port,
        threads_num,
        timeout,
    )
}

pub fn tcp_ack_scan(
    target: Target,
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    let (ret, _) = scan(
        target,
        ScanMethod::Ack,
        src_ipv4,
        src_port,
        None,
        None,
        None,
        threads_num,
        timeout,
    )?;
    Ok(ret)
}

pub fn tcp_ack_scan6(
    target: Target,
    src_ipv6: Option<Ipv6Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    scan6(
        target,
        ScanMethod6::Ack,
        src_ipv6,
        src_port,
        threads_num,
        timeout,
    )
}

pub fn tcp_null_scan(
    target: Target,
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    let (ret, _) = scan(
        target,
        ScanMethod::Null,
        src_ipv4,
        src_port,
        None,
        None,
        None,
        threads_num,
        timeout,
    )?;
    Ok(ret)
}

pub fn tcp_null_scan6(
    target: Target,
    src_ipv6: Option<Ipv6Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    scan6(
        target,
        ScanMethod6::Null,
        src_ipv6,
        src_port,
        threads_num,
        timeout,
    )
}

pub fn tcp_xmas_scan(
    target: Target,
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    let (ret, _) = scan(
        target,
        ScanMethod::Xmas,
        src_ipv4,
        src_port,
        None,
        None,
        None,
        threads_num,
        timeout,
    )?;
    Ok(ret)
}

pub fn tcp_xmas_scan6(
    target: Target,
    src_ipv6: Option<Ipv6Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    scan6(
        target,
        ScanMethod6::Xmas,
        src_ipv6,
        src_port,
        threads_num,
        timeout,
    )
}

pub fn tcp_window_scan(
    target: Target,
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    let (ret, _) = scan(
        target,
        ScanMethod::Window,
        src_ipv4,
        src_port,
        None,
        None,
        None,
        threads_num,
        timeout,
    )?;
    Ok(ret)
}

pub fn tcp_window_scan6(
    target: Target,
    src_ipv6: Option<Ipv6Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    scan6(
        target,
        ScanMethod6::Window,
        src_ipv6,
        src_port,
        threads_num,
        timeout,
    )
}

pub fn tcp_maimon_scan(
    target: Target,
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    let (ret, _) = scan(
        target,
        ScanMethod::Maimon,
        src_ipv4,
        src_port,
        None,
        None,
        None,
        threads_num,
        timeout,
    )?;
    Ok(ret)
}

pub fn tcp_maimon_scan6(
    target: Target,
    src_ipv6: Option<Ipv6Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    scan6(
        target,
        ScanMethod6::Maimon,
        src_ipv6,
        src_port,
        threads_num,
        timeout,
    )
}

pub fn tcp_idle_scan(
    target: Target,
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    zombie_ipv4: Option<Ipv4Addr>,
    zombie_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    let (ret, _) = scan(
        target,
        ScanMethod::Idle,
        src_ipv4,
        src_port,
        zombie_ipv4,
        zombie_port,
        None,
        threads_num,
        timeout,
    )?;
    Ok(ret)
}

pub fn udp_scan(
    target: Target,
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    let (ret, _) = scan(
        target,
        ScanMethod::Udp,
        src_ipv4,
        src_port,
        None,
        None,
        None,
        threads_num,
        timeout,
    )?;
    Ok(ret)
}

pub fn udp_scan6(
    target: Target,
    src_ipv6: Option<Ipv6Addr>,
    src_port: Option<u16>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<TcpUdpScanResults> {
    scan6(
        target,
        ScanMethod6::Udp,
        src_ipv6,
        src_port,
        threads_num,
        timeout,
    )
}

pub fn ip_procotol_scan(
    target: Target,
    src_ipv4: Option<Ipv4Addr>,
    src_port: Option<u16>,
    protocol: Option<IpNextHeaderProtocol>,
    threads_num: usize,
    timeout: Option<Duration>,
) -> Result<IpScanResults> {
    let (_, ret) = scan(
        target,
        ScanMethod::IpProcotol,
        src_ipv4,
        src_port,
        None,
        None,
        protocol,
        threads_num,
        timeout,
    )?;
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Host, Target};
    use subnetwork::Ipv4Pool;
    #[test]
    fn test_arp_scan_subnet() -> Result<()> {
        let subnet: Ipv4Pool = Ipv4Pool::from("192.168.1.0/24").unwrap();
        let mut hosts: Vec<Host> = vec![];
        for ip in subnet {
            let host = Host::new(ip, None)?;
            hosts.push(host);
        }
        let target: Target = Target::new(hosts);
        let threads_num = 300;
        let timeout = Some(Duration::new(1, 5));
        // let print_result = false;
        let src_ipv4 = None;
        let ret: ArpScanResults = arp_scan(target, src_ipv4, threads_num, timeout).unwrap();
        println!("{}", ret);
        Ok(())
    }
    #[test]
    fn test_arp_scan_subnet_new() -> Result<()> {
        let target: Target = Target::from_subnet("192.168.59.130/24", None)?;
        let threads_num = 300;
        let timeout = Some(Duration::new(1, 5));
        // let print_result = false;
        let src_ipv4 = None;
        let ret: ArpScanResults = arp_scan(target, src_ipv4, threads_num, timeout).unwrap();
        println!("{}", ret);
        Ok(())
    }
    #[test]
    fn test_tcp_connect_scan() -> Result<()> {
        let src_ipv4 = None;
        let src_port = None;
        let dst_ipv4 = Ipv4Addr::new(192, 168, 31, 2);
        let threads_num: usize = 8;
        let timeout = Some(Duration::new(3, 0));
        let host = Host::new(dst_ipv4, Some(vec![22, 99]))?;
        let target: Target = Target::new(vec![host]);
        let ret = tcp_connect_scan(target, src_ipv4, src_port, threads_num, timeout).unwrap();
        println!("{}", ret);
        Ok(())
    }
    #[test]
    fn test_tcp_syn_scan() -> Result<()> {
        let src_ipv4 = None;
        let src_port = None;
        let dst_ipv4: Ipv4Addr = Ipv4Addr::new(192, 168, 72, 128);
        let threads_num: usize = 8;
        let timeout = Some(Duration::new(3, 0));
        let host = Host::new(dst_ipv4, Some(vec![22]))?;
        let target: Target = Target::new(vec![host]);
        let ret = tcp_syn_scan(target, src_ipv4, src_port, threads_num, timeout).unwrap();
        println!("{}", ret);
        Ok(())
    }
    #[test]
    fn test_tcp_fin_scan() -> Result<()> {
        let src_ipv4: Option<Ipv4Addr> = Some(Ipv4Addr::new(192, 168, 72, 128));
        let src_port: Option<u16> = None;
        let dst_ipv4: Ipv4Addr = Ipv4Addr::new(192, 168, 72, 135);
        let threads_num: usize = 8;
        let timeout = Some(Duration::new(3, 0));
        let host = Host::new(dst_ipv4, Some(vec![22, 99]))?;
        let target: Target = Target::new(vec![host]);
        let ret = tcp_fin_scan(target, src_ipv4, src_port, threads_num, timeout).unwrap();
        println!("{}", ret);
        Ok(())
    }
    #[test]
    fn test_tcp_ack_scan() -> Result<()> {
        let src_ipv4: Option<Ipv4Addr> = Some(Ipv4Addr::new(192, 168, 72, 128));
        let src_port: Option<u16> = None;
        let dst_ipv4: Ipv4Addr = Ipv4Addr::new(192, 168, 72, 135);
        let threads_num: usize = 8;
        let timeout = Some(Duration::new(3, 0));
        let host = Host::new(dst_ipv4, Some(vec![22, 99]))?;
        let target: Target = Target::new(vec![host]);
        let ret = tcp_ack_scan(target, src_ipv4, src_port, threads_num, timeout).unwrap();
        println!("{}", ret);
        Ok(())
    }
    #[test]
    fn test_tcp_null_scan() -> Result<()> {
        let src_ipv4: Option<Ipv4Addr> = Some(Ipv4Addr::new(192, 168, 72, 128));
        let src_port: Option<u16> = None;
        let dst_ipv4: Ipv4Addr = Ipv4Addr::new(192, 168, 72, 135);
        let threads_num: usize = 8;
        let timeout = Some(Duration::new(3, 0));
        let host = Host::new(dst_ipv4, Some(vec![22, 99]))?;
        let target: Target = Target::new(vec![host]);
        let ret = tcp_null_scan(target, src_ipv4, src_port, threads_num, timeout).unwrap();
        println!("{}", ret);
        Ok(())
    }
    #[test]
    fn test_udp_scan() -> Result<()> {
        let src_ipv4: Option<Ipv4Addr> = Some(Ipv4Addr::new(192, 168, 72, 128));
        let src_port: Option<u16> = None;
        let dst_ipv4: Ipv4Addr = Ipv4Addr::new(192, 168, 72, 135);
        let threads_num: usize = 8;
        let timeout = Some(Duration::new(3, 0));
        let host = Host::new(dst_ipv4, Some(vec![22, 99]))?;
        let target: Target = Target::new(vec![host]);
        let ret = udp_scan(target, src_ipv4, src_port, threads_num, timeout).unwrap();
        println!("{}", ret);
        Ok(())
    }
    #[test]
    fn test_ip_scan() -> Result<()> {
        use pnet::packet::ip::IpNextHeaderProtocols;
        let protocol = Some(IpNextHeaderProtocols::Udp);
        let src_ipv4: Option<Ipv4Addr> = Some(Ipv4Addr::new(192, 168, 72, 128));
        let src_port: Option<u16> = None;
        let dst_ipv4: Ipv4Addr = Ipv4Addr::new(192, 168, 72, 135);
        let threads_num: usize = 8;
        let timeout = Some(Duration::new(3, 0));
        let host = Host::new(dst_ipv4, Some(vec![22, 99]))?;
        let target: Target = Target::new(vec![host]);
        let ret =
            ip_procotol_scan(target, src_ipv4, src_port, protocol, threads_num, timeout).unwrap();
        println!("{}", ret);
        Ok(())
    }
}
