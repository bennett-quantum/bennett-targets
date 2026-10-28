//! Validator for target descriptions (bennett-isa-spec section 8). Skeleton.

/// Validate a target description. Currently only rejects empty input; full
/// JSON parsing and schema checks are future work (no dependencies yet).
pub fn validate(json: &str) -> Result<(), String> {
    if json.trim().is_empty() {
        return Err("target description is empty".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty() {
        assert!(validate("").is_err());
        assert!(validate("  \n").is_err());
    }

    #[test]
    fn accepts_example_target() {
        let s = include_str!("../../targets/example-surface-code-d17.json");
        assert!(validate(s).is_ok());
    }
}
