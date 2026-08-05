const STATUS_URL = "/api/mvp/v1/status";
const EXPECTED_SCHEMA = "ntpro.mvp_shared_status_api.response.v1";
const EXPECTED_CONTRACT = "ntpro.mvp_shared_status_api.v1";

const CLOSED_BOUNDARIES = [
  "external_venue_connection",
  "order_submission_allowed",
  "order_mutation_allowed",
  "automatic_retry_allowed",
  "automatic_remediation_allowed",
  "real_orders_submitted",
] as const;

type JsonRecord = Record<string, unknown>;

export interface StatusAxis {
  status: string;
  availability: string;
  freshness: string;
  sourceRefs: string[];
  reasons: string[];
}

export interface MvpStatusView {
  generatedAtUnixMs: number;
  identityContractId: string;
  strategyId: string;
  strategyVersion: string;
  backtestRunId: string;
  backtestResultRef: string;
  nodeId: string;
  strategyInstanceId: string;
  accountId: string;
  venueId: string;
  environment: "sandbox";
  axes: {
    research: StatusAxis;
    runtime: StatusAxis;
    technicalHealth: StatusAxis;
    tradingReadiness: StatusAxis;
  };
  business: {
    availability: string;
    freshness: string;
    positions: string;
    lifecycle: string;
    fills: string;
    diagnostic: string;
    sourceRef: string;
  };
  sourceRefs: string[];
}

export class StatusContractError extends Error {
  constructor(field: string) {
    super(`共享状态合同无效：${field}`);
    this.name = "StatusContractError";
  }
}

function record(value: unknown, field: string): JsonRecord {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new StatusContractError(field);
  }
  return value as JsonRecord;
}

function stringValue(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new StatusContractError(field);
  }
  return value;
}

function numberValue(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value <= 0) {
    throw new StatusContractError(field);
  }
  return value;
}

function stringList(
  value: unknown,
  field: string,
  allowEmpty = false,
): string[] {
  if (!Array.isArray(value)) throw new StatusContractError(field);
  const result = value.map((entry, index) =>
    stringValue(entry, `${field}[${index}]`),
  );
  if (!allowEmpty && result.length === 0) throw new StatusContractError(field);
  return result;
}

function assertClosedBoundaries(
  value: unknown,
  field: string,
  readOnlyKey: string,
): void {
  const boundaries = record(value, field);
  if (boundaries[readOnlyKey] !== true) {
    throw new StatusContractError(`${field}.${readOnlyKey}`);
  }
  for (const boundary of CLOSED_BOUNDARIES) {
    if (boundaries[boundary] !== false) {
      throw new StatusContractError(`${field}.${boundary}`);
    }
  }
}

function axis(
  value: unknown,
  field: string,
  allowedStatuses: readonly string[],
): StatusAxis {
  const source = record(value, field);
  const status = stringValue(source.status, `${field}.status`);
  const availability = stringValue(
    source.availability,
    `${field}.availability`,
  );
  const freshness = stringValue(source.freshness, `${field}.freshness`);
  if (!allowedStatuses.includes(status))
    throw new StatusContractError(`${field}.status`);
  if (!["available", "missing", "error", "unknown"].includes(availability))
    throw new StatusContractError(`${field}.availability`);
  if (!["fresh", "stale", "unknown"].includes(freshness))
    throw new StatusContractError(`${field}.freshness`);
  numberValue(source.observed_at_unix_ms, `${field}.observed_at_unix_ms`);
  return {
    status,
    availability,
    freshness,
    sourceRefs: stringList(source.source_refs, `${field}.source_refs`),
    reasons: stringList(source.reasons, `${field}.reasons`, true),
  };
}

function dashboardString(value: unknown, field: string): string {
  const source = record(value, field);
  const availability = stringValue(
    source.availability,
    `${field}.availability`,
  );
  if (availability !== "available") return "不可用";
  return stringValue(source.value, `${field}.value`);
}

function componentSummary(value: unknown, field: string): string {
  return dashboardString(record(value, field).summary, `${field}.summary`);
}

