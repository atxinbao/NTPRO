use std::str::FromStr;

use nautilus_core::{StackStr, UUID4, UnixNanos};

#[test]
fn unix_nanos_parsing_and_json_roundtrip_cover_public_value_contract() {
    let expected = UnixNanos::new(1_704_067_200_000_000_000);

    assert_eq!(
        UnixNanos::from_str("1704067200000000000").unwrap(),
        expected
    );
    assert_eq!(
        UnixNanos::from_str("2024-01-01T00:00:00Z").unwrap(),
        expected
    );
    assert_eq!(UnixNanos::from_str("2024-01-01").unwrap(), expected);
    assert!(UnixNanos::from_str("-1").is_err());

    let json = serde_json::to_string(&expected).unwrap();
    let decoded: UnixNanos = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, expected);
}

#[test]
fn uuid4_string_json_and_version_validation_cover_public_value_contract() {
    let uuid = UUID4::from_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    assert_eq!(uuid.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    assert!(UUID4::from_str("550e8400-e29b-11d4-a716-446655440000").is_err());

    let json = serde_json::to_string(&uuid).unwrap();
    let decoded: UUID4 = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, uuid);
}

#[test]
fn stack_str_checked_constructor_and_json_roundtrip_cover_identifier_storage_contract() {
    let value = StackStr::new_checked("STRAT-001").unwrap();

    assert_eq!(value.as_str(), "STRAT-001");
    assert_eq!(value.len(), 9);
    assert!(StackStr::new_checked("").is_err());
    assert!(StackStr::new_checked("not-ascii-µ").is_err());

    let json = serde_json::to_string(&value).unwrap();
    let decoded: StackStr = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, value);
}
