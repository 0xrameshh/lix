pub fn timestamp_to_iso(ts: f64) -> String {
    let total_secs = ts as i64;
    let micros = ((ts - total_secs as f64) * 1_000_000.0).round() as i64;
    let micros = if micros < 0 { 0 } else { micros as u32 };
    let mut days = total_secs / 86400;
    let remaining = total_secs % 86400;
    let hours = remaining / 3600;
    let mins = (remaining % 3600) / 60;
    let sec = remaining % 60;
    let mut y = 1970i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        y += 1;
    }
    let month_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 0usize;
    let mut d = days;
    loop {
        if d < month_days[m] {
            break;
        }
        d -= month_days[m];
        m += 1;
    }
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}Z",
        y,
        m + 1,
        d + 1,
        hours,
        mins,
        sec,
        micros
    )
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
