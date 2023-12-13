// 365 days
pub const SEC_PER_YEAR: u64 = 31536000;

pub fn year_to_secs(year: u64) -> u64 {
    year * SEC_PER_YEAR
}

pub fn sec_to_years(sec: u64) -> u64 {
    sec / SEC_PER_YEAR
}
