fn main() {
    let input = r#"
sprite-session: 3 windows (created Wed Oct 18 10:30:00 2023) [attached]
another-session: 1 windows (created Wed Oct 18 09:15:00 2023)
"#;

    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }

        println!("Processing line: '{}'", line);

        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 2 {
            continue;
        }

        let name = parts[0];
        let info_part = parts[1];

        println!("  name: '{}'", name);
        println!("  info_part: '{}'", info_part);
        println!("  contains [attached]: {}", info_part.contains("[attached]"));
        println!();
    }
}