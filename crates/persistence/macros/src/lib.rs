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

//! Procedural macros for Nautilus. Provides `#[custom_data]` for defining custom data types
//! with generated boilerplate (constructor, HasTsInit, CustomDataTrait, optional Arrow, derives).

mod custom;

use proc_macro::TokenStream;

/// Expands a struct into a custom data type with generated impls: `#[derive(Debug, Clone,
/// Serialize, Deserialize, PartialEq)]`, constructor, HasTsInit, CustomDataTrait,
/// ArrowSchemaProvider, EncodeToRecordBatch, DecodeDataFromRecordBatch unless `no_arrow` is set,
/// CatalogPathPrefix, From/TryFrom for Data. Call `nautilus_serialization::ensure_custom_data_registered::<T>()`
/// once per Arrow-enabled type, or `nautilus_model::data::ensure_custom_data_json_registered::<T>()`
/// for `no_arrow` types.
/// Requires fields to include `ts_event` and `ts_init` (e.g. `nautilus_core::UnixNanos`).
/// Supported field types include InstrumentId, AccountId, Currency, BarType, Params, UnixNanos,
/// f64, f32, bool, String, u64, i64, u32, i32, `Vec<f64>`, and `Vec<u8>`.
/// Use `#[custom_data_field(json)]` on a field to store any Serde serializable field as a
/// JSON-backed Arrow `Utf8` column.
///
/// Use `#[custom_data(no_display)]` to skip generating `repr()` and `Display` so you can
/// implement them manually. Use `#[custom_data(no_arrow)]` for live-only custom data that does
/// not need Arrow or catalog persistence.
#[proc_macro_attribute]
pub fn custom_data(attr: TokenStream, item: TokenStream) -> TokenStream {
    custom::expand_custom_data(attr.into(), item.into()).into()
}
