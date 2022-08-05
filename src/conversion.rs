use byte_unit::{Byte, ByteUnit};

pub fn convert_bytes(size: i64) -> String {
    let mut byteunit = ByteUnit::B;

    if size / 1000 >= 1 {
        byteunit = ByteUnit::KB;
    }

    if size / 1000000 >= 1 {
        byteunit = ByteUnit::MB;
    }

    if size / 1000000000 >= 1 {
        byteunit = ByteUnit::GB;
    }

    if size / 1000000000000 >= 1 {
        byteunit = ByteUnit::TB;
    }

    let byte = Byte::from_bytes(u128::from(size.unsigned_abs()));
    let adjusted_byte = byte.get_adjusted_unit(byteunit);
    adjusted_byte.to_string()
}
