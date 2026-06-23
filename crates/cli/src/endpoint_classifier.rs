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

use url::Url;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EndpointAuthKind {
    None,
    Signed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EndpointClass {
    SandboxSpotDemo,
    SandboxSpotTestNetwork,
    ProductionPublicReadOnly,
    ProductionAuthenticatedReadOnly,
    ProductionOrderStateReadOnly,
    ProductionMutationScopeCandidate,
    ProductionMutationOwnerApprovedManualOnly,
    ProductionMutationForbidden,
    WebsocketPublicReadOnly,
    WebsocketUserReadOnly,
    UnknownForbidden,
}

impl EndpointClass {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::SandboxSpotDemo => "sandbox_spot_demo",
            Self::SandboxSpotTestNetwork => "sandbox_spot_test_network",
            Self::ProductionPublicReadOnly => "production_public_read_only",
            Self::ProductionAuthenticatedReadOnly => "production_authenticated_read_only",
            Self::ProductionOrderStateReadOnly => "production_order_state_read_only",
            Self::ProductionMutationScopeCandidate => "production_mutation_scope_candidate",
            Self::ProductionMutationOwnerApprovedManualOnly => {
                "production_mutation_owner_approved_manual_only"
            }
            Self::ProductionMutationForbidden => "production_mutation_forbidden",
            Self::WebsocketPublicReadOnly => "websocket_public_read_only",
            Self::WebsocketUserReadOnly => "websocket_user_read_only",
            Self::UnknownForbidden => "unknown_forbidden",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EndpointDecision {
    AllowReadOnly,
    AllowRequestPreviewOnly,
    Deny,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ClassifiedEndpoint {
    pub(crate) input_url_redacted: String,
    pub(crate) method: String,
    pub(crate) host_class: String,
    pub(crate) endpoint_class: EndpointClass,
    pub(crate) requires_signature: bool,
    pub(crate) requires_api_key: bool,
    pub(crate) mutation_allowed: bool,
    pub(crate) read_allowed: bool,
    pub(crate) request_preview_allowed: bool,
    pub(crate) owner_gate_required: bool,
    pub(crate) dashboard_order_controls_allowed: bool,
    pub(crate) decision: EndpointDecision,
    pub(crate) reason: String,
    pub(crate) path: String,
}

#[derive(Debug)]
pub(crate) struct EndpointClassifier;

impl EndpointClassifier {
    pub(crate) fn classify(
        method: &str,
        url: &str,
        auth_kind: EndpointAuthKind,
    ) -> ClassifiedEndpoint {
        Self::classify_with_context(method, url, auth_kind, false)
    }

    pub(crate) fn classify_with_context(
        method: &str,
        url: &str,
        auth_kind: EndpointAuthKind,
        owner_manual_scope: bool,
    ) -> ClassifiedEndpoint {
        classify_endpoint_with_context(method, url, auth_kind, owner_manual_scope)
    }
}

fn classify_endpoint_with_context(
    method: &str,
    url: &str,
    auth_kind: EndpointAuthKind,
    owner_manual_scope: bool,
) -> ClassifiedEndpoint {
    let normalized_method = method.trim().to_ascii_uppercase();
    let Ok(parsed_url) = Url::parse(url) else {
        return unknown_forbidden(
            url,
            normalized_method,
            "endpoint URL cannot be parsed by the classifier",
        );
    };

    let input_url_redacted = redact_url(&parsed_url);
    let scheme = parsed_url.scheme();
    let Some(host) = parsed_url.host_str() else {
        return unknown_forbidden(
            &input_url_redacted,
            normalized_method,
            "endpoint URL has no host",
        );
    };

    let path = parsed_url.path().to_string();
    match scheme {
        "https" => classify_rest_endpoint_with_context(
            input_url_redacted,
            normalized_method,
            host,
            path,
            auth_kind,
            owner_manual_scope,
        ),
        "wss" => classify_websocket_endpoint(
            input_url_redacted,
            normalized_method,
            host,
            path,
            auth_kind,
        ),
        _ => unknown_forbidden(
            &input_url_redacted,
            normalized_method,
            "endpoint scheme is outside the v0.11 HTTPS/WSS contract",
        ),
    }
}

fn classify_rest_endpoint_with_context(
    input_url_redacted: String,
    method: String,
    host: &str,
    path: String,
    auth_kind: EndpointAuthKind,
    owner_manual_scope: bool,
) -> ClassifiedEndpoint {
    match host {
        "demo-api.binance.com" => sandbox_endpoint(
            input_url_redacted,
            method,
            "sandbox",
            EndpointClass::SandboxSpotDemo,
            path,
        ),
        "testnet.binance.vision" => sandbox_endpoint(
            input_url_redacted,
            method,
            "testnet",
            EndpointClass::SandboxSpotTestNetwork,
            path,
        ),
        "api.binance.com" => classify_production_rest_endpoint(
            input_url_redacted,
            method,
            path,
            auth_kind,
            owner_manual_scope,
        ),
        _ => unknown_forbidden(
            &input_url_redacted,
            method,
            "endpoint host is not in the v0.11 endpoint allow list",
        ),
    }
}

fn classify_production_rest_endpoint(
    input_url_redacted: String,
    method: String,
    path: String,
    auth_kind: EndpointAuthKind,
    owner_manual_scope: bool,
) -> ClassifiedEndpoint {
    if is_production_order_state_readonly_endpoint(&method, &path) {
        let signed = auth_kind == EndpointAuthKind::Signed;
        return classified_endpoint(ClassifiedEndpointInput {
            input_url_redacted,
            method,
            host_class: "production",
            endpoint_class: EndpointClass::ProductionOrderStateReadOnly,
            requires_signature: true,
            requires_api_key: true,
            mutation_allowed: false,
            read_allowed: signed,
            request_preview_allowed: false,
            owner_gate_required: true,
            dashboard_order_controls_allowed: false,
            decision: if signed {
                EndpointDecision::AllowReadOnly
            } else {
                EndpointDecision::Deny
            },
            reason: if signed {
                "owner-approved production order-state read-only endpoint"
            } else {
                "production order-state read-only endpoint requires signed credentials"
            },
            path,
        });
    }

    if is_production_mutation_request_preview_candidate(&method, &path) {
        return classify_production_mutation_scope_candidate(
            input_url_redacted,
            method,
            path,
            auth_kind,
            owner_manual_scope,
        );
    }

    if is_production_order_or_mutation_endpoint(&method, &path) {
        return classified_endpoint(ClassifiedEndpointInput {
            input_url_redacted,
            method,
            host_class: "production",
            endpoint_class: EndpointClass::ProductionMutationForbidden,
            requires_signature: auth_kind == EndpointAuthKind::Signed,
            requires_api_key: auth_kind != EndpointAuthKind::None,
            mutation_allowed: false,
            read_allowed: false,
            request_preview_allowed: false,
            owner_gate_required: true,
            dashboard_order_controls_allowed: false,
            decision: EndpointDecision::Deny,
            reason: "production order mutation is out of scope except explicit owner-gated mutation-candidate flows",
            path,
        });
    }

    if method == "GET" && matches!(path.as_str(), "/api/v3/time" | "/api/v3/exchangeInfo") {
        return classified_endpoint(ClassifiedEndpointInput {
            input_url_redacted,
            method,
            host_class: "production",
            endpoint_class: EndpointClass::ProductionPublicReadOnly,
            requires_signature: false,
            requires_api_key: false,
            mutation_allowed: false,
            read_allowed: true,
            request_preview_allowed: false,
            owner_gate_required: false,
            dashboard_order_controls_allowed: false,
            decision: EndpointDecision::AllowReadOnly,
            reason: "public production read-only endpoint",
            path,
        });
    }

    if method == "GET" && path == "/api/v3/account" {
        let signed = auth_kind == EndpointAuthKind::Signed;
        return classified_endpoint(ClassifiedEndpointInput {
            input_url_redacted,
            method,
            host_class: "production",
            endpoint_class: EndpointClass::ProductionAuthenticatedReadOnly,
            requires_signature: true,
            requires_api_key: true,
            mutation_allowed: false,
            read_allowed: signed,
            request_preview_allowed: false,
            owner_gate_required: true,
            dashboard_order_controls_allowed: false,
            decision: if signed {
                EndpointDecision::AllowReadOnly
            } else {
                EndpointDecision::Deny
            },
            reason: if signed {
                "owner-approved production authenticated read-only endpoint"
            } else {
                "production authenticated read-only endpoint requires signed credentials"
            },
            path,
        });
    }

    unknown_forbidden(
        &input_url_redacted,
        method,
        "production endpoint path is not in the v0.11 read-only contract",
    )
}

fn classify_production_mutation_scope_candidate(
    input_url_redacted: String,
    method: String,
    path: String,
    auth_kind: EndpointAuthKind,
    owner_manual_scope: bool,
) -> ClassifiedEndpoint {
    let signed = auth_kind == EndpointAuthKind::Signed;
    let preview_allowed = owner_manual_scope && signed;
    classified_endpoint(ClassifiedEndpointInput {
        input_url_redacted,
        method,
        host_class: "production",
        endpoint_class: if preview_allowed {
            EndpointClass::ProductionMutationOwnerApprovedManualOnly
        } else {
            EndpointClass::ProductionMutationScopeCandidate
        },
        requires_signature: true,
        requires_api_key: true,
        mutation_allowed: false,
        read_allowed: false,
        request_preview_allowed: preview_allowed,
        owner_gate_required: true,
        dashboard_order_controls_allowed: false,
        decision: if preview_allowed {
            EndpointDecision::AllowRequestPreviewOnly
        } else {
            EndpointDecision::Deny
        },
        reason: if preview_allowed {
            "owner-approved manual dry-run request preview only; production request execution remains forbidden"
        } else if signed {
            "production mutation endpoint is an owner-gated mutation-candidate scope; owner/manual scope is required"
        } else {
            "production mutation request preview requires signed owner/manual scope"
        },
        path,
    })
}

fn classify_websocket_endpoint(
    input_url_redacted: String,
    method: String,
    host: &str,
    path: String,
    auth_kind: EndpointAuthKind,
) -> ClassifiedEndpoint {
    let host_class = match host {
        "stream.binance.com" | "stream.testnet.binance.vision" | "data-stream.binance.vision" => {
            "websocket"
        }
        _ => {
            return unknown_forbidden(
                &input_url_redacted,
                method,
                "websocket host is not in the v0.11 endpoint allow list",
            );
        }
    };

    let user_stream = auth_kind == EndpointAuthKind::Signed;
    let decision = if user_stream {
        EndpointDecision::Deny
    } else {
        EndpointDecision::AllowReadOnly
    };
    classified_endpoint(ClassifiedEndpointInput {
        input_url_redacted,
        method,
        host_class,
        endpoint_class: if user_stream {
            EndpointClass::WebsocketUserReadOnly
        } else {
            EndpointClass::WebsocketPublicReadOnly
        },
        requires_signature: false,
        requires_api_key: user_stream,
        mutation_allowed: false,
        read_allowed: !user_stream,
        request_preview_allowed: false,
        owner_gate_required: user_stream,
        dashboard_order_controls_allowed: false,
        decision,
        reason: if user_stream {
            "websocket user stream requires listenKey lifecycle and is deferred for v0.12"
        } else {
            "websocket public stream is classified as read-only evidence only"
        },
        path,
    })
}

fn sandbox_endpoint(
    input_url_redacted: String,
    method: String,
    host_class: &'static str,
    endpoint_class: EndpointClass,
    path: String,
) -> ClassifiedEndpoint {
    let read_only = method == "GET";
    classified_endpoint(ClassifiedEndpointInput {
        input_url_redacted,
        method,
        host_class,
        endpoint_class,
        requires_signature: false,
        requires_api_key: false,
        mutation_allowed: false,
        read_allowed: read_only,
        request_preview_allowed: false,
        owner_gate_required: false,
        dashboard_order_controls_allowed: false,
        decision: if read_only {
            EndpointDecision::AllowReadOnly
        } else {
            EndpointDecision::Deny
        },
        reason: if read_only {
            "sandbox/testnet REST read-only endpoint"
        } else {
            "sandbox/testnet mutation is not authorized by the central read-only classifier"
        },
        path,
    })
}

fn is_production_order_state_readonly_endpoint(method: &str, path: &str) -> bool {
    method == "GET" && matches!(path, "/api/v3/openOrders" | "/api/v3/order")
}

fn is_production_mutation_request_preview_candidate(method: &str, path: &str) -> bool {
    method == "POST" && path == "/api/v3/order"
}

fn is_production_order_or_mutation_endpoint(method: &str, path: &str) -> bool {
    if matches!(method, "POST" | "PUT" | "PATCH" | "DELETE") {
        return true;
    }

    matches!(
        path,
        "/api/v3/order"
            | "/api/v3/order/test"
            | "/api/v3/openOrders"
            | "/api/v3/allOrders"
            | "/api/v3/orderList"
            | "/api/v3/openOrderList"
            | "/api/v3/allOrderList"
    )
}

fn unknown_forbidden(input_url: &str, method: String, reason: &'static str) -> ClassifiedEndpoint {
    classified_endpoint(ClassifiedEndpointInput {
        input_url_redacted: redact_raw_url(input_url),
        method,
        host_class: "unknown",
        endpoint_class: EndpointClass::UnknownForbidden,
        requires_signature: false,
        requires_api_key: false,
        mutation_allowed: false,
        read_allowed: false,
        request_preview_allowed: false,
        owner_gate_required: false,
        dashboard_order_controls_allowed: false,
        decision: EndpointDecision::Deny,
        reason,
        path: String::new(),
    })
}

fn redact_url(url: &Url) -> String {
    let mut redacted = url.clone();
    let had_query = redacted.query().is_some();
    redacted.set_query(None);
    let _ = redacted.set_username("");
    let _ = redacted.set_password(None);
    let base_url = redacted.to_string();
    if had_query {
        format!("{base_url}?<redacted>")
    } else {
        base_url
    }
}

fn redact_raw_url(url: &str) -> String {
    url.split_once('?').map_or_else(
        || url.to_string(),
        |(prefix, _)| format!("{prefix}?<redacted>"),
    )
}

#[derive(Debug)]
struct ClassifiedEndpointInput<'a> {
    input_url_redacted: String,
    method: String,
    host_class: &'a str,
    endpoint_class: EndpointClass,
    requires_signature: bool,
    requires_api_key: bool,
    mutation_allowed: bool,
    read_allowed: bool,
    request_preview_allowed: bool,
    owner_gate_required: bool,
    dashboard_order_controls_allowed: bool,
    decision: EndpointDecision,
    reason: &'a str,
    path: String,
}

fn classified_endpoint(input: ClassifiedEndpointInput<'_>) -> ClassifiedEndpoint {
    ClassifiedEndpoint {
        input_url_redacted: input.input_url_redacted,
        method: input.method,
        host_class: input.host_class.to_string(),
        endpoint_class: input.endpoint_class,
        requires_signature: input.requires_signature,
        requires_api_key: input.requires_api_key,
        mutation_allowed: input.mutation_allowed,
        read_allowed: input.read_allowed,
        request_preview_allowed: input.request_preview_allowed,
        owner_gate_required: input.owner_gate_required,
        dashboard_order_controls_allowed: input.dashboard_order_controls_allowed,
        decision: input.decision,
        reason: input.reason.to_string(),
        path: input.path,
    }
}

#[cfg(test)]
mod tests {
    use super::{EndpointAuthKind, EndpointClass, EndpointClassifier, EndpointDecision};

    #[test]
    fn endpoint_classifier_allows_production_public_read_only_endpoint() {
        let endpoint = EndpointClassifier::classify(
            "GET",
            "https://api.binance.com/api/v3/time",
            EndpointAuthKind::None,
        );

        assert_eq!(endpoint.host_class, "production");
        assert_eq!(
            endpoint.endpoint_class,
            EndpointClass::ProductionPublicReadOnly
        );
        assert_eq!(
            endpoint.endpoint_class.as_str(),
            "production_public_read_only"
        );
        assert_eq!(endpoint.decision, EndpointDecision::AllowReadOnly);
        assert_eq!(endpoint.method, "GET");
        assert_eq!(endpoint.path, "/api/v3/time");
        assert!(endpoint.read_allowed);
        assert!(!endpoint.mutation_allowed);
        assert!(!endpoint.requires_api_key);
        assert!(!endpoint.requires_signature);
        assert!(!endpoint.dashboard_order_controls_allowed);
    }

    #[test]
    fn endpoint_classifier_allows_signed_account_read_only_endpoint() {
        let endpoint = EndpointClassifier::classify(
            "get",
            "https://api.binance.com/api/v3/account?timestamp=123&signature=secret",
            EndpointAuthKind::Signed,
        );

        assert_eq!(
            endpoint.endpoint_class,
            EndpointClass::ProductionAuthenticatedReadOnly
        );
        assert_eq!(
            endpoint.endpoint_class.as_str(),
            "production_authenticated_read_only"
        );
        assert_eq!(endpoint.decision, EndpointDecision::AllowReadOnly);
        assert!(endpoint.read_allowed);
        assert!(!endpoint.mutation_allowed);
        assert!(endpoint.requires_api_key);
        assert!(endpoint.requires_signature);
        assert!(endpoint.owner_gate_required);
        assert_eq!(
            endpoint.input_url_redacted,
            "https://api.binance.com/api/v3/account?<redacted>"
        );
    }

    #[test]
    fn endpoint_classifier_denies_unsigned_account_read_only_endpoint() {
        let endpoint = EndpointClassifier::classify(
            "GET",
            "https://api.binance.com/api/v3/account",
            EndpointAuthKind::None,
        );

        assert_eq!(
            endpoint.endpoint_class,
            EndpointClass::ProductionAuthenticatedReadOnly
        );
        assert_eq!(endpoint.decision, EndpointDecision::Deny);
        assert!(!endpoint.read_allowed);
        assert!(!endpoint.mutation_allowed);
        assert!(endpoint.requires_api_key);
        assert!(endpoint.requires_signature);
    }

    #[test]
    fn endpoint_classifier_marks_production_order_post_as_preview_candidate_by_default() {
        let endpoint = EndpointClassifier::classify(
            "POST",
            "https://api.binance.com/api/v3/order",
            EndpointAuthKind::Signed,
        );

        assert_eq!(
            endpoint.endpoint_class,
            EndpointClass::ProductionMutationScopeCandidate
        );
        assert_eq!(
            endpoint.endpoint_class.as_str(),
            "production_mutation_scope_candidate"
        );
        assert_eq!(endpoint.decision, EndpointDecision::Deny);
        assert!(!endpoint.read_allowed);
        assert!(!endpoint.mutation_allowed);
        assert!(!endpoint.request_preview_allowed);
        assert!(endpoint.owner_gate_required);
        assert!(!endpoint.dashboard_order_controls_allowed);
        assert!(endpoint.reason.contains("owner-gated mutation-candidate"));
        assert!(!endpoint.reason.contains("v0.15"));
    }

    #[test]
    fn endpoint_classifier_allows_owner_manual_order_request_preview_only() {
        let endpoint = EndpointClassifier::classify_with_context(
            "POST",
            "https://api.binance.com/api/v3/order?timestamp=123&signature=secret",
            EndpointAuthKind::Signed,
            true,
        );

        assert_eq!(
            endpoint.endpoint_class,
            EndpointClass::ProductionMutationOwnerApprovedManualOnly
        );
        assert_eq!(
            endpoint.endpoint_class.as_str(),
            "production_mutation_owner_approved_manual_only"
        );
        assert_eq!(endpoint.decision, EndpointDecision::AllowRequestPreviewOnly);
        assert_eq!(endpoint.path, "/api/v3/order");
        assert!(!endpoint.read_allowed);
        assert!(!endpoint.mutation_allowed);
        assert!(endpoint.request_preview_allowed);
        assert!(endpoint.requires_api_key);
        assert!(endpoint.requires_signature);
        assert!(endpoint.owner_gate_required);
        assert!(!endpoint.dashboard_order_controls_allowed);
        assert_eq!(
            endpoint.input_url_redacted,
            "https://api.binance.com/api/v3/order?<redacted>"
        );
    }

    #[test]
    fn endpoint_classifier_denies_production_order_test_preview_under_owner_scope() {
        let endpoint = EndpointClassifier::classify_with_context(
            "POST",
            "https://api.binance.com/api/v3/order/test?timestamp=123&signature=secret",
            EndpointAuthKind::Signed,
            true,
        );

        assert_eq!(
            endpoint.endpoint_class,
            EndpointClass::ProductionMutationForbidden
        );
        assert_eq!(endpoint.decision, EndpointDecision::Deny);
        assert_eq!(endpoint.path, "/api/v3/order/test");
        assert!(!endpoint.read_allowed);
        assert!(!endpoint.mutation_allowed);
        assert!(!endpoint.request_preview_allowed);
        assert!(endpoint.requires_api_key);
        assert!(endpoint.requires_signature);
        assert!(endpoint.owner_gate_required);
        assert!(!endpoint.dashboard_order_controls_allowed);
        assert!(endpoint.reason.contains("owner-gated mutation-candidate"));
        assert!(!endpoint.reason.contains("v0.15"));
        assert_eq!(
            endpoint.input_url_redacted,
            "https://api.binance.com/api/v3/order/test?<redacted>"
        );
    }

    #[test]
    fn endpoint_classifier_denies_owner_manual_preview_without_signed_scope() {
        let endpoint = EndpointClassifier::classify_with_context(
            "POST",
            "https://api.binance.com/api/v3/order",
            EndpointAuthKind::None,
            true,
        );

        assert_eq!(
            endpoint.endpoint_class,
            EndpointClass::ProductionMutationScopeCandidate
        );
        assert_eq!(endpoint.decision, EndpointDecision::Deny);
        assert!(!endpoint.read_allowed);
        assert!(!endpoint.mutation_allowed);
        assert!(!endpoint.request_preview_allowed);
        assert!(endpoint.requires_api_key);
        assert!(endpoint.requires_signature);
        assert!(endpoint.owner_gate_required);
    }

    #[test]
    fn endpoint_classifier_keeps_cancel_and_listen_key_forbidden_under_owner_scope() {
        let cancel = EndpointClassifier::classify_with_context(
            "DELETE",
            "https://api.binance.com/api/v3/order",
            EndpointAuthKind::Signed,
            true,
        );
        let listen_key = EndpointClassifier::classify_with_context(
            "POST",
            "https://api.binance.com/api/v3/userDataStream",
            EndpointAuthKind::Signed,
            true,
        );

        for endpoint in [cancel, listen_key] {
            assert_eq!(
                endpoint.endpoint_class,
                EndpointClass::ProductionMutationForbidden
            );
            assert_eq!(endpoint.decision, EndpointDecision::Deny);
            assert!(!endpoint.read_allowed);
            assert!(!endpoint.mutation_allowed);
            assert!(!endpoint.request_preview_allowed);
            assert!(endpoint.owner_gate_required);
            assert!(!endpoint.dashboard_order_controls_allowed);
        }
    }

    #[test]
    fn endpoint_classifier_allows_signed_production_order_state_read_endpoint() {
        let endpoint = EndpointClassifier::classify(
            "GET",
            "https://api.binance.com/api/v3/openOrders",
            EndpointAuthKind::Signed,
        );

        assert_eq!(
            endpoint.endpoint_class,
            EndpointClass::ProductionOrderStateReadOnly
        );
        assert_eq!(
            endpoint.endpoint_class.as_str(),
            "production_order_state_read_only"
        );
        assert_eq!(endpoint.decision, EndpointDecision::AllowReadOnly);
        assert!(endpoint.read_allowed);
        assert!(!endpoint.mutation_allowed);
        assert!(endpoint.requires_api_key);
        assert!(endpoint.requires_signature);
        assert!(endpoint.owner_gate_required);
        assert!(!endpoint.dashboard_order_controls_allowed);
    }

    #[test]
    fn endpoint_classifier_denies_unsigned_production_order_state_read_endpoint() {
        let endpoint = EndpointClassifier::classify(
            "GET",
            "https://api.binance.com/api/v3/order",
            EndpointAuthKind::None,
        );

        assert_eq!(
            endpoint.endpoint_class,
            EndpointClass::ProductionOrderStateReadOnly
        );
        assert_eq!(endpoint.decision, EndpointDecision::Deny);
        assert!(!endpoint.read_allowed);
        assert!(!endpoint.mutation_allowed);
        assert!(endpoint.requires_api_key);
        assert!(endpoint.requires_signature);
    }

    #[test]
    fn endpoint_classifier_denies_unknown_endpoint() {
        let endpoint = EndpointClassifier::classify(
            "GET",
            "https://example.com/api/v3/time?signature=secret",
            EndpointAuthKind::Signed,
        );

        assert_eq!(endpoint.endpoint_class, EndpointClass::UnknownForbidden);
        assert_eq!(endpoint.decision, EndpointDecision::Deny);
        assert_eq!(
            endpoint.input_url_redacted,
            "https://example.com/api/v3/time?<redacted>"
        );
        assert!(!endpoint.read_allowed);
        assert!(!endpoint.mutation_allowed);
    }

    #[test]
    fn endpoint_classifier_classifies_sandbox_and_testnet_hosts() {
        let sandbox = EndpointClassifier::classify(
            "GET",
            "https://demo-api.binance.com/api/v3/time",
            EndpointAuthKind::None,
        );
        let testnet = EndpointClassifier::classify(
            "GET",
            "https://testnet.binance.vision/api/v3/time",
            EndpointAuthKind::None,
        );

        assert_eq!(sandbox.endpoint_class, EndpointClass::SandboxSpotDemo);
        assert_eq!(sandbox.decision, EndpointDecision::AllowReadOnly);
        assert_eq!(
            testnet.endpoint_class,
            EndpointClass::SandboxSpotTestNetwork
        );
        assert_eq!(testnet.decision, EndpointDecision::AllowReadOnly);
    }

    #[test]
    fn endpoint_classifier_classifies_public_websocket_read_only_surface() {
        let public_stream = EndpointClassifier::classify(
            "GET",
            "wss://stream.binance.com/ws/btcusdt@trade",
            EndpointAuthKind::None,
        );

        assert_eq!(
            public_stream.endpoint_class,
            EndpointClass::WebsocketPublicReadOnly
        );
        assert_eq!(public_stream.decision, EndpointDecision::AllowReadOnly);
        assert!(public_stream.read_allowed);
        assert!(!public_stream.mutation_allowed);
    }

    #[test]
    fn endpoint_classifier_denies_websocket_user_stream_for_v12() {
        let user_stream = EndpointClassifier::classify(
            "GET",
            "wss://stream.binance.com/ws/<listen-key>",
            EndpointAuthKind::Signed,
        );

        assert_eq!(
            user_stream.endpoint_class,
            EndpointClass::WebsocketUserReadOnly
        );
        assert_eq!(user_stream.decision, EndpointDecision::Deny);
        assert!(!user_stream.read_allowed);
        assert!(user_stream.owner_gate_required);
        assert!(!user_stream.mutation_allowed);
        assert_eq!(
            user_stream.reason,
            "websocket user stream requires listenKey lifecycle and is deferred for v0.12"
        );
    }
}
