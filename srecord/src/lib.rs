use std::error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::num::Wrapping;
use std::path::Path;

type Result<T> = std::result::Result<T, SrecordError>;

#[derive(Debug)]
pub enum SrecordError {
    InvalidLine(u32, &'static str),
    IO(io::Error),
}

impl fmt::Display for SrecordError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SrecordError::InvalidLine(ref l, ref s) => {
                write!(f, "invalid s-record line {}: {}", l, s)
            }
            SrecordError::IO(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for SrecordError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            SrecordError::InvalidLine(_, _) => None,
            SrecordError::IO(ref e) => Some(e),
        }
    }
}

impl From<io::Error> for SrecordError {
    fn from(err: io::Error) -> SrecordError {
        SrecordError::IO(err)
    }
}

pub fn parse_file(p: &Path) -> Result<Srecord> {
    let file = File::open(p)?;
    let buf_reader = BufReader::new(file);

    let mut records = Vec::new();

    for (i, l) in buf_reader.lines().enumerate() {
        use crate::SrecType::*;

        macro_rules! invalid_line {
            ($msg:expr_2021) => {
                return Err(SrecordError::InvalidLine(i as u32 + 1, $msg));
            };
        }

        let raw_line = l?;
        let line = raw_line.trim().as_bytes();

        if line.len() < 10 || line.len() > 514 || line[0] != b'S' {
            invalid_line!("invalid length or doesn't start with S");
        }

        let converted_hex = convert_hex(&line[2..]); // Bytecount, address, data, checksum

        let bcount = converted_hex[0];
        if converted_hex.len() - 1 != bcount as usize {
            invalid_line!("invalid byte count");
        }

        let mut data_start = 3;
        let checksum = converted_hex[converted_hex.len() - 1];

        let mut record = Line::new(SrecType::from_byte(line[1]));

        match record.rec_type {
            SrecHeader | SrecData16 | SrecCount16 | SrecStart16 => {
                record.address = bytes_to_address(&converted_hex[1..3]);
            }
            SrecData24 | SrecCount24 | SrecStart24 => {
                record.address = bytes_to_address(&converted_hex[1..4]);
                data_start = 4;
            }
            SrecData32 | SrecStart32 => {
                record.address = bytes_to_address(&converted_hex[1..5]);
                data_start = 5;
            }
            _ => {}
        }

        let data = &converted_hex[data_start..converted_hex.len() - 1];
        record.data = data.to_vec();

        if record.gen_checksum() != checksum {
            invalid_line!("checksum doesn't match");
        }

        match record.rec_type {
            SrecCount16 | SrecCount24 => {
                if records.len() - 1 != record.address as usize {
                    invalid_line!("count doesn't match number of data lines");
                }
            }
            _ => {}
        }

        records.push(record);
    }

    Ok(Srecord(records))
}

fn convert_hex(bytes: &[u8]) -> Vec<u8> {
    let mut converted = Vec::new();

    if bytes.len() % 2 != 0 {
        return converted;
    }

    for i in 0..bytes.len() / 2 {
        let num = (hex_to_byte(bytes[i * 2]) << 4) | hex_to_byte(bytes[(i * 2) + 1]);
        converted.push(num);
    }

    converted
}

fn bytes_to_address(bytes: &[u8]) -> u32 {
    match bytes.len() {
        2 => ((u32::from(bytes[0])) << 8) | u32::from(bytes[1]),
        3 => ((u32::from(bytes[0])) << 16) | ((u32::from(bytes[1])) << 8) | u32::from(bytes[2]),
        4 => {
            ((u32::from(bytes[0])) << 24)
                | ((u32::from(bytes[1])) << 16)
                | ((u32::from(bytes[2])) << 8)
                | u32::from(bytes[3])
        }
        _ => 0,
    }
}

fn to_hex_string(bytes: &[u8]) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
    strs.join("")
}

fn hex_to_byte(h: u8) -> u8 {
    match h as char {
        '0'..='9' => h - 0x30,
        'A'..='F' => h - 0x37,
        _ => 0,
    }
}

pub struct Srecord(pub Vec<Line>);

impl Srecord {
    pub fn add_header(&mut self, data: &str) {
        self.0.push(Line::new_with_data(
            SrecType::SrecHeader,
            (*data.as_bytes()).to_vec(),
        ));
    }

    pub fn add_record16(&mut self, rec_type: SrecType, address: u16, data: &[u8]) {
        self.0.push(Line {
            rec_type,
            address: u32::from(address),
            data: data.to_vec(),
        });
    }

    pub fn add_record32(&mut self, rec_type: SrecType, address: u32, data: &[u8]) {
        self.0.push(Line {
            rec_type,
            address,
            data: data.to_vec(),
        });
    }
}

