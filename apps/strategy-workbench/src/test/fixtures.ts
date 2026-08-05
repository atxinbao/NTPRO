const closedBoundaries = {
  external_venue_connection: false,
  order_submission_allowed: false,
  order_mutation_allowed: false,
  automatic_retry_allowed: false,
  automatic_remediation_allowed: false,
  real_orders_submitted: false,
};

const axis = (status: string) => ({
  status,
  availability: "available",
  freshness: "fresh",
  source_refs: ["status.json"],
  observed_at_unix_ms: 1,
  reasons: [],
});

const value = (entry: string) => ({ availability: "available", value: entry });
const component = (summary: string) => ({
  status: value("available"),
  summary: value(summary),
  freshness_status: value("fresh"),
  source_ref: value("snapshot.json"),
});

export const validStatusPayload = {
  schema_version: "ntpro.mvp_shared_status_api.response.v1",
  contract_version: "ntpro.mvp_shared_status_api.v1",
  generated_at_unix_ms: 1_754_400_000_000,
  consumers: ["institution_workbench", "control_center"],
  identity: {
    contract_id: "node-1:btc-ema:instance-1",
    identities: {
      strategy_id: "btc-ema",
      strategy_version: "sha256:v12",
      backtest_run_id: "bt-1",
      backtest_result_ref: "artifact://backtests/bt-1.json",
      node_id: "node-1",
      strategy_instance_id: "instance-1",
      account_id: "acct-sandbox-1",
      venue_id: "sandbox",
      environment: "sandbox",
    },
    boundaries: { read_only_product_contract: true, ...closedBoundaries },
  },
  status: {
    identity_contract_id: "node-1:btc-ema:instance-1",
    research: axis("reference_bound"),
    runtime: axis("running"),
    technical_health: axis("healthy"),
    trading_readiness: axis("blocked"),
    boundaries: { read_only_product_contract: true, ...closedBoundaries },
  },
  business: {
    availability: "available",
    freshness_status: value("fresh"),
    source_ref: value("snapshot.json"),
    positions: component("无持仓"),
    lifecycle: component("运行中"),
    fills: component("无成交"),
    diagnostic: value("只读状态正常"),
  },
  source_refs: ["identity.json", "status.json"],
  boundaries: { read_only: true, ...closedBoundaries },
};
