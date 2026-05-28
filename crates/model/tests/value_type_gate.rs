use std::str::FromStr;

use nautilus_model::{
    identifiers::{ClientOrderId, InstrumentId, StrategyId},
    types::{Currency, Money, PRICE_UNDEF, Price, QUANTITY_UNDEF, Quantity},
};
use rust_decimal_macros::dec;

fn assert_invalid_raw_precision_error(error: &impl ToString) {
    assert!(error.to_string().contains("Invalid fixed-point raw value"));
}

#[test]
fn price_quantity_and_money_preserve_precision_through_string_and_json_roundtrips() {
    let price = Price::from_str("123.4500").unwrap();
    assert_eq!(price.precision, 4);
    assert_eq!(price.as_decimal(), dec!(123.4500));
    assert_eq!(price.to_string(), "123.4500");
    let decoded_price: Price =
        serde_json::from_str(&serde_json::to_string(&price).unwrap()).unwrap();
    assert_eq!(decoded_price.raw, price.raw);
    assert_eq!(decoded_price.precision, price.precision);

    let quantity = Quantity::from_str("10.250").unwrap();
    assert_eq!(quantity.precision, 3);
    assert_eq!(quantity.as_decimal(), dec!(10.250));
    assert_eq!(quantity.to_string(), "10.250");
    let decoded_quantity: Quantity =
        serde_json::from_str(&serde_json::to_string(&quantity).unwrap()).unwrap();
    assert_eq!(decoded_quantity.raw, quantity.raw);
    assert_eq!(decoded_quantity.precision, quantity.precision);

    let money = Money::from_str("99.95 USD").unwrap();
    assert_eq!(money.currency, Currency::USD());
    assert_eq!(money.as_decimal(), dec!(99.95));
    assert_eq!(money.to_string(), "99.95 USD");
    let decoded_money: Money =
        serde_json::from_str(&serde_json::to_string(&money).unwrap()).unwrap();
    assert_eq!(decoded_money.raw, money.raw);
    assert_eq!(decoded_money.currency, money.currency);
}

#[test]
fn raw_precision_mismatches_are_rejected_by_checked_constructors() {
    let price_error = Price::from_raw_checked(PRICE_UNDEF, 1).unwrap_err();
    assert!(
        price_error
            .to_string()
            .contains("`precision` must be 0 when `raw` is PRICE_UNDEF")
    );

    let quantity_error = Quantity::from_raw_checked(QUANTITY_UNDEF, 1).unwrap_err();
    assert!(
        quantity_error
            .to_string()
            .contains("`precision` must be 0 when `raw` is QUANTITY_UNDEF")
    );

    assert_invalid_raw_precision_error(&Price::from_raw_checked(1, 0).unwrap_err());
    assert_invalid_raw_precision_error(&Quantity::from_raw_checked(1, 0).unwrap_err());
    assert_invalid_raw_precision_error(&Money::from_raw_checked(1, Currency::USD()).unwrap_err());
}

#[test]
fn identifier_value_types_keep_parse_display_and_json_contracts() {
    let instrument_id = InstrumentId::from_str("ETH/USDT.BINANCE").unwrap();
    assert_eq!(instrument_id.symbol.as_str(), "ETH/USDT");
    assert_eq!(instrument_id.venue.as_str(), "BINANCE");
    assert_eq!(instrument_id.to_string(), "ETH/USDT.BINANCE");
    let decoded_instrument_id: InstrumentId =
        serde_json::from_str(&serde_json::to_string(&instrument_id).unwrap()).unwrap();
    assert_eq!(decoded_instrument_id, instrument_id);

    let client_order_id = ClientOrderId::new("O-19700101-000000-001-001-1");
    let decoded_client_order_id: ClientOrderId =
        serde_json::from_str(&serde_json::to_string(&client_order_id).unwrap()).unwrap();
    assert_eq!(decoded_client_order_id, client_order_id);

    let strategy_id = StrategyId::new_checked("EMACross-001").unwrap();
    assert_eq!(strategy_id.get_tag(), "001");
    assert!(StrategyId::new_checked("missingtag").is_err());
}

#[cfg(feature = "defi")]
#[test]
fn defi_quantity_wei_roundtrip_covers_18_decimal_precision_boundary() {
    use alloy_primitives::U256;
    use nautilus_model::types::fixed::MAX_FLOAT_PRECISION;

    let wei = U256::from(1_000_000_000_000_000_000_u128);
    let quantity = Quantity::from_wei(wei);

    assert_eq!(quantity.precision, 18);
    assert_eq!(quantity.as_wei(), wei);
    assert!(quantity.precision > MAX_FLOAT_PRECISION);
    assert_eq!(quantity.to_string(), "1000000000000000000");
}
