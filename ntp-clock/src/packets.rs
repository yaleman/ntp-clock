use crate::{ClockError, constants::NTP_MIN_PACKET_LEN};

use heapless::string::String as HeaplessString;
use packed_struct::prelude::*;
#[cfg(feature = "std")]
use std::{fmt::Display, net::IpAddr};

#[derive(PackedStruct)]
pub struct NtpRequestPacket {
    firstbyte: u8,
    leap: u8,
    mode: u8,
    stratum: u8,
    poll: u8,
    data: [u8; 43],
}

impl Default for NtpRequestPacket {
    fn default() -> Self {
        Self {
            firstbyte: 27,
            leap: 0,
            mode: 0,
            stratum: 0,
            poll: 0,
            data: [0u8; 43],
        }
    }
}

#[repr(u8)]
#[derive(PrimitiveEnum_u8, Debug, Clone, Copy)]
pub enum NtpMode {
    Reserved = 0,
    SymmetricActive = 1,
    SymmetricPassive = 2,
    Client = 3,
    Server = 4,
    Broadcast = 5,
    ReservedForNtpControlMessages = 6,
    ReservedForPrivateUse = 7,
}

#[derive(PackedStruct, Clone, Debug)]
#[packed_struct(bit_numbering = "msb0")]
pub struct NtpResponse {
    // flags
    #[packed_field(bits = "0..=1")]
    pub leap_indicator: u8,
    #[packed_field(bits = "2..=4")]
    pub version: u8,
    #[packed_field(bits = "5..=7", ty = "enum")]
    /// Indicates the NTP modes.
    /// 0: reserved
    /// 1: symmetric active
    /// 2: symmetric passive
    /// 3: client
    /// 4: server
    /// 5: broadcast
    /// 6: reserved for NTP control messages
    /// 7: reserved for private use
    pub mode: EnumCatchAll<NtpMode>,

    // rest of the fields
    pub stratum: u8,
    /// This is an eight-bit signed integer indicating the
    // maximum interval between successive messages, in seconds to the
    // nearest power of two. The values that can appear in this field
    // presently range from 4 (16 s) to 14 (16284 s); however, most
    // applications use only the sub-range 6 (64 s) to 10 (1024 s).
    #[packed_field(endian = "msb")]
    pub poll: i8,
    #[packed_field(endian = "msb")]
    /// Precision: This is an eight-bit signed integer indicating the
    /// precision of the local clock, in seconds to the nearest power of two.
    /// The values that normally appear in this field range from -6 for
    /// mains-frequency clocks to -20 for microsecond clocks found in some
    /// workstations.
    pub precision: i8,

    #[packed_field(endian = "msb")]
    /// Root Delay: This is a 32-bit signed fixed-point number indicating the
    /// total roundtrip delay to the primary reference source, in seconds
    /// with fraction point between bits 15 and 16. Note that this variable
    /// can take on both positive and negative values, depending on the
    /// relative time and frequency offsets. The values that normally appear
    /// in this field range from negative values of a few milliseconds to
    /// positive values of several hundred milliseconds.
    root_delay_ms: [u8; 4],
    #[packed_field(endian = "msb")]
    /// Indicates the estimated dispersion to the primary synchronizing source.
    dispersion: [u8; 4],
    #[packed_field(endian = "msb")]
    /// A four-byte reference identifier identifying the particular server or reference clock.
    pub identifier: u32,
    #[packed_field(endian = "msb")]
    /// Indicates the local time at which the local clock is last set or corrected.Value 0 indicates that the local clock is never synchronized.
    pub ref_time: u64,
    #[packed_field(endian = "msb")]
    /// Indicates the local time at which the NTP request is sent from the client host.
    pub origin_time: u64,
    #[packed_field(endian = "msb")]
    /// Indicates the local time at which the request arrives at the service host.
    pub recv_time: u64,
    #[packed_field(endian = "msb")]
    /// Indicates the local time at which the response packet is sent from the service host to the client host.
    pub transmit_time: u64,