impl fmt::Display for Srecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.0 {
            writeln!(f, "{}", line)?
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum SrecType {
    SrecUnknown,
    SrecHeader,
    SrecData16,
    SrecData24,
    SrecData32,
    SrecReserved,
    SrecCount16,
    SrecCount24,
    SrecStart32,
    SrecStart24,
    SrecStart16,
}

impl SrecType {
    fn from_byte(c: u8) -> SrecType {
        match c as char {
            '0' => SrecType::SrecHeader,
            '1' => SrecType::SrecData16,
            '2' => SrecType::SrecData24,
            '3' => SrecType::SrecData32,
            '4' => SrecType::SrecReserved,
            '5' => SrecType::SrecCount16,
            '6' => SrecType::SrecCount24,
            '7' => SrecType::SrecStart32,
            '8' => SrecType::SrecStart24,
            '9' => SrecType::SrecStart16,
            _ => SrecType::SrecUnknown,
        }
    }

    fn to_char(&self) -> char {
        match self {
            SrecType::SrecHeader => '0',
            SrecType::SrecData16 => '1',
            SrecType::SrecData24 => '2',
            SrecType::SrecData32 => '3',
            SrecType::SrecReserved => '4',
            SrecType::SrecCount16 => '5',
            SrecType::SrecCount24 => '6',
            SrecType::SrecStart32 => '7',
            SrecType::SrecStart24 => '8',
            SrecType::SrecStart16 => '9',
            _ => '\0',
        }
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    pub rec_type: SrecType,
    pub address: u32,
    pub data: Vec<u8>,
}

impl Line {
    fn new(rec_type: SrecType) -> Line {
        Line {
            rec_type,
            address: 0,
            data: Vec::new(),
        }
    }

    fn new_with_data(rec_type: SrecType, data: Vec<u8>) -> Line {
        Line {
            rec_type,
            address: 0,
            data,
        }
    }

    fn gen_checksum(&self) -> u8 {
        // Checksum = !(byte count + address + data (wrapped to u8))
        use crate::SrecType::*;

        let mut sum = Wrapping(self.byte_count());

        // All addresses are at least 16 bits
        sum += Wrapping(self.address as u8);
        sum += Wrapping((self.address >> 8) as u8);

        match self.rec_type {
            SrecData24 | SrecStart24 => sum += Wrapping((self.address >> 16) as u8),
            SrecData32 | SrecStart32 => {
                sum += Wrapping((self.address >> 16) as u8);
                sum += Wrapping((self.address >> 24) as u8);
            }
            _ => {}
        }

        for d in &self.data {
            sum += Wrapping(d.to_owned());
        }

        !sum.0
    }

    fn byte_count(&self) -> u8 {
        use crate::SrecType::*;

        let mut count = self.data.len() as u8; // Data
        count += 1; // Checksum byte

        // Address byte length
        count += match self.rec_type {
            SrecHeader | SrecData16 | SrecCount16 | SrecStart16 => 2,
            SrecData24 | SrecCount24 | SrecStart24 => 3,
            SrecData32 | SrecStart32 => 4,
            _ => 0,
        };

        count
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "S{}{:02X}{:04X}{}{:02X}",
            self.rec_type.to_char(),
            self.byte_count(),
            self.address,
            to_hex_string(&self.data),
            self.gen_checksum()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_checksum {
        ($func:ident, $line_type:ident, $address:expr_2021, $expected:expr_2021, [$($data:expr_2021),*]) => {
            #[test]
            fn $func() {
                let l = Line {
                    rec_type: SrecType::$line_type,
                    address: $address,
                    data: vec![$($data),*],
                };

                assert_eq!(l.gen_checksum(), $expected);
            }
        };
    }

    test_checksum!(
        test_header,
        SrecHeader,
        0x0000,
        0x3C,
        [0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x20, 0x20, 0x20, 0x20, 0x00, 0x00]
    );

    test_checksum!(
        test_data16_1,
        SrecData16,
        0x0038,
        0x42,
        [0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x2E, 0x0A, 0x00]
    );

    test_checksum!(
        test_data16_2,
        SrecData16,
        0x0000,
        0x26,
        [
            0x7C, 0x08, 0x02, 0xA6, 0x90, 0x01, 0x00, 0x04, 0x94, 0x21, 0xFF, 0xF0, 0x7C, 0x6C,
            0x1B, 0x78, 0x7C, 0x8C, 0x23, 0x78, 0x3C, 0x60, 0x00, 0x00, 0x38, 0x63, 0x00, 0x00
        ]
    );

    test_checksum!(test_count16, SrecCount16, 0x0003, 0xF9, []);

    test_checksum!(test_start16, SrecStart16, 0x0000, 0xFC, []);

    #[test]
    fn test_convert_hex() {
        let hex_str = "0123456789ABCDEF".as_bytes();
        let converted = convert_hex(hex_str);

        assert_eq!(converted, [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]);
    }
}
