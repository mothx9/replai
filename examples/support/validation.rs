//! Example-host grammar only: balanced braces, no REPLAI parser dependency.
pub fn classify(
    snapshot: &replai::AnalysisSnapshot,
) -> Result<replai::ValidationDisposition, replai::ValidationError> {
    use replai::{Diagnostic, ValidationDisposition as V};
    let mut balance = 0i32;
    for (index, c) in snapshot.text().char_indices() {
        if c == '{' {
            balance += 1;
        }
        if c == '}' {
            balance -= 1;
        }
        if balance < 0 {
            return Ok(V::Invalid(vec![Diagnostic::new(
                "Unmatched closing brace",
                Some(index..index + 1),
            )?]));
        }
    }
    Ok(if balance > 0 {
        V::Incomplete
    } else {
        V::Complete
    })
}