    #[packed_field(endian = "msb", optional = "true")]
    pub authenticator: [u8; 12],
}

impl NtpResponse {
    /// Create an NTP response packet from a given UNIX timestamp in nanoseconds.
    pub fn from_nanos(unix_nanos: u64) -> NtpResponse {
        // let mut packet = [0u8; NTP_PACKET_LEN];
        // let unix_seconds = unix_nanos / 1_000_000_000;
        // let nanos = (unix_nanos % 1_000_000_000) as u128;
        // let seconds = (unix_seconds as i64 + NTP_UNIX_EPOCH) as u32;
        // let fraction = ((nanos << 32) / 1_000_000_000u128) as u32;
        // packet[40..44].copy_from_slice(&ntp_seconds.to_be_bytes());
        // packet[44..NTP_PACKET_LEN].copy_from_slice(&fraction.to_be_bytes());

        #[allow(clippy::expect_used)]
        Self {
            leap_indicator: 0,
            version: 3,
            mode: NtpMode::Server.into(),
            stratum: 1,
            poll: 4,
            precision: -25,
            root_delay_ms: [0, 0, 0, 15],         // 15 microseconds
            dispersion: [0x00, 0x00, 0x99, 0x9f], // 0.6 seconds
            identifier: 0x50505300,               // Generic PPS
            ref_time: unix_nanos,
            origin_time: 0,
            recv_time: unix_nanos,
            transmit_time: unix_nanos,
            authenticator: [0u8; 12],
        }
    }

    #[cfg(not(feature = "std"))]
    pub fn to_string(&self) -> HeaplessString<256> {
        {
            use core::fmt::Write;
            let mut s = HeaplessString::<256>::new();
            let _ = core::write!(s, "{}", self.ref_time,);
            s
        }
    }

