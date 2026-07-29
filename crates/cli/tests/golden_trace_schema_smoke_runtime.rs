// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

use std::{error::Error, fs, path::Path};

use nautilus_model::{
    data::QuoteTick,
    identifiers::InstrumentId,
    types::{Price, Quantity},
};
use serde_json::{Value, json};

#[test]
fn rust_model_replays_schema_smoke_quote_envelope() -> Result<(), Box<dyn Error>> {
    let trace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/golden/schema_smoke.jsonl")
        .canonicalize()?;
    let rows = fs::read_to_string(&trace)?
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
        .map(serde_json::from_str::<Value>)
        .collect::<Result<Vec<_>, _>>()?;
    if rows.len() != 1 {
        return Err(format!("{} must contain exactly one case", trace.display()).into());
    }

    let case = &rows[0];
    if string_field(case, "case_id")? != "market_data.schema_smoke.001" {
        return Err("schema smoke trace contains an unexpected case".into());
    }
    let input = single_event(case, "input")?;
    let payload = object_field(input, "payload")?;
    let quote = QuoteTick::new(
        InstrumentId::from(string_field(input, "instrument_id")?),
        Price::from(string_field(payload, "bid")?),
        Price::from(string_field(payload, "ask")?),
        Quantity::from(string_field(payload, "bid_size")?),
        Quantity::from(string_field(payload, "ask_size")?),
        string_field(input, "ts_event")?.parse::<u64>()?.into(),
        string_field(input, "ts_init")?.parse::<u64>()?.into(),
    );
    let actual = json!({
        "event_type": string_field(input, "event_type")?,
        "ts_event": quote.ts_event.to_string(),
        "ts_init": quote.ts_init.to_string(),
        "instrument_id": quote.instrument_id.to_string(),
        "venue": quote.instrument_id.venue.to_string(),
        "payload": {
            "bid": quote.bid_price.to_string(),
            "ask": quote.ask_price.to_string(),
            "bid_size": quote.bid_size.to_string(),
            "ask_size": quote.ask_size.to_string(),
        }
    });
    let expected = single_event(case, "expected")?;
    if actual != *expected {
        return Err(format!(
            "Rust QuoteTick replay mismatch\nexpected={expected}\nactual={actual}"
        )
        .into());
    }

    Ok(())
}

fn single_event<'a>(case: &'a Value, section: &str) -> Result<&'a Value, Box<dyn Error>> {
    let events = case
        .get(section)
        .and_then(|value| value.get("events"))
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{section}.events must be an array"))?;
    if events.len() != 1 {
        return Err(format!("{section}.events must contain one event").into());
    }
    Ok(&events[0])
}

fn object_field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, Box<dyn Error>> {
    value
        .get(key)
        .filter(|field| field.is_object())
        .ok_or_else(|| format!("missing object field {key}").into())
}

fn string_field<'a>(value: &'a Value, key: &str) -> Result<&'a str, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field {key}").into())
}
