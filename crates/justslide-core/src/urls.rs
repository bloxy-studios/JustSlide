//! Address-bar resolution and URL policy. Only http(s) (and about:blank for
//! the person) ever reach WebKit from JustSlide's own commands.

use crate::error::{CoreError, CoreResult};
use url::Url;

pub const MAX_URL: usize = 8192;

/// Validates an http(s) URL from any actor.
pub fn require_http(input: &str) -> CoreResult<String> {
    if input.len() > MAX_URL {
        return Err(CoreError::invalid("url too long"));
    }
    let u = Url::parse(input).map_err(|_| CoreError::invalid("url is not absolute"))?;
    match u.scheme() {
        "http" | "https" if u.host_str().is_some() => Ok(u.to_string()),
        s => Err(CoreError::invalid(format!("scheme {s}: is not allowed"))),
    }
}

/// Resolves what a person typed into the address bar: a URL, a bare host,
/// or a search query using the configured template.
pub fn resolve_user_input(input: &str, search_template: &str) -> CoreResult<String> {
    let t = input.trim();
    if t.is_empty() {
        return Err(CoreError::invalid("empty input"));
    }
    if t.len() > MAX_URL {
        return Err(CoreError::invalid("input too long"));
    }
    if t == "about:blank" {
        return Ok(t.to_string());
    }
    if let Ok(u) = Url::parse(t) {
        if matches!(u.scheme(), "http" | "https") {
            return require_http(t);
        }
        if !t.contains(' ') && t.contains("://") {
            return Err(CoreError::invalid(format!("scheme {}: is not allowed", u.scheme())));
        }
    }
    let looks_like_host = !t.contains(char::is_whitespace)
        && (t.contains('.') || t.starts_with("localhost"))
        && !t.starts_with('.')
        && !t.ends_with('.');
    if looks_like_host {
        let candidate = if t.starts_with("localhost") { format!("http://{t}") } else { format!("https://{t}") };
        if let Ok(u) = require_http(&candidate) {
            return Ok(u);
        }
    }
    let q: String = url::form_urlencoded::byte_serialize(t.as_bytes()).collect();
    require_http(&search_template.replace("{query}", &q))
}

/// Redacts a URL for logs/activity: origin + path, no query, fragment or
/// credentials.
pub fn redact(input: &str) -> String {
    match Url::parse(input) {
        Ok(mut u) => {
            let _ = u.set_username("");
            let _ = u.set_password(None);
            u.set_query(None);
            u.set_fragment(None);
            let mut s = u.to_string();
            s.truncate(256);
            s
        }
        Err(_) => "<invalid-url>".into(),
    }
}

pub fn host_of(input: &str) -> String {
    Url::parse(input).ok().and_then(|u| u.host_str().map(str::to_string)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    const S: &str = "https://duckduckgo.com/?q={query}";

    #[test]
    fn resolves_hosts_urls_and_searches() {
        assert_eq!(resolve_user_input("figma.com", S).unwrap(), "https://figma.com/");
        assert_eq!(resolve_user_input("https://example.com/a?b=1", S).unwrap(), "https://example.com/a?b=1");
        assert_eq!(resolve_user_input("localhost:8080", S).unwrap(), "http://localhost:8080/");
        assert_eq!(resolve_user_input("rust ownership rules", S).unwrap(), "https://duckduckgo.com/?q=rust+ownership+rules");
        assert_eq!(resolve_user_input("about:blank", S).unwrap(), "about:blank");
    }

    #[test]
    fn rejects_dangerous_schemes() {
        for bad in ["javascript:alert(1)", "file:///etc/passwd", "data:text/html,hi", "justslide://settings"] {
            assert!(require_http(bad).is_err(), "{bad}");
        }
        assert!(resolve_user_input("file:///etc/passwd", S).is_err());
        assert!(resolve_user_input("justslide://settings", S).is_err());
        // javascript: without // is treated as a search, never executed
        assert!(resolve_user_input("javascript:alert(1)", S).unwrap().starts_with("https://duckduckgo.com/"));
    }

    #[test]
    fn redaction_strips_secrets() {
        assert_eq!(redact("https://user:pw@mail.example.com/inbox?token=abc#x"), "https://mail.example.com/inbox");
    }
}
