/// Replaces invalid identifier chars.
fn replace_invalid_identifier_chars(s: &str) -> String {
    s.strip_prefix('$')
        .unwrap_or(s)
        .replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
}

/// Replaces numeric at the beginning of a string.
fn replace_numeric_start(s: &str) -> String {
    if s.chars().next().is_some_and(char::is_numeric) {
        format!("_{s}")
    } else {
        s.to_string()
    }
}

/// Removes excessive underscores.
fn remove_excess_underscores(s: &str) -> String {
    let mut result = String::new();
    let mut char_iter = s.chars().peekable();

    while let Some(c) = char_iter.next() {
        let next_c = char_iter.peek();
        if c != '_' || !matches!(next_c, Some('_')) {
            result.push(c);
        }
    }

    result
}

/// Capitalizes a string.
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

/// Converts a string to enum variant identifier.
pub fn str_to_enum_variant(s: &str) -> syn::Ident {
    let parts: Vec<_> = s.split([' ', '_', '-']).map(capitalize).collect();
    parse_str(&parts.join(""))
}

/// Converts a string to identifier
pub fn parse_str(s: &str) -> syn::Ident {
    if s.is_empty() {
        return quote::format_ident!("empty_");
        // return syn::Ident::new("empty_", Span::call_site());
    }

    if s.chars().all(|c| c == '_') {
        return quote::format_ident!("underscore_");
        // return syn::Ident::new("underscore_", Span::call_site());
    }

    let s = replace_invalid_identifier_chars(s);
    let s = replace_numeric_start(&s);
    let s = remove_excess_underscores(&s);

    if s.is_empty() {
        return quote::format_ident!("invalid_");
        // return syn::Ident::new("invalid_", Span::call_site());
    }

    let keywords = [
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
        "where", "while", "abstract", "become", "box", "do", "final", "macro", "override", "priv",
        "typeof", "unsized", "virtual", "yield", "async", "await", "try",
    ];
    if keywords.iter().any(|&keyword| keyword == s) {
        return quote::format_ident!("{}_", s);
        // return syn::Ident::new(&format!("{}_", s), Span::call_site());
    }

    quote::format_ident!("{}", s)
    // syn::Ident::new(&s, Span::call_site())
}
