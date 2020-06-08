pub use super::layer::{Layer, LayerType, LayerTypes};
use pnet::packet::tcp::{self, MutableTcpPacket, TcpFlags, TcpPacket};
use std::clone::Clone;
use std::fmt::{self, Display, Formatter};
use std::net::Ipv4Addr;

/// Represents a TCP packet.
#[derive(Clone, Debug)]
pub struct Tcp {
    pub layer: tcp::Tcp,
    pub src: Ipv4Addr,
    pub dst: Ipv4Addr,
}

impl Tcp {
    /// Creates a `Tcp` represents a TCP ACK.
    pub fn new_ack(
        src_ip_addr: Ipv4Addr,
        dst_ip_addr: Ipv4Addr,
        src: u16,
        dst: u16,
        sequence: u32,
        acknowledgement: u32,
    ) -> Tcp {
        Tcp {
            layer: tcp::Tcp {
                source: src,
                destination: dst,
                sequence,
                acknowledgement,
                data_offset: 5,
                reserved: 0,
                flags: TcpFlags::ACK,
                window: 65535,
                checksum: 0,
                urgent_ptr: 0,
                options: vec![],
                payload: vec![],
            },
            src: src_ip_addr,
            dst: dst_ip_addr,
        }
    }

    /// Creates a `Tcp` represents a TCP ACK/SYN.
    pub fn new_ack_syn(
        src_ip_addr: Ipv4Addr,
        dst_ip_addr: Ipv4Addr,
        src: u16,
        dst: u16,
        sequence: u32,
        acknowledgement: u32,
    ) -> Tcp {
        let mut tcp = Tcp::new_ack(
            src_ip_addr,
            dst_ip_addr,
            src,
            dst,
            sequence,
            acknowledgement,
        );
        tcp.layer.flags = TcpFlags::ACK | TcpFlags::SYN;
        tcp
    }

    /// Creates a `Tcp` represents a TCP RST.
    pub fn new_rst(
        src_ip_addr: Ipv4Addr,
        dst_ip_addr: Ipv4Addr,
        src: u16,
        dst: u16,
        sequence: u32,
        acknowledgement: u32,
    ) -> Tcp {
        let mut tcp = Tcp::new_ack(
            src_ip_addr,
            dst_ip_addr,
            src,
            dst,
            sequence,
            acknowledgement,
        );
        tcp.layer.flags = TcpFlags::RST;
        tcp
    }

    /// Creates a `Tcp` according to the given `Tcp`.
    pub fn from(tcp: tcp::Tcp, src: Ipv4Addr, dst: Ipv4Addr) -> Tcp {
        Tcp {
            layer: tcp,
            src,
            dst,
        }
    }

    /// Creates a `Tcp` according to the given TCP packet, source and destination.
    pub fn parse(packet: &TcpPacket, src: Ipv4Addr, dst: Ipv4Addr) -> Tcp {
        Tcp {
            layer: tcp::Tcp {
                source: packet.get_source(),
                destination: packet.get_destination(),
                sequence: packet.get_sequence(),
                acknowledgement: packet.get_acknowledgement(),
                data_offset: packet.get_data_offset(),
                reserved: packet.get_reserved(),
                flags: packet.get_flags(),
                window: packet.get_window(),
                checksum: packet.get_checksum(),
                urgent_ptr: packet.get_urgent_ptr(),
                options: packet.get_options(),
                payload: vec![],
            },
            src,
            dst,
        }
    }

    /// Get the source IP address of the layer.
    pub fn get_src_ip_addr(&self) -> Ipv4Addr {
        self.src
    }

    /// Get the destination IP address of the layer.
    pub fn get_dst_ip_addr(&self) -> Ipv4Addr {
        self.dst
    }

    /// Get the source of the layer.
    pub fn get_src(&self) -> u16 {
        self.layer.source
    }

    /// Get the destination of the layer.
    pub fn get_dst(&self) -> u16 {
        self.layer.destination
    }

    /// Get the sequence of the layer.
    pub fn get_sequence(&self) -> u32 {
        self.layer.sequence
    }

    /// Get the acknowledgement of the layer.
    pub fn get_acknowledgement(&self) -> u32 {
        self.layer.acknowledgement
    }

    /// Returns if the `Tcp` is a TCP acknowledgement.
    pub fn is_ack(&self) -> bool {
        self.layer.flags & TcpFlags::ACK != 0
    }

    /// Returns if the `Tcp` is a TCP reset.
    pub fn is_rst(&self) -> bool {
        self.layer.flags & TcpFlags::RST != 0
    }

    /// Returns if the `Tcp` is a TCP synchronization.
    pub fn is_syn(&self) -> bool {
        self.layer.flags & TcpFlags::SYN != 0
    }

    /// Returns if the `Tcp` is a TCP finish.
    pub fn is_fin(&self) -> bool {
        self.layer.flags & TcpFlags::FIN != 0
    }

    /// Returns if the `Tcp` is a TCP reset or finish.
    pub fn is_rst_or_fin(&self) -> bool {
        self.is_rst() || self.is_fin()
    }

    fn serialize_internal(
        &self,
        buffer: &mut [u8],
        fix_length: bool,
        _: usize,
        compute_checksum: bool,
    ) -> Result<usize, String> {
        let mut packet = match MutableTcpPacket::new(buffer) {
            Some(packet) => packet,
            None => return Err(format!("buffer is too small")),
        };

        packet.populate(&self.layer);

        // Fix length
        if fix_length {
            packet.set_data_offset((self.get_size() / 4) as u8);
        }

        // Compute checksum
        if compute_checksum {
            let checksum = tcp::ipv4_checksum(
                &packet.to_immutable(),
                &self.get_src_ip_addr(),
                &self.get_dst_ip_addr(),
            );
            packet.set_checksum(checksum);
        }

        Ok(self.get_size())
    }
}

impl Display for Tcp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut flags = String::from("[");
        flags = flags + "-";
        if self.is_ack() {
            flags = flags + "A";
        } else {
            flags = flags + ".";
        }
        flags = flags + "-";
        if self.is_rst() {
            flags = flags + "R";
        } else {
            flags = flags + ".";
        }
        if self.is_syn() {
            flags = flags + "S";
        } else {
            flags = flags + ".";
        }
        if self.is_fin() {
            flags = flags + "F";
        } else {
            flags = flags + ".";
        }
        flags = flags + "]";

        write!(
            f,
            "{}: {} -> {} {}",
            LayerTypes::Tcp,
            self.layer.source,
            self.layer.destination,
            flags
        )
    }
}

impl Layer for Tcp {
    fn get_type(&self) -> LayerType {
        LayerTypes::Tcp
    }

    fn get_size(&self) -> usize {
        TcpPacket::packet_size(&self.layer)
    }

    fn serialize(&self, buffer: &mut [u8]) -> Result<usize, String> {
        self.serialize_internal(buffer, false, 0, true)
    }

    fn serialize_n(&self, buffer: &mut [u8], n: usize) -> Result<usize, String> {
        self.serialize_internal(buffer, true, n, true)
    }
}