    #[cfg(feature = "std")]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        {
            format!(
                "NtpResponse {{ leap_indicator: {}, version: {}, mode: {:?}, stratum: {}, poll: {}, precision: {}, delay_ms: {:?}, dispersion: {}, identifier: {}, ref_time: {}, origin_time: {}, recv_time: {}, transmit_time: {} }}",
                self.leap_indicator,
                self.version,
                self.mode,
                self.stratum,
                self.poll,
                self.precision,
                self.delay_ms(),
                self.dispersion(),
                match self.remote_id() {
                    Ok(id) => match id {
                        NtpIdentifier::IpAddr(ip) => format!("IP({})", ip),
                        NtpIdentifier::Source(s) => format!("Source({})", s),
                    },
                    Err(_) => "Invalid Identifier".to_string(),
                },
                self.ref_time,
                self.origin_time,
                self.recv_time,
                self.transmit_time
            )
        }
        #[cfg(not(feature = "std"))]
        {
            use core::str::FromStr;

            heapless::String::<256>::from_str(
                "NtpResponse string representation not available in no_std",
            )
        }
    }

    /// Returns the packed bytes of the NTP response packet without the authenticator bytes
    pub fn as_bytes(&self) -> Result<[u8; NTP_MIN_PACKET_LEN], packed_struct::PackingError> {
        let bytes = self.pack()?;
        let mut array = [0u8; NTP_MIN_PACKET_LEN];
        array.copy_from_slice(&bytes[0..NTP_MIN_PACKET_LEN]);
        Ok(array)
    }

    /// Get the remote ID as an IpAddr
    pub fn remote_id(&self) -> Result<NtpIdentifier, ClockError> {
        match self.stratum {
            0 => {
                let vecval = heapless::Vec::from(self.identifier.to_be_bytes());

                match HeaplessString::from_utf8(vecval) {
                    Ok(s) => Ok(NtpIdentifier::Source(s)),
                    Err(_) => Err(ClockError::InvalidIdentifier),
                }
            }
            _ => {
                #[cfg(feature = "std")]
                {
                    let bytes = self.identifier.to_be_bytes();
                    let ip = IpAddr::from(bytes);
                    Ok(NtpIdentifier::IpAddr(ip))
                }
                #[cfg(not(feature = "std"))]
                Ok(NtpIdentifier::IpAddr(self.identifier))
            }
        }
    }

    /// Calculate the offset between the local clock and the NTP server clock in nanoseconds.
    pub fn offset_from_local(&self, local_time_nanos: u64) -> i64 {
        let origin_time: i128 = match self.origin_time as i128 {
            0 => local_time_nanos as i128,
            v => v,
        };

        ((self.recv_time as i128 - origin_time)
            + (self.transmit_time as i128 - local_time_nanos as i128))
            .saturating_div(2) as i64
    }

    /// Root Delay: This is a 32-bit signed fixed-point number indicating the
    /// total roundtrip delay to the primary reference source, in seconds
    /// with fraction point between bits 15 and 16. Note that this variable
    /// can take on both positive and negative values, depending on the
    /// relative time and frequency offsets. The values that normally appear
    /// in this field range from negative values of a few milliseconds to
    /// positive values of several hundred milliseconds.
    pub fn delay_ms(&self) -> Option<f32> {
        let root_delay = RootDelay::unpack(&self.root_delay_ms).ok()?;
        Some(root_delay.to_milliseconds())
    }

    pub fn dispersion(&self) -> f32 {
        f32::from_be_bytes(self.dispersion)
    }

    // From [RFC2030 Section 4](https://www.rfc-editor.org/rfc/rfc2030.html#section-4)
    pub fn leap_identifier_string(&self) -> &'static str {
        match self.leap_indicator {
            0 => "no warning",
            1 => "last minute has 61 seconds",
            2 => "last minute has 59 seconds",
            3 => "alarm condition (clock not synchronized)",
            _ => "undefined",
        }
    }

    pub fn stratum_string(&self) -> &'static str {
        match self.stratum {
            0 => "unspecified or invalid",
            1 => "primary reference (e.g., radio clock)",
            2..=15 => "secondary reference (via NTP or SNTP)",
            16..=255 => "reserved",
        }
    }
}

pub enum NtpClockSource {
    GOES,
    GPS,
    GAL,
    PPS,
    IRIG,
    WWVB,
    DCF,
    HBG,
    MSF,
    JJY,
    LORC,
    TDF,
    CHU,
    WWV,
    WWVH,
    NIST,
    ACTS,
    USNO,
    PTB,
}
#[cfg(feature = "std")]
impl TryFrom<&str> for NtpClockSource {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "GOES" => Ok(Self::GOES),
            "GPS" => Ok(Self::GPS),
            "GAL" => Ok(Self::GAL),
            "PPS" => Ok(Self::PPS),
            "IRIG" => Ok(Self::IRIG),
            "WWVB" => Ok(Self::WWVB),
            "DCF" => Ok(Self::DCF),
            "HBG" => Ok(Self::HBG),
            "MSF" => Ok(Self::MSF),
            "JJY" => Ok(Self::JJY),
            "LORC" => Ok(Self::LORC),
            "TDF" => Ok(Self::TDF),
            "CHU" => Ok(Self::CHU),
            "WWV" => Ok(Self::WWV),
            "WWVH" => Ok(Self::WWVH),
            "NIST" => Ok(Self::NIST),
            "ACTS" => Ok(Self::ACTS),
            "USNO" => Ok(Self::USNO),
            "PTB" => Ok(Self::PTB),
            _ => Err(()),
        }
    }
}

