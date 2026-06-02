//! File size formatting utilities.
//!
//! Convert raw byte counts into human-readable strings with appropriate
//! units (B, KB, MB, GB, TB, PB). Also provides speed formatting for
//! transfer rates.

// ---------------------------------------------------------------------------
// Unit conversion
// ---------------------------------------------------------------------------

/// Pick the best unit and suffix for a byte count.
///
/// Returns a tuple of `(size_in_unit, suffix)` where `size_in_unit` is the
/// original size divided down to fit in the chosen unit range.
///
/// # Examples
///
/// ```
/// use rusty_rich::filesize::pick_unit_and_suffix;
///
/// let (size, unit) = pick_unit_and_suffix(2048, 1);
/// assert_eq!(format!("{:.1}", size), "2.0");
/// assert_eq!(unit, "KB");
/// ```
pub fn pick_unit_and_suffix(bytes: u64, _precision: usize) -> (f64, &'static str) {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    (size, UNITS[unit_idx])
}

/// Format bytes as a human-readable string.
///
/// Uses binary (1024-based) units. The `precision` parameter controls the
/// number of decimal places (defaults to 1).
///
/// # Examples
///
/// ```
/// use rusty_rich::filesize::format_size;
///
/// assert_eq!(format_size(0, None), "0 B");
/// assert_eq!(format_size(1024, None), "1.0 KB");
/// assert_eq!(format_size(1048576, None), "1.0 MB");
/// assert_eq!(format_size(1073741824, None), "1.0 GB");
/// ```
pub fn format_size(bytes: u64, precision: Option<usize>) -> String {
    let prec = precision.unwrap_or(1);
    if bytes < 1024 {
        return format!("{} B", bytes);
    }
    let (size, unit) = pick_unit_and_suffix(bytes, prec);
    format!("{:.prec$} {}", size, unit)
}

/// Format a transfer speed (bytes per second).
///
/// # Examples
///
/// ```
/// use rusty_rich::filesize::format_speed;
///
/// assert_eq!(format_speed(500), "500 B/s");
/// assert_eq!(format_speed(2048), "2.0 KB/s");
/// ```
pub fn format_speed(bytes_per_sec: u64) -> String {
    if bytes_per_sec < 1024 {
        return format!("{} B/s", bytes_per_sec);
    }
    let (size, unit) = pick_unit_and_suffix(bytes_per_sec, 1);
    format!("{:.1} {}/s", size, unit)
}

/// Format bytes as a human-readable string using SI (1000-based) units.
///
/// This is the decimal equivalent of [`format_size`], using powers of 1000
/// (kB, MB, GB) instead of 1024 (KB, MB, GB). Equivalent to Python Rich's
/// `filesize.decimal()`.
///
/// # Examples
///
/// ```
/// use rusty_rich::filesize::decimal;
///
/// assert_eq!(decimal(0, None, None), "0 B");
/// assert_eq!(decimal(1000, None, None), "1.0 kB");
/// assert_eq!(decimal(1000000, None, None), "1.0 MB");
/// ```
pub fn decimal(bytes: u64, precision: Option<usize>, separator: Option<&str>) -> String {
    const SI_UNITS: &[&str] = &["B", "kB", "MB", "GB", "TB", "PB"];
    let sep = separator.unwrap_or(" ");
    let prec = precision.unwrap_or(1);

    if bytes < 1000 {
        return format!("{bytes}{sep}B");
    }

    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1000.0 && unit_idx < SI_UNITS.len() - 1 {
        size /= 1000.0;
        unit_idx += 1;
    }
    format!("{size:.prec$}{sep}{}", SI_UNITS[unit_idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_unit_and_suffix() {
        let (size, unit) = pick_unit_and_suffix(0, 1);
        assert_eq!(size, 0.0);
        assert_eq!(unit, "B");

        let (size, unit) = pick_unit_and_suffix(1024, 1);
        assert!((size - 1.0).abs() < f64::EPSILON);
        assert_eq!(unit, "KB");

        let (size, unit) = pick_unit_and_suffix(1048576, 1);
        assert!((size - 1.0).abs() < f64::EPSILON);
        assert_eq!(unit, "MB");

        let (size, unit) = pick_unit_and_suffix(1073741824, 1);
        assert!((size - 1.0).abs() < f64::EPSILON);
        assert_eq!(unit, "GB");

        let (size, unit) = pick_unit_and_suffix(1099511627776, 1);
        assert!((size - 1.0).abs() < f64::EPSILON);
        assert_eq!(unit, "TB");
    }

    #[test]
    fn test_format_size_zero() {
        assert_eq!(format_size(0, None), "0 B");
    }

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(500, None), "500 B");
        assert_eq!(format_size(1023, None), "1023 B");
    }

    #[test]
    fn test_format_size_kb() {
        assert_eq!(format_size(1024, None), "1.0 KB");
        assert_eq!(format_size(2048, None), "2.0 KB");
        assert_eq!(format_size(1536, None), "1.5 KB");
    }

    #[test]
    fn test_format_size_mb() {
        assert_eq!(format_size(1048576, None), "1.0 MB");
        assert_eq!(format_size(2097152, None), "2.0 MB");
    }

    #[test]
    fn test_format_size_gb() {
        assert_eq!(format_size(1073741824, None), "1.0 GB");
    }

    #[test]
    fn test_format_size_precision() {
        assert_eq!(format_size(1536, Some(2)), "1.50 KB");
        assert_eq!(format_size(1536, Some(0)), "2 KB");
    }

    #[test]
    fn test_format_speed_bytes() {
        assert_eq!(format_speed(0), "0 B/s");
        assert_eq!(format_speed(500), "500 B/s");
        assert_eq!(format_speed(1023), "1023 B/s");
    }

    #[test]
    fn test_format_speed_kb() {
        assert_eq!(format_speed(1024), "1.0 KB/s");
        assert_eq!(format_speed(2048), "2.0 KB/s");
    }

    #[test]
    fn test_format_speed_mb() {
        assert_eq!(format_speed(1048576), "1.0 MB/s");
    }

    #[test]
    fn test_decimal_zero() {
        assert_eq!(decimal(0, None, None), "0 B");
    }

    #[test]
    fn test_decimal_bytes() {
        assert_eq!(decimal(500, None, None), "500 B");
        assert_eq!(decimal(999, None, None), "999 B");
    }

    #[test]
    fn test_decimal_kb() {
        assert_eq!(decimal(1000, None, None), "1.0 kB");
        assert_eq!(decimal(1500, None, None), "1.5 kB");
        assert_eq!(decimal(2000, None, None), "2.0 kB");
    }

    #[test]
    fn test_decimal_mb() {
        assert_eq!(decimal(1_000_000, None, None), "1.0 MB");
        assert_eq!(decimal(2_500_000, None, None), "2.5 MB");
    }

    #[test]
    fn test_decimal_gb() {
        assert_eq!(decimal(1_000_000_000, None, None), "1.0 GB");
    }

    #[test]
    fn test_decimal_precision() {
        assert_eq!(decimal(1500, Some(2), None), "1.50 kB");
        assert_eq!(decimal(1500, Some(0), None), "2 kB");
    }

    #[test]
    fn test_decimal_separator() {
        assert_eq!(decimal(1500, None, Some("")), "1.5kB");
    }
}
