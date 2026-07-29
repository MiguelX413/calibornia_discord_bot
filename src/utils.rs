pub(crate) fn chunk_lines(lines: Vec<String>, limit: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut pending = Vec::new();
    let mut pending_len = 0;

    for line in lines {
        let separator_len = usize::from(!pending.is_empty());
        if !pending.is_empty() && pending_len + separator_len + line.len() > limit {
            chunks.push(pending.join("\n"));
            pending.clear();
            pending_len = 0;
        }

        pending_len += line.len() + usize::from(!pending.is_empty());
        pending.push(line);
    }

    if !pending.is_empty() {
        chunks.push(pending.join("\n"));
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_lines_respects_limit_between_lines() {
        let chunks = chunk_lines(vec!["abc".into(), "def".into(), "g".into()], 7);

        assert_eq!(chunks, vec!["abc\ndef", "g"]);
    }
}
