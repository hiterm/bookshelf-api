use time::OffsetDateTime;

/// Normalizes a timestamp to the microsecond precision used by persistence.
pub fn normalize_timestamp_for_persistence(value: OffsetDateTime) -> OffsetDateTime {
    let nanosecond = value.nanosecond() / 1_000 * 1_000;
    value
        .replace_nanosecond(nanosecond)
        .expect("a truncated nanosecond is always valid")
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::normalize_timestamp_for_persistence;

    #[test]
    fn truncates_submicrosecond_precision() {
        let value = datetime!(2026-07-21 13:44:15.729823647 UTC);

        let actual = normalize_timestamp_for_persistence(value);

        assert_eq!(actual, datetime!(2026-07-21 13:44:15.729823 UTC));
    }

    #[test]
    fn preserves_microsecond_precision() {
        let value = datetime!(2026-07-21 13:44:15.729823 UTC);

        assert_eq!(normalize_timestamp_for_persistence(value), value);
    }
}
