#![allow(missing_docs)]

use envbind::{Binder, MapEnvironment, StringVar, validators};

#[test]
fn optional_scheme_still_rejects_disallowed_explicit_schemes() {
    let validate = validators::is_url_with_options(false, ["https"]);
    let inputs = [
        "http:example.com",
        "HtTp:example.com",
        "javascript:alert(1)",
        "data:text/plain,hello",
        "ftp://example.com",
    ];
    let accepted: Vec<_> = inputs
        .into_iter()
        .filter(|value| validate(value).is_ok())
        .collect();

    assert!(
        accepted.is_empty(),
        "accepted disallowed schemes: {accepted:?}"
    );
}

#[test]
fn required_scheme_rejects_disallowed_explicit_schemes() {
    let validate = validators::is_url_with_options(true, ["https"]);
    let inputs = [
        "http:example.com",
        "javascript:alert(1)",
        "data:text/plain,hello",
    ];
    let accepted: Vec<_> = inputs
        .into_iter()
        .filter(|value| validate(value).is_ok())
        .collect();

    assert!(
        accepted.is_empty(),
        "accepted disallowed schemes: {accepted:?}"
    );
}

#[test]
fn url_validator_rejects_missing_hosts() {
    let inputs = [
        "https://",
        "https://:443",
        "https://user@",
        "https://?query",
        "https://#fragment",
    ];
    let accepted: Vec<_> = inputs
        .into_iter()
        .filter(|value| validators::is_url()(value).is_ok())
        .collect();

    assert!(accepted.is_empty(), "accepted missing hosts: {accepted:?}");
}

#[test]
fn url_validator_rejects_invalid_ports() {
    let inputs = [
        "https://example.com:65536",
        "https://example.com:99999",
        "https://example.com:-1",
        "https://example.com:abc",
    ];
    let accepted: Vec<_> = inputs
        .into_iter()
        .filter(|value| validators::is_url()(value).is_ok())
        .collect();

    assert!(accepted.is_empty(), "accepted invalid ports: {accepted:?}");
}

#[test]
fn url_validator_rejects_malformed_ipv6_hosts() {
    let inputs = [
        "https://[invalid]",
        "https://[::1",
        "https://::1/",
        "https://[::1]extra",
    ];
    let accepted: Vec<_> = inputs
        .into_iter()
        .filter(|value| validators::is_url()(value).is_ok())
        .collect();

    assert!(accepted.is_empty(), "accepted malformed IPv6: {accepted:?}");
}

#[test]
fn url_validator_accepts_supported_hosts_and_credentials() {
    let inputs = [
        "http://localhost",
        "https://example.com/path?query=value#fragment",
        "https://127.0.0.1:443",
        "https://[::1]:8443/path",
        "https://bücher.example",
        "https://user:synthetic-password@example.com:443/path",
        "https://user:p%40ss@example.com",
        "HTTPS://EXAMPLE.COM",
    ];
    let rejected: Vec<_> = inputs
        .into_iter()
        .filter(|value| validators::is_url()(value).is_err())
        .collect();

    assert!(rejected.is_empty(), "rejected valid URLs: {rejected:?}");
}

#[test]
fn url_validator_accepts_numeric_port_boundaries() {
    let inputs = ["https://example.com:0", "https://example.com:65535"];
    let rejected: Vec<_> = inputs
        .into_iter()
        .filter(|value| validators::is_url()(value).is_err())
        .collect();

    assert!(rejected.is_empty(), "rejected valid ports: {rejected:?}");
}

#[test]
fn url_validator_checks_the_parsed_scheme_without_requiring_slashes() {
    let result = validators::is_url_with_options(true, ["HTTPS"])("hTtPs:example.com");

    assert_eq!(result, Ok(()));
}

#[test]
fn url_validator_accepts_configured_custom_authorities() {
    let validate = validators::is_url_with_options(true, ["POSTGRES"]);
    let inputs = [
        "postgres://db.example.com/service",
        "PoStGrEs://user:synthetic-password@[::1]:5432/db",
    ];
    let rejected: Vec<_> = inputs
        .into_iter()
        .filter(|value| validate(value).is_err())
        .collect();

    assert!(rejected.is_empty(), "rejected custom URLs: {rejected:?}");
}

#[test]
fn url_validator_rejects_invalid_custom_authorities() {
    let validate = validators::is_url_with_options(true, ["postgres"]);
    let inputs = [
        "postgres://user@",
        "postgres://:5432",
        "postgres://[invalid]/db",
        "postgres://host:65536/db",
    ];
    let accepted: Vec<_> = inputs
        .into_iter()
        .filter(|value| validate(value).is_ok())
        .collect();

    assert!(
        accepted.is_empty(),
        "accepted invalid custom URLs: {accepted:?}"
    );
}

#[test]
fn url_validator_requires_hosts_even_for_allowed_schemes() {
    let validate = validators::is_url_with_options(true, ["mailto", "file", "postgres"]);
    let inputs = [
        "mailto:user@example.com",
        "file:///tmp/config",
        "postgres:database",
    ];
    let accepted: Vec<_> = inputs
        .into_iter()
        .filter(|value| validate(value).is_ok())
        .collect();

    assert!(accepted.is_empty(), "accepted hostless URLs: {accepted:?}");
}

#[test]
fn optional_scheme_accepts_bare_hosts_with_paths() {
    let validate = validators::is_url_with_options(false, ["https"]);
    let inputs = [
        "localhost",
        "example.com",
        "127.0.0.1/path",
        "bücher.example/path",
        "example.com/path:part?redirect=https://other.example#fragment",
    ];
    let rejected: Vec<_> = inputs
        .into_iter()
        .filter(|value| validate(value).is_err())
        .collect();

    assert!(rejected.is_empty(), "rejected bare hosts: {rejected:?}");
}