export function parseMvpStatus(payload: unknown): MvpStatusView {
  const root = record(payload, "response");
  if (root.schema_version !== EXPECTED_SCHEMA)
    throw new StatusContractError("schema_version");
  if (root.contract_version !== EXPECTED_CONTRACT)
    throw new StatusContractError("contract_version");
  if (
    !stringList(root.consumers, "consumers").includes("institution_workbench")
  )
    throw new StatusContractError("consumers");

  const identity = record(root.identity, "identity");
  const identities = record(identity.identities, "identity.identities");
  const status = record(root.status, "status");
  const business = record(root.business, "business");
  const identityContractId = stringValue(
    identity.contract_id,
    "identity.contract_id",
  );
  const strategyId = stringValue(
    identities.strategy_id,
    "identity.identities.strategy_id",
  );
  const nodeId = stringValue(identities.node_id, "identity.identities.node_id");
  const strategyInstanceId = stringValue(
    identities.strategy_instance_id,
    "identity.identities.strategy_instance_id",
  );
  if (identityContractId !== `${nodeId}:${strategyId}:${strategyInstanceId}`) {
    throw new StatusContractError("identity.contract_id");
  }
  if (status.identity_contract_id !== identityContractId) {
    throw new StatusContractError("status.identity_contract_id");
  }

  assertClosedBoundaries(root.boundaries, "boundaries", "read_only");
  assertClosedBoundaries(
    identity.boundaries,
    "identity.boundaries",
    "read_only_product_contract",
  );
  assertClosedBoundaries(
    status.boundaries,
    "status.boundaries",
    "read_only_product_contract",
  );

  const environment = stringValue(
    identities.environment,
    "identity.identities.environment",
  );
  if (environment !== "sandbox")
    throw new StatusContractError("identity.identities.environment");

  const research = axis(status.research, "status.research", [
    "reference_bound",
  ]);
  const runtime = axis(status.runtime, "status.runtime", [
    "running",
    "stopped",
    "transitioning",
    "unknown",
  ]);
  const technicalHealth = axis(
    status.technical_health,
    "status.technical_health",
    ["healthy", "degraded", "unhealthy", "not_running", "unknown"],
  );
  const tradingReadiness = axis(
    status.trading_readiness,
    "status.trading_readiness",
    ["blocked"],
  );
  const businessAvailability = stringValue(
    business.availability,
    "business.availability",
  );
  if (
    !["available", "missing", "stale", "error", "identity_mismatch"].includes(
      businessAvailability,
    )
  ) {
    throw new StatusContractError("business.availability");
  }

  return {
    generatedAtUnixMs: numberValue(
      root.generated_at_unix_ms,
      "generated_at_unix_ms",
    ),
    identityContractId,
    strategyId,
    strategyVersion: stringValue(
      identities.strategy_version,
      "identity.identities.strategy_version",
    ),
    backtestRunId: stringValue(
      identities.backtest_run_id,
      "identity.identities.backtest_run_id",
    ),
    backtestResultRef: stringValue(
      identities.backtest_result_ref,
      "identity.identities.backtest_result_ref",
    ),
    nodeId,
    strategyInstanceId,
    accountId: stringValue(
      identities.account_id,
      "identity.identities.account_id",
    ),
    venueId: stringValue(identities.venue_id, "identity.identities.venue_id"),
    environment,
    axes: {
      research,
      runtime,
      technicalHealth,
      tradingReadiness,
    },
    business: {
      availability: businessAvailability,
      freshness: dashboardString(
        business.freshness_status,
        "business.freshness_status",
      ),
      positions: componentSummary(business.positions, "business.positions"),
      lifecycle: componentSummary(business.lifecycle, "business.lifecycle"),
      fills: componentSummary(business.fills, "business.fills"),
      diagnostic: dashboardString(business.diagnostic, "business.diagnostic"),
      sourceRef: dashboardString(business.source_ref, "business.source_ref"),
    },
    sourceRefs: stringList(root.source_refs, "source_refs"),
  };
}

export async function fetchMvpStatus(
  signal?: AbortSignal,
): Promise<MvpStatusView> {
  const response = await fetch(STATUS_URL, {
    credentials: "same-origin",
    headers: { Accept: "application/json" },
    signal,
  });
  if (!response.ok)
    throw new Error(`共享状态请求失败：HTTP ${response.status}`);
  return parseMvpStatus(await response.json());
}

export function statusLabel(value: string): string {
  const labels: Record<string, string> = {
    reference_bound: "已绑定",
    running: "运行中",
    stopped: "已停止",
    transitioning: "切换中",
    healthy: "健康",
    degraded: "降级",
    unhealthy: "不健康",
    blocked: "阻断",
    fresh: "新鲜",
    stale: "陈旧",
    available: "可用",
  };
  return labels[value] ?? value;
}
