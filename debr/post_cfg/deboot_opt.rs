use std::collections::HashSet;

pub fn parse(command: &String, include_extras: Vec<String>) -> String {
    let mut extras_set: HashSet<&str> = include_extras.iter().map(|s| s.as_str()).collect();

    if let Some(include_start) = command.find("--include=") {
        let include_end = command[include_start..].find(' ').unwrap_or(command.len());
        let existing_values = &command[include_start + 10..include_start + include_end];
        
        existing_values.split(',').for_each(|value| { extras_set.insert(value); });

        let combined_values = extras_set.into_iter().collect::<Vec<&str>>().join(",");

        format!(
            "{}--include={}{}",
            &command[..include_start],
            combined_values,
            &command[include_start + include_end..]
        )
    } else {
        let unique_values = extras_set.into_iter().collect::<Vec<&str>>().join(",");
        format!("{} --include={}", command, unique_values)
    }
}