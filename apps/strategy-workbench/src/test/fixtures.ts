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
    contract_id: "mvp-node-001:ema-cross:mvp-strategy-001",
    identities: {
      strategy_id: "ema-cross",
      strategy_version: "v1",
      backtest_run_id: "backtest-001",
      backtest_result_ref: "artifact://backtests/backtest-001/summary.json",
      node_id: "mvp-node-001",
      strategy_instance_id: "mvp-strategy-001",
      account_id: "acct-sandbox-001",
      venue_id: "BINANCE",
      environment: "sandbox",
    },
    boundaries: { read_only_product_contract: true, ...closedBoundaries },
  },
  status: {
    identity_contract_id: "mvp-node-001:ema-cross:mvp-strategy-001",
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
