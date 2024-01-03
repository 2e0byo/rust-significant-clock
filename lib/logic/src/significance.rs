use chrono::{DateTime, Local, Timelike};

pub fn is_significant(time: DateTime<Local>) -> bool {
    let same_start_end = |numbers: &[u8]| numbers[0] == numbers[3] && numbers[2] == numbers[5];

    let same_start_end_reversed =
        |numbers: &[u8]| numbers[0] == numbers[5] && numbers[2] == numbers[3];

    let internal_pattern = |time: DateTime<Local>| {
        // TODO use an iterator here not vector
        let numbers: Vec<u8> = time
            .format("%H%M%S")
            .to_string()
            .chars()
            .map(|c| c.try_into().expect("Unable to convert '{c}' to u8!"))
            .collect();
        let diffs: Vec<i8> = numbers
            .windows(2)
            .map(|window| window[1] as i8 - window[0] as i8)
            .collect();
        (diffs[0] == -diffs[4]
            && diffs[1] == -diffs[3]
            && (same_start_end(&numbers) || same_start_end_reversed(&numbers)))
            || diffs == vec![1, 1, 1, 1, 1]
    };

    time.minute() == time.hour() && time.hour() == time.second() // 12:12:12
        || internal_pattern(time) // 12:34:56 || 12:33:21
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::significance::is_significant;
    use chrono::prelude::*;

    #[test]
    fn ascending_descending_is_significant() {
        let time = Local.with_ymd_and_hms(2024, 1, 1, 01, 22, 10).unwrap();
        assert!(is_significant(time))
    }

    #[test]
    fn pairs_are_significant() {
        let time = Local.with_ymd_and_hms(2024, 1, 1, 11, 22, 11).unwrap();
        assert!(is_significant(time))
    }

    #[test]
    fn all_equal_is_significant() {
        let time = Local.with_ymd_and_hms(2024, 1, 1, 11, 11, 11).unwrap();
        assert!(is_significant(time))
    }

    #[test]
    fn write_csv() {
        let mut rows = vec![];
        for h in 0..=23 {
            for m in 0..=59 {
                for s in 0..=59 {
                    let time = Local.with_ymd_and_hms(2024, 1, 1, h, m, s).unwrap();
                    let val = format!("{},{}", time.format("%H:%M:%S"), is_significant(time));
                    rows.push(val);
                }
            }
        }
        fs::write("significance.csv", rows.join("\n")).unwrap();
    }

    #[test]
    fn repeated_pairs_are_significant() {
        let time = Local.with_ymd_and_hms(2024, 1, 1, 23, 23, 23).unwrap();
        assert!(is_significant(time));
    }

    #[test]
    fn invisible_pattern_ignored() {
        let time = Local.with_ymd_and_hms(2024, 1, 1, 23, 57, 54).unwrap();
        assert!(!is_significant(time));
    }

    #[test]
    fn barely_visible_pattern_ignored() {
        let time = Local.with_ymd_and_hms(2024, 1, 1, 00, 16, 55).unwrap();
        assert!(!is_significant(time));
    }

    #[test]
    fn slightly_visible_pattern_ignored() {
        let time = Local.with_ymd_and_hms(2024, 1, 1, 00, 12, 11).unwrap();
        assert!(!is_significant(time));
    }
}
