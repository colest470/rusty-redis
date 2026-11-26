use rand::Rng;

pub fn parse_command(input: &[u8]) -> Option<Vec<String>> {
    let mut parts = input.split(|&b| b == b'\n');

    // First line: *<count>\r
    let first = parts.next()?;
    let first = strip_cr(first);
    if !first.starts_with(b"*") {
        return None;
    }
    let count: usize = std::str::from_utf8(&first[1..]).ok()?.parse().ok()?;

    let mut args = Vec::with_capacity(count);
    for _ in 0..count {
        // $<len>\r
        let len_line = strip_cr(parts.next()?);
        if !len_line.starts_with(b"$") {
            return None;
        }
        let _len: usize = std::str::from_utf8(&len_line[1..]).ok()?.parse().ok()?;

        // <payload>\r
        let payload = strip_cr(parts.next()?);
        args.push(String::from_utf8(payload.to_vec()).ok()?);
    }

    Some(args)
}

fn strip_cr(line: &[u8]) -> &[u8] {
    if line.last() == Some(&b'\r') { &line[..line.len() - 1] } else { line }
}