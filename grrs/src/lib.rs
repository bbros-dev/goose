use anyhow::Error;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_find_matches() {
        let mut output = Vec::new();
        let _result = find_matches("lorem ipsum\ndolor sit amet", "lorem", &mut output);
        assert_eq!(output, b"lorem ipsum\n");
    }
}

/// Search for a pattern in a multi-line string.
//  Display the lines that contain it.
pub fn find_matches(content: &str, pattern: &str, mut writer: impl std::io::Write) -> Result<(), Error> {
    for line in content.lines() {
        if line.contains(pattern) {
            writeln!(writer, "{}", line)?;
        }
    }
    Ok(())
}
