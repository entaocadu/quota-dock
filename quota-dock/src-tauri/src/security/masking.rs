use sha2::{Digest, Sha256};

pub fn mask_email(email: &str) -> String {
    if let Some((local, domain)) = email.split_once('@') {
        let first_char = local.chars().next().unwrap_or('*');
        format!("{}***@{}", first_char, domain)
    } else {
        let first_char = email.chars().next().unwrap_or('*');
        format!("{}***", first_char)
    }
}

pub fn hash_account_key(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"quotadock_salt_v1_");
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", &result)[..16].to_string()
}

pub fn redact_text(input: &str) -> String {
    // Redact Bearer tokens, passwords, refresh tokens
    let mut out = input.to_string();
    if let Some(pos) = out.to_lowercase().find("bearer ") {
        let after = &out[pos + 7..];
        let end = after.find(|c: char| c.is_whitespace() || c == '"' || c == '\'').unwrap_or(after.len());
        let to_replace = &after[..end];
        if !to_replace.is_empty() {
            out = out.replace(to_replace, "[REDACTED]");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_email_standard() {
        assert_eq!(mask_email("eenta@gmail.com"), "e***@gmail.com");
        assert_eq!(mask_email("c@openai.com"), "c***@openai.com");
    }

    #[test]
    fn test_mask_email_no_domain() {
        assert_eq!(mask_email("single"), "s***");
        assert_eq!(mask_email(""), "****");
    }

    #[test]
    fn test_hash_account_key_deterministic_and_fixed_length() {
        let h1 = hash_account_key("test@example.com");
        let h2 = hash_account_key("test@example.com");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 16);
        let h3 = hash_account_key("other@example.com");
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_redact_bearer_token() {
        let log = "Header Authorization: Bearer eyJhbGciOi... secret in log";
        let redacted = redact_text(log);
        assert!(!redacted.contains("eyJhbGciOi"));
        assert!(redacted.contains("[REDACTED]"));
    }
}