#[cfg(any(test, feature = "std"))]
impl Display for NtpClockSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::GOES => "Geosynchronous Orbit Environment Satellite",
                Self::GPS => "Global Position System",
                Self::GAL => "Galileo Positioning System",
                Self::PPS => "Generic pulse-per-second",
                Self::IRIG => "Inter-Range Instrumentation Group",
                Self::WWVB => "LF Radio WWVB Ft. Collins, CO 60 kHz",
                Self::DCF => "LF Radio DCF77 Mainflingen, DE 77.5 kHz",
                Self::HBG => "LF Radio HBG Prangins, HB 75 kHz",
                Self::MSF => "LF Radio MSF Anthorn, UK 60 kHz",
                Self::JJY => "LF Radio JJY Fukushima, JP 40 kHz, Saga, JP 60 kHz",
                Self::LORC => "MF Radio LORAN C station, 100 kHz",
                Self::TDF => "MF Radio Allouis, FR 162 kHz",
                Self::CHU => "HF Radio CHU Ottawa, Ontario",
                Self::WWV => "HF Radio WWV Ft. Collins, CO",
                Self::WWVH => "HF Radio WWVH Kauai, HI",
                Self::NIST => "NIST telephone modem",
                Self::ACTS => "NIST telephone modem",
                Self::USNO => "USNO telephone modem",
                Self::PTB => "European telephone modem",
            }
        )
    }
}

pub enum NtpIdentifier {
    #[cfg(feature = "std")]
    IpAddr(IpAddr),
    #[cfg(not(feature = "std"))]
    IpAddr(u32),
    Source(HeaplessString<4>),
}

impl NtpIdentifier {
    pub fn as_u32(&self) -> u32 {
        match self {
            #[cfg(feature = "std")]
            NtpIdentifier::IpAddr(ip) => match ip {
                IpAddr::V4(v4) => u32::from_be_bytes(v4.octets()),
                // TODO: Handle IPv6 properly?
                IpAddr::V6(_) => 0,
            },
            #[cfg(not(feature = "std"))]
            NtpIdentifier::IpAddr(ip_u32) => *ip_u32,
            NtpIdentifier::Source(s) => {
                let bytes = s.as_bytes();
                let mut arr = [0u8; 4];
                for (i, b) in bytes.iter().enumerate().take(4) {
                    arr[i] = *b;
                }
                u32::from_be_bytes(arr)
            }
        }
    }
}

#[derive(PackedStruct)]
#[packed_struct(bit_numbering = "msb0")]
struct RootDelay {
    #[packed_field(endian = "msb", bits = "0..=15")]
    int_part: u16,
    #[packed_field(endian = "msb", bits = "16..=31")]
    frac_part: u16,
}
impl RootDelay {
    pub fn to_milliseconds(&self) -> f32 {
        let int_ms = self.int_part as f32;
        let frac_ms = (self.frac_part as f32) / 65536.0;
        int_ms + frac_ms
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::NTP_MIN_PACKET_LEN;

    #[test]
    fn test_packet_size() {
        let req = NtpRequestPacket::default();
        let packed_req = req.pack().expect("Should pack NtpRequestPacket");
        assert_eq!(
            packed_req.len(),
            NTP_MIN_PACKET_LEN,
            "Packed NtpRequestPacket size mismatch, should be {NTP_MIN_PACKET_LEN} bytes"
        );

        let response = NtpResponse::from_nanos(0);
        let packed_resp = response.as_bytes().expect("Should pack NtpResponse");

        println!(
            "{} ",
            packed_resp
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ")
        );

        assert_eq!(
            packed_resp.len(),
            NTP_MIN_PACKET_LEN,
            "Packed NtpResponse size mismatch, should be {NTP_MIN_PACKET_LEN} bytes"
        );
    }
    #[test]
    fn test_root_delay() {
        let test_bytes = [0x00, 0x00, 0x04, 0x78];
        let root_delay = RootDelay::unpack(&test_bytes).expect("Should unpack RootDelay");
        let delay_ms = root_delay.to_milliseconds();
        assert_eq!(
            delay_ms, 0.017456055,
            "RootDelay to_milliseconds calculation incorrect"
        );
    }
}