#[test]
fn optional_scheme_accepts_explicit_network_authorities() {
    let validate = validators::is_url_with_options(false, ["https"]);
    let inputs = [
        "//localhost:8080",
        "//example.com:443/path",
        "//127.0.0.1:443",
        "//[::1]:65535/path",
        "//user:synthetic-password@example.com:443/path",
    ];
    let rejected: Vec<_> = inputs
        .into_iter()
        .filter(|value| validate(value).is_err())
        .collect();

    assert!(
        rejected.is_empty(),
        "rejected network authorities: {rejected:?}"
    );
}

#[test]
fn optional_scheme_accepts_allowed_absolute_urls() {
    let result =
        validators::is_url_with_options(false, ["postgres"])("postgres://db.example.com:5432/db");

    assert_eq!(result, Ok(()));
}

#[test]
fn required_scheme_rejects_schemeless_inputs() {
    let inputs = [
        "example.com",
        "//example.com:443",
        "//[::1]:443",
        "127.0.0.1/path",
    ];
    let accepted: Vec<_> = inputs
        .into_iter()
        .filter(|value| validators::is_url()(value).is_ok())
        .collect();

    assert!(
        accepted.is_empty(),
        "accepted missing schemes: {accepted:?}"
    );
}

#[test]
fn optional_scheme_requires_prefix_for_ports_and_credentials() {
    let validate = validators::is_url_with_options(false, ["https"]);
    let inputs = [
        "localhost:443",
        "example.com:443",
        "127.0.0.1:443",
        "[::1]:443",
        "user@example.com",
        "user:password@example.com",
    ];
    let accepted: Vec<_> = inputs
        .into_iter()
        .filter(|value| validate(value).is_ok())
        .collect();

    assert!(
        accepted.is_empty(),
        "accepted ambiguous authorities: {accepted:?}"
    );
}

#[test]
fn optional_scheme_rejects_empty_or_malformed_authorities() {
    let validate = validators::is_url_with_options(false, ["https"]);
    let inputs = [
        "",
        "//",
        "///example.com",
        "////example.com",
        "//:443",
        "//user@",
        "//host:abc",
        "//[invalid]",
    ];
    let accepted: Vec<_> = inputs
        .into_iter()
        .filter(|value| validate(value).is_ok())
        .collect();

    assert!(
        accepted.is_empty(),
        "accepted malformed authorities: {accepted:?}"
    );
}

#[test]
fn optional_scheme_rejects_relative_path_references() {
    let validate = validators::is_url_with_options(false, ["https"]);
    let inputs = ["/path", "./path", "../path", "?query", "#fragment"];
    let accepted: Vec<_> = inputs
        .into_iter()
        .filter(|value| validate(value).is_ok())
        .collect();

    assert!(
        accepted.is_empty(),
        "accepted relative references: {accepted:?}"
    );
}

#[test]
fn url_validator_rejects_raw_whitespace_controls_and_backslashes() {
    let inputs = [
        " https://example.com",
        "https://example.com ",
        "https://example.com/a b",
        "https://example.com/\tpath",
        "https://example.com/\npath",
        "https://example.com/\0path",
        "https://example.com/\u{7f}path",
        "https://example.com/\u{a0}path",
        r"https://example.com\path",
        r"\\example.com",
    ];
    let accepted: Vec<_> = inputs
        .into_iter()
        .filter(|value| validators::is_url_with_options(false, ["https"])(value).is_ok())
        .collect();

    assert!(
        accepted.is_empty(),
        "accepted invalid characters: {accepted:?}"
    );
}

#[test]
fn url_validator_accepts_percent_encoded_whitespace() {
    let result = validators::is_url()("https://example.com/a%20b");

    assert_eq!(result, Ok(()));
}

#[test]
fn optional_scheme_does_not_apply_allowlist_to_synthetic_scheme() {
    let result = validators::is_url_with_options(false, Vec::<String>::new())("//localhost:8080");

    assert_eq!(result, Ok(()));
}

#[test]
fn empty_allowlist_rejects_explicit_schemes() {
    let result =
        validators::is_url_with_options(false, Vec::<String>::new())("https://example.com");

    assert!(result.is_err());
}

#[test]
fn url_parse_errors_do_not_expose_credentials() {
    let result =
        validators::is_url()("https://user:synthetic-password@[invalid]/").map_err(|error| {
            [
                error.to_string(),
                format!("{error:?}"),
                format!("{error:#?}"),
            ]
            .iter()
            .all(|output| !output.contains("synthetic-password"))
        });

    assert_eq!(result, Err(true));
}

#[test]
fn non_sensitive_url_validation_errors_keep_credentials_private() {
    let result = Binder::new(MapEnvironment::from_pairs([(
        "API_URL",
        "https://user:synthetic-password@",
    )]))
    .bind(
        &StringVar::new("API_URL")
            .sensitive(false)
            .validate(validators::is_url()),
    )
    .map_err(|error| {
        (
            error.error_code(),
            error.variable_name().to_owned(),
            [
                error.to_string(),
                format!("{error:?}"),
                format!("{error:#?}"),
            ]
            .iter()
            .all(|output| !output.contains("synthetic-password")),
        )
    });

    assert_eq!(
        result,
        Err(("validation_failed", "API_URL".to_owned(), true))
    );
}

#[test]
fn url_validation_preserves_the_original_bound_string() {
    let raw = "HTTPS://EXAMPLE.COM:443/path";
    let result = Binder::new(MapEnvironment::from_pairs([("API_URL", raw)]))
        .bind(&StringVar::new("API_URL").validate(validators::is_url()));

    assert_eq!(result, Ok(raw.to_owned()));
}
