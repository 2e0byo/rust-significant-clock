pub fn is_significant(time: DateTime<Local>) -> bool {
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
        diffs[0] == -diffs[4] && diffs[1] == -diffs[3] || diffs == vec![1, 1, 1, 1, 1]
    };

    time.minute() == time.hour() && time.hour() == time.second() // 12:12:12
        || internal_pattern(time) // 12:34:56 || 12:33:21
}
