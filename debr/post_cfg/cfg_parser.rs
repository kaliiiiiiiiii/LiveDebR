use std::fs::{self,OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom, BufRead, BufReader};
use std::path::Path;


pub fn add(content: &str, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if path.exists() {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        if reader.lines().any(|line| line.as_ref().map(|l| l == content).unwrap_or(false)) {
            return Ok(());
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

    // Filter out lines containing the content
    let filtered_lines: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| line != content)
        .collect();

    // Rewrite the file with the filtered content
    let mut file = fs::File::create(path)?;
    for line in filtered_lines {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}


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
    if let Some(kv_position) = get(key, path)? {
        if kv_position.value != value {
            let mut file = OpenOptions::new().write(true).open(path)?;
            file.seek(SeekFrom::Start(kv_position.start_pos))?;

            let new_data = format!("{}=\"{}\"\n", key, value);
            let new_data_bytes = new_data.as_bytes();

            let mut remaining_content = Vec::new();
            let mut file_for_reading = OpenOptions::new().read(true).open(path)?;
            file_for_reading.seek(SeekFrom::Start(kv_position.end_pos))?;
            file_for_reading.read_to_end(&mut remaining_content)?;

            let mut full_data = Vec::new();
            full_data.extend_from_slice(new_data_bytes);
            full_data.extend_from_slice(&remaining_content);

            file.write_all(&full_data)?;

            let pos = file.stream_position()?;
            file.set_len(pos)?;
        }
    } else {
        let mut file = OpenOptions::new().write(true).append(true).open(path)?;
        let new_data = format!("\n{}=\"{}\"", key, value);
        file.write_all(new_data.as_bytes())?;
    }

    Ok(())
}