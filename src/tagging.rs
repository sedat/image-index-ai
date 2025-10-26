pub fn parse_tags(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|tag| tag.trim())
        .filter(|tag| !tag.is_empty())
        .map(String::from)
        .collect()
}
