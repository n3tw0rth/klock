pub fn recompute(items: &[String], query: &str) -> Vec<usize> {
    if query.trim().is_empty() {
        return (0..items.len()).collect();
    }
    let q = query.to_lowercase();
    items
        .iter()
        .enumerate()
        .filter(|(_, s)| s.to_lowercase().contains(&q))
        .map(|(i, _)| i)
        .collect()
}
