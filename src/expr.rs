//! Tokenizing `test` expressions well enough to pull out the names they
//! reference. Deliberately tiny — deon evaluates nothing (DESIGN §9); we only
//! need the identifiers.

/// Split an expression into `(token, is_function_call)` pairs. A token is a
/// maximal run of identifier characters (alphanumeric, `_`, `-`, `.`); it is a
/// function call if the next non-space character is `(`. Kebab-case names stay
/// whole (`fair-value`), while a spaced `a - b` splits into `a` and `b`.
pub(crate) fn tokenize(expr: &str) -> Vec<(String, bool)> {
    let chars: Vec<char> = expr.chars().collect();
    let is_id = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.';
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !is_id(chars[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && is_id(chars[i]) {
            i += 1;
        }
        let tok: String = chars[start..i].iter().collect();
        let mut j = i;
        while j < chars.len() && chars[j] == ' ' {
            j += 1;
        }
        let is_call = chars.get(j) == Some(&'(');
        out.push((tok, is_call));
    }
    out
}
