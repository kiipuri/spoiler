use std::cmp::Ordering;

use byte_unit::{Byte, ByteUnit};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use transmission_rpc::types::Torrent;

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

pub fn get_status_percentage(torrent: &Torrent) -> String {
    match torrent.status.unwrap() {
        2 => get_percentage(torrent.recheck_progress.unwrap()),
        4 => get_percentage(torrent.percent_done.unwrap()),
        6 => get_percentage(torrent.percent_done.unwrap()),
        _ => "".to_string(),
    }
}

pub fn get_percentage(percent: f32) -> String {
    let percent = percent * 100f32;
    format!("{:.1} %", percent)
}

pub fn status_string(status: &i64) -> &'static str {
    match status {
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

pub fn date(date: i64) -> String {
    if date == 0 {
        return "".to_string();
    }
    let naive = NaiveDateTime::from_timestamp(date, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    datetime.format("%d/%m/%Y %H:%M").to_string()
}

pub fn convert_secs(secs: i64) -> String {
    match secs {
        -1 => return String::from("Done"),
        -2 => return String::from("Inf"),
        _ => (),
    }

    let mut time = Duration::seconds(secs);
    let mut time_str = String::new();

    let days = time.num_days();
    if days >= 1 {
        time_str.push_str(format!("{}d ", days).as_str());
        return time_str;
    }

    let hours = time.num_hours();
    if hours >= 1 {
        time_str.push_str(format!("{}h ", hours).as_str());
        time = time
            .checked_sub(&Duration::hours(time.num_hours()))
            .unwrap();
    }

    let minutes = time.num_minutes();
    if minutes >= 1 {
        time_str.push_str(format!("{}min ", minutes).as_str());
        time = time
            .checked_sub(&Duration::minutes(time.num_minutes()))
            .unwrap();
    }

    let seconds = time.num_seconds();
    if seconds >= 1 {
        time_str.push_str(format!("{}sec", seconds).as_str());
    }

    time_str
}

pub fn compare_int(a: i64, b: i64) -> Ordering {
    a.cmp(&b)
}

pub fn compare_float(a: f32, b: f32) -> Ordering {
    if a.min(b) == a {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

pub fn compare_string(a: &String, b: &String) -> Ordering {
    a.cmp(b)
}
