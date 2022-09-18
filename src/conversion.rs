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

pub fn convert_rate(rate: i64) -> String {
    let rate = convert_bytes(rate);
    format!("{}/s", rate)
}

pub fn get_percentage(percent: f32) -> String {
    let percent = percent * 100f32;
    format!("{} %", percent)
}

pub fn status_string(id: &i64) -> &'static str {
    match id {
        0 => "Stopped",
        1 => "Queued to verify local data",
        2 => "Verifying local data",
        3 => "Queued to download",
        4 => "Downloading",
        5 => "Queued to seed",
        6 => "Seeding",
        _ => "",
    }
}
