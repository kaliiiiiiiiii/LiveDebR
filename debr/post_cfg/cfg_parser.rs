use std::fs::{self,OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom, BufRead, BufReader};
use std::path::Path;


pub fn add(content: &str, path: &Path) -> io::Result<()> {
    // Create parent directories if they do not exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Normalize the content to check for duplicates
    let normalized_content = content.trim();

    // Check if the file already contains the content
    if path.exists() {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        if reader.lines().any(|line| line.as_ref().map(|l| l.trim() == normalized_content).unwrap_or(false)) {
            return Ok(()); // duplicate
        }
    }

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)?;

    writeln!(file, "\n{}", content)?;

    Ok(())
}

pub fn strip(content: &str, path: &Path) -> io::Result<()> {
    // Return early if the file doesn't exist
    if !path.exists() {
        return Ok(());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);

    // Normalize the content to remove
    let normalized_content = content.trim();

    // Filter out lines containing the content
    let filtered_lines: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| line.trim() != normalized_content)
        .collect();

    // Rewrite the file with the filtered content
    let mut file = fs::File::create(path)?;
    for line in filtered_lines {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}

/// Key-value position
pub struct KeyValuePosition {
    pub value: String,
    pub start_pos: u64,
    pub end_pos: u64,
}

pub fn get(key: &str, path: &Path) -> io::Result<Option<KeyValuePosition>> {
    let file = OpenOptions::new().read(true).open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut position: u64 = 0;
    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        // Skip comment lines
        if line.trim_start().starts_with('#') {
            position += line.len() as u64;
            line.clear();
            continue;
        }

        // Found key
        if line.starts_with(key) {
            let mut parts = line.splitn(2, '=');
            let key_found = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim_matches('"').trim();

            // Ensure that the key matches exactly
            if key_found == key {
                let end_pos = position + line.len() as u64;
                return Ok(Some(KeyValuePosition {
                    value: value.to_string(),
                    start_pos: position,
                    end_pos,
                }));
            }
        }

        position += line.len() as u64;
        line.clear(); // Clear buffer
    }

    Ok(None)
}

pub fn set(key: &str, value: &str, path: &Path) -> io::Result<()> {
    // Get key position
    if let Some(kv_position) = get(key, path)? {
        if kv_position.value != value{
            // Open file for writing
            let mut file = OpenOptions::new().write(true).open(path)?;

            // Seek to position
            file.seek(SeekFrom::Start(kv_position.start_pos))?;

            // Write new key-value
            let new_data = format!("{}=\"{}\"\n", key, value);
            let new_len = new_data.len() as u64;
            let old_len = kv_position.end_pos - kv_position.start_pos;

            // Longer new key-value
            if new_len > old_len {
                // Read remaining content
                let mut remaining_content = Vec::new();
                let mut file_for_reading = OpenOptions::new().read(true).open(path)?;
                file_for_reading.seek(SeekFrom::Start(kv_position.end_pos))?;
                file_for_reading.read_to_end(&mut remaining_content)?;

                // Write new pair
                write!(file, "{}", new_data)?;

                // Append remaining content
                file.write_all(&remaining_content)?;
            }
            // Shorter new key-value
            else {
                write!(file, "{}", new_data)?;

                // Pad with '#'
                if new_len < old_len {
                    let padding = "#".repeat((old_len - new_len) as usize);
                    write!(file, "{}{}", padding, " ".repeat((old_len - new_len) as usize))?;
                }
            }
        }
        
    } else {
        // Key not found
        let mut file = OpenOptions::new().write(true).append(true).open(path)?;
        write!(file, "\n{}=\"{}\"", key, value)?;
    }

    Ok(())
}
