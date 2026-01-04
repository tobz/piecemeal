static RUST_KEYWORDS: [&str; 75] = [
    "abstract",
    "alignof",
    "as",
    "become",
    "bool",
    "box",
    "Box",
    "break",
    "BytesReader",
    "const",
    "continue",
    "crate",
    "Cow",
    "Default",
    "do",
    "else",
    "enum",
    "Err",
    "extern",
    "f32",
    "f64",
    "false",
    "final",
    "fn",
    "for",
    "HashMap",
    "i32",
    "i64",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "None",
    "MessageWrite",
    "offsetof",
    "Ok",
    "Option",
    "override",
    "priv",
    "pub",
    "pure",
    "ref",
    "Result",
    "return",
    "self",
    "Self",
    "sizeof",
    "Some",
    "static",
    "str",
    "String",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "typeof",
    "u8",
    "u32",
    "u64",
    "unsafe",
    "unsized",
    "use",
    "Vec",
    "virtual",
    "where",
    "while",
    "Write",
    "Writer",
    "yield",
];

/// Check if the identifier is a Rust keyword and appends an underscore suffix if that's the case
pub fn sanitize_keyword(ident: &mut String) {
    if !ident.contains('.') && RUST_KEYWORDS.contains(&&**ident) {
        ident.push('_');
    } else {
        *ident = ident
            .split('.')
            .map(|s| {
                if RUST_KEYWORDS.contains(&s) {
                    format!("{}_", s)
                } else {
                    s.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(".");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_simple_keyword() {
        let mut s = "type".to_string();
        sanitize_keyword(&mut s);
        assert_eq!(s, "type_");
    }

    #[test]
    fn test_sanitize_non_keyword() {
        let mut s = "field_name".to_string();
        sanitize_keyword(&mut s);
        assert_eq!(s, "field_name"); // Unchanged
    }

    #[test]
    fn test_sanitize_dotted_with_keyword() {
        let mut s = "foo.type.bar".to_string();
        sanitize_keyword(&mut s);
        assert_eq!(s, "foo.type_.bar");
    }

    #[test]
    fn test_sanitize_dotted_all_keywords() {
        let mut s = "self.match.impl".to_string();
        sanitize_keyword(&mut s);
        assert_eq!(s, "self_.match_.impl_");
    }

    #[test]
    fn test_sanitize_dotted_no_keywords() {
        let mut s = "foo.bar.baz".to_string();
        sanitize_keyword(&mut s);
        assert_eq!(s, "foo.bar.baz"); // Unchanged
    }

    #[test]
    fn test_rust_keywords_sample() {
        // Test a sampling of keywords from different categories
        let keywords = [
            "fn", "struct", "enum", "impl", "pub", "use", "mod", "self", "Self", "super", "crate",
            "where", "move", "ref", "mut", "const", "static", "type", "trait", "return",
        ];

        for kw in keywords {
            let mut s = kw.to_string();
            sanitize_keyword(&mut s);
            assert_eq!(
                s,
                format!("{}_", kw),
                "Keyword '{}' should be sanitized",
                kw
            );
        }
    }

    #[test]
    fn test_type_aliases_sanitized() {
        // These aren't keywords but are in RUST_KEYWORDS for safety
        let type_aliases = ["String", "Vec", "Option", "Result", "Box", "HashMap"];

        for alias in type_aliases {
            let mut s = alias.to_string();
            sanitize_keyword(&mut s);
            assert_eq!(s, format!("{}_", alias));
        }
    }

    #[test]
    fn test_primitives_sanitized() {
        let primitives = [
            "bool", "i32", "i64", "u8", "u32", "u64", "f32", "f64", "str",
        ];

        for prim in primitives {
            let mut s = prim.to_string();
            sanitize_keyword(&mut s);
            assert_eq!(s, format!("{}_", prim));
        }
    }

    #[test]
    fn test_empty_string() {
        let mut s = "".to_string();
        sanitize_keyword(&mut s);
        assert_eq!(s, "");
    }

    #[test]
    fn test_single_dot() {
        let mut s = ".".to_string();
        sanitize_keyword(&mut s);
        assert_eq!(s, ".");
    }
}
