import { createClient } from "./generated/productApi/client";
import {
  actOnDemoRun,
  compareRuns,
  createBacktestRun,
  createDemoRun,
  getDemoRunSnapshot,
  getRunReproductionProof,
  getRunAnalysis,
  getRun,
  getRunMetrics,
  getRunReport,
  getStrategy,
  getStrategyVersion,
  listRuns,
  listStrategies,
  listStrategyVersions,
  reproduceBacktestRun,
  type CompareRunsData,
  type CreateBacktestRunRequest,
  type CreateDemoRunRequest,
  type DemoRunAction,
  type DemoRunActionResponse,
  type DemoRunCreateResponse,
  type DemoRunSnapshotResponse,
  type GetDemoRunSnapshotData,
  type GetRunData,
  type GetRunAnalysisData,
  type GetRunMetricsData,
  type GetRunReportData,
  type GetRunReproductionProofData,
  type GetStrategyData,
  type GetStrategyVersionData,
  type ListRunsData,
  type ListStrategiesData,
  type ListStrategyVersionsData,
  type ProductErrorResponse,
  type ReproduceBacktestRunRequest,
  type RunComparisonResponse,
  type RunCreateResponse,
  type RunAnalysisResponse,
  type RunDetailResponse,
  type RunListResponse,
  type RunMetricsResponse,
  type RunReportResponse,
  type RunReproductionProofResponse,
  type RunReproductionResponse,
  type StrategyDetailResponse,
  type StrategyListResponse,
  type StrategyVersionDetailResponse,
  type StrategyVersionListResponse,
} from "./generated/productApi";
import {
  zDemoRunActionResponse,
  zDemoRunCreateResponse,
  zDemoRunSnapshotResponse,
  zProductErrorResponse,
  zRunCreateResponse,
  zRunAnalysisResponse,
  zRunDetailResponse,
  zRunListResponse,
  zRunMetricsResponse,
  zRunReportResponse,
  zRunComparisonResponse,
  zRunReproductionProofResponse,
  zRunReproductionResponse,
  zStrategyDetailResponse,
  zStrategyListResponse,
  zStrategyVersionDetailResponse,
  zStrategyVersionListResponse,
} from "./generated/productApi/zod.gen";
import type { z } from "zod";

const PRODUCT_API_BASE_URL = "/api/product/v1";

type ListStrategiesQuery = NonNullable<ListStrategiesData["query"]>;
type StrategyPath = GetStrategyData["path"];
type ListStrategyVersionsQuery = NonNullable<ListStrategyVersionsData["query"]>;
type StrategyVersionPath = GetStrategyVersionData["path"];
type ListRunsQuery = NonNullable<ListRunsData["query"]>;
type RunPath = GetRunData["path"];
type DemoRunSnapshotPath = GetDemoRunSnapshotData["path"];
type RunAnalysisPath = GetRunAnalysisData["path"];
type RunMetricsPath = GetRunMetricsData["path"];
type RunReportPath = GetRunReportData["path"];
type RunReproductionPath = GetRunReproductionProofData["path"];

interface ProductApiClientOptions {
  baseUrl?: string;
  fetch?: typeof fetch;
}

interface RequestFields {
  data: unknown;
  error: unknown;
  response?: Response;
}

function resolveBaseUrl(baseUrl: string): string {
  if (/^https?:\/\//u.test(baseUrl)) return baseUrl;
  if (typeof globalThis.location === "undefined") return baseUrl;
  return new URL(baseUrl, globalThis.location.origin)
    .toString()
    .replace(/\/$/u, "");
}

export class ProductApiContractError extends Error {
  readonly field: string;

  constructor(field: string, cause?: unknown) {
    super(`产品 API 合同无效：${field}`, { cause });
    this.name = "ProductApiContractError";
    this.field = field;
  }
}

export class ProductApiRequestError extends Error {
  readonly status: number;
  readonly requestId: string;
  readonly code: ProductErrorResponse["error"]["code"];
  readonly field: string;
  readonly retryable: boolean;

  constructor(status: number, response: ProductErrorResponse) {
    super(response.error.summary);
    this.name = "ProductApiRequestError";
    this.status = status;
    this.requestId = response.request_id;
    this.code = response.error.code;
    this.field = response.error.field;
    this.retryable = response.error.retryable;
  }
}

export class ProductApiTransportError extends Error {
  constructor(cause: unknown) {
    super("产品 API 请求未获得有效 HTTP 响应", { cause });
    this.name = "ProductApiTransportError";
  }
}

function canonicalJson(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(canonicalJson);
  if (typeof value !== "object" || value === null) return value;
  return Object.fromEntries(
    Object.entries(value)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([key, entry]) => [key, canonicalJson(entry)]),
  );
}

function parseStrict<T>(
  schema: z.ZodType<T>,
  payload: unknown,
  field: string,
): T {
  const result = schema.safeParse(payload);
  if (!result.success) throw new ProductApiContractError(field, result.error);
  if (
    JSON.stringify(canonicalJson(result.data)) !==
    JSON.stringify(canonicalJson(payload))
  ) {
    throw new ProductApiContractError(`${field}.unknown_or_defaulted_field`);
  }
  return result.data;
}

function assertPage(
  payload: StrategyListResponse | StrategyVersionListResponse | RunListResponse,
  field: string,
): void {
  if (payload.page.returned_count !== payload.data.length) {
    throw new ProductApiContractError(`${field}.page.returned_count`);
  }
  if (payload.page.returned_count > payload.page.limit) {
    throw new ProductApiContractError(`${field}.page.limit`);
  }
  const hasCursor = payload.page.next_cursor !== null;
  if (payload.page.has_more !== hasCursor) {
    throw new ProductApiContractError(`${field}.page.next_cursor`);
  }
  if (hasCursor && payload.page.next_cursor?.trim() === "") {
    throw new ProductApiContractError(`${field}.page.next_cursor`);
  }
  if (payload.data.length === 0 && payload.page.has_more) {
    throw new ProductApiContractError(`${field}.page.has_more`);
  }
}

function assertIdentity(condition: boolean, field: string): void {
  if (!condition) throw new ProductApiContractError(field);
}

function assertUniqueIds(values: string[], field: string): void {
  assertIdentity(new Set(values).size === values.length, field);
}

function assertStrategyListScope(
  payload: StrategyListResponse,
  query?: ListStrategiesQuery,
): void {
  assertUniqueIds(
    payload.data.map((strategy) => strategy.strategy_id),
    "strategy_list.data.strategy_id.duplicate",
  );
  for (const strategy of payload.data) {
    if (query?.lifecycle) {
      assertIdentity(
        strategy.lifecycle === query.lifecycle,
        "strategy_list.query.lifecycle",
      );
    }
    if (query?.owner) {
      assertIdentity(
        strategy.owner === query.owner,
        "strategy_list.query.owner",
      );
    }
  }
}

function assertVersionListScope(
  payload: StrategyVersionListResponse,
  strategyId: string,
  query?: ListStrategyVersionsQuery,
): void {
  assertUniqueIds(
    payload.data.map((version) => version.strategy_version_id),
    "strategy_version_list.data.strategy_version_id.duplicate",
  );
  for (const version of payload.data) {
    assertIdentity(
      version.strategy_id === strategyId,
      "strategy_version_list.path.strategy_id",
    );
    if (query?.status) {
      assertIdentity(
        version.status === query.status,
        "strategy_version_list.query.status",
      );
    }
  }
}

function assertRunListScope(
  payload: RunListResponse,
  query?: ListRunsQuery,
): void {
  assertUniqueIds(
    payload.data.map((run) => run.run_id),
    "run_list.data.run_id.duplicate",
  );
  for (const run of payload.data) {
    if (query?.strategy_id) {
      assertIdentity(
        run.strategy_id === query.strategy_id,
        "run_list.query.strategy_id",
      );
    }
    if (query?.strategy_version_id) {
      assertIdentity(
        run.strategy_version_id === query.strategy_version_id,
        "run_list.query.strategy_version_id",
      );
    }
    if (query?.environment) {
      assertIdentity(
        run.environment === query.environment,
        "run_list.query.environment",
      );
    }
    if (query?.lifecycle) {
      assertIdentity(
        run.lifecycle === query.lifecycle,
        "run_list.query.lifecycle",
      );
    }
  }
}

function assertReadOnlyBoundaries(
  boundaries: RunComparisonResponse["boundaries"],
  field: string,
): void {
  assertIdentity(
    boundaries.read_only &&
      !boundaries.strategy_mutation_allowed &&
      !boundaries.run_mutation_allowed &&
      !boundaries.external_venue_connection &&
      !boundaries.order_submission_allowed &&
      !boundaries.order_mutation_allowed &&
      !boundaries.automatic_retry_allowed &&
      !boundaries.automatic_remediation_allowed &&
      !boundaries.real_orders_submitted &&
      !boundaries.trading_controls_enabled,
    field,
  );
}

function assertDemoBoundaries(
  boundaries: DemoRunCreateResponse["boundaries"],
  field: string,
): void {
  assertIdentity(
    boundaries.demo_run_creation_allowed &&
      boundaries.demo_start_allowed &&
      boundaries.demo_stop_allowed &&
      !boundaries.live_run_creation_allowed &&
      !boundaries.external_venue_connection &&
      !boundaries.order_submission_allowed &&
      !boundaries.order_mutation_allowed &&
      !boundaries.automatic_retry_allowed &&
      !boundaries.automatic_remediation_allowed &&
      !boundaries.real_orders_submitted &&
      !boundaries.trading_controls_enabled,
    field,
  );
}

function assertDemoRun(
  run: DemoRunCreateResponse["data"],
  field: string,
): void {
  assertIdentity(
    run.environment === "sandbox" &&
      run.runtime !== null &&
      run.runtime.supervisor_node_id.trim() !== "" &&
      run.runtime.strategy_instance_id.trim() !== "" &&
      !run.capabilities.external_venue_connection &&
      !run.capabilities.order_submission_allowed &&
      !run.capabilities.order_mutation_allowed &&
      !run.capabilities.automatic_retry_allowed &&
      !run.capabilities.automatic_remediation_allowed &&
      !run.capabilities.real_orders_submitted &&
      !run.capabilities.trading_controls_enabled,
    field,
  );
}

async function resolveResponse<T>(
  request: Promise<RequestFields>,
  schema: z.ZodType<T>,
  field: string,
  paginated = false,
): Promise<T> {
  let result: RequestFields;
  try {
    result = await request;
  } catch (error) {
    throw new ProductApiTransportError(error);
  }

  if (result.data !== undefined) {
    const parsed = parseStrict(schema, result.data, field);
    if (paginated) {
      assertPage(
        parsed as
          StrategyListResponse | StrategyVersionListResponse | RunListResponse,
        field,
      );
    }
    return parsed;
  }

  if (!result.response) throw new ProductApiTransportError(result.error);
  const error = parseStrict(
    zProductErrorResponse,
    result.error,
    `${field}.error`,
  );
  throw new ProductApiRequestError(result.response.status, error);
}

export function createProductApiClient(options: ProductApiClientOptions = {}) {
  const client = createClient({
    baseUrl: resolveBaseUrl(options.baseUrl ?? PRODUCT_API_BASE_URL),
    credentials: "same-origin",
    fetch: options.fetch,
    headers: { Accept: "application/json" },
  });

  return {
    async createDemoRun(
      body: CreateDemoRunRequest,
      signal?: AbortSignal,
    ): Promise<DemoRunCreateResponse> {
      const payload = await resolveResponse(
        createDemoRun({ client, body, signal }),
        zDemoRunCreateResponse,
        "demo_run_create",
      );
      assertIdentity(
        payload.data.strategy_id === body.strategy_id &&
          payload.data.strategy_version_id === body.strategy_version_id &&
          payload.data.account_ref === body.account_ref &&
          payload.data.venue_ref === body.venue_ref &&
          payload.data.runtime?.supervisor_node_id ===
            body.supervisor_node_id &&
          payload.data.lifecycle === "created",
        "demo_run_create.data.identity",
      );
      assertDemoRun(payload.data, "demo_run_create.data.boundaries");
      assertDemoBoundaries(payload.boundaries, "demo_run_create.boundaries");
      return payload;
    },

    async actOnDemoRun(
      runId: string,
      action: DemoRunAction,
      signal?: AbortSignal,
    ): Promise<DemoRunActionResponse> {
      const body = { run_id: runId, action, user_confirmed: true } as const;
      const payload = await resolveResponse(
        actOnDemoRun({ client, path: { run_id: runId }, body, signal }),
        zDemoRunActionResponse,
        "demo_run_action",
      );
      assertIdentity(
        payload.data.run_id === runId &&
          payload.data.action === action &&
          payload.data.current_run.run_id === runId,
        "demo_run_action.data.identity",
      );
      assertDemoRun(
        payload.data.current_run,
        "demo_run_action.data.boundaries",
      );
      if (action === "start") {
        assertIdentity(
          ["queued", "running"].includes(payload.data.current_run.lifecycle),
          "demo_run_action.data.lifecycle",
        );
      } else {
        assertIdentity(
          payload.data.current_run.lifecycle === "stopped",
          "demo_run_action.data.lifecycle",
        );
      }
      assertDemoBoundaries(payload.boundaries, "demo_run_action.boundaries");
      return payload;
    },

    async compareRuns(
      runIds: string[],
      signal?: AbortSignal,
    ): Promise<RunComparisonResponse> {
      assertIdentity(
        runIds.length >= 2 &&
          runIds.length <= 4 &&
          new Set(runIds).size === runIds.length,
        "run_comparison.request.run_ids",
      );
      const query: CompareRunsData["query"] = {
        run_ids: runIds.join(","),
      };
      const payload = await resolveResponse(
        compareRuns({ client, query, signal }),
        zRunComparisonResponse,
        "run_comparison",
      );
      assertIdentity(
        payload.data.baseline_run_id === runIds[0] &&
          payload.data.run_ids.length === runIds.length &&
          payload.data.run_ids.every(
            (runId, index) => runId === runIds[index],
          ) &&
          payload.data.items.length === runIds.length &&
          payload.data.items.every(
            (item, index) => item.run_id === runIds[index],
          ) &&
          payload.data.compatibility.same_strategy,
        "run_comparison.data.identity",
      );
      assertIdentity(
        payload.data.compatibility.directly_comparable ===
          (payload.data.compatibility.same_data &&
            payload.data.compatibility.same_instrument &&
            payload.data.compatibility.same_currency),
        "run_comparison.data.compatibility",
      );
      assertReadOnlyBoundaries(payload.boundaries, "run_comparison.boundaries");
      return payload;
    },

    async createBacktestRun(
      body: CreateBacktestRunRequest,
      signal?: AbortSignal,
    ): Promise<RunCreateResponse> {
      const payload = await resolveResponse(
        createBacktestRun({ client, body, signal }),
        zRunCreateResponse,
        "run_create",
      );
      assertIdentity(
        payload.data.strategy_id === body.strategy_id,
        "run_create.body.strategy_id",
      );
      assertIdentity(
        payload.data.strategy_version_id === body.strategy_version_id,
        "run_create.body.strategy_version_id",
      );
      assertIdentity(
        payload.data.data_ref === body.data_ref,
        "run_create.body.data_ref",
      );
      assertIdentity(
        payload.data.venue_ref === body.venue_ref,
        "run_create.body.venue_ref",
      );
      assertIdentity(
        payload.data.environment === "backtest" &&
          payload.data.lifecycle === "completed" &&
          payload.data.result.status === "available" &&
          payload.data.result.result_ref !== null &&
          payload.data.result.report_ref !== null &&
          payload.data.result.analysis_ref !== null &&
          payload.data.started_at_unix_ms !== null &&
          payload.data.completed_at_unix_ms !== null &&
          payload.data.created_at_unix_ms <= payload.data.started_at_unix_ms &&
          payload.data.started_at_unix_ms <=
            payload.data.completed_at_unix_ms &&
          payload.data.completed_at_unix_ms <= payload.data.updated_at_unix_ms,
        "run_create.data.lifecycle",
      );
      assertIdentity(
        payload.boundaries.backtest_run_creation_allowed &&
          !payload.boundaries.sandbox_run_creation_allowed &&
          !payload.boundaries.live_run_creation_allowed &&
          !payload.boundaries.external_venue_connection &&
          !payload.boundaries.order_submission_allowed &&
          !payload.boundaries.order_mutation_allowed &&
          !payload.boundaries.automatic_retry_allowed &&
          !payload.boundaries.automatic_remediation_allowed &&
          !payload.boundaries.real_orders_submitted &&
          !payload.boundaries.trading_controls_enabled,
        "run_create.boundaries",
      );
      return payload;
    },

    async reproduceBacktestRun(
      sourceRunId: string,
      signal?: AbortSignal,
    ): Promise<RunReproductionResponse> {
      const body: ReproduceBacktestRunRequest = {
        source_run_id: sourceRunId,
        deterministic_replay: true,
      };
      const payload = await resolveResponse(
        reproduceBacktestRun({
          client,
          path: { run_id: sourceRunId },
          body,
          signal,
        }),
        zRunReproductionResponse,
        "run_reproduction",
      );
      const { proof, reproduced_run: reproducedRun } = payload.data;
      assertIdentity(
        payload.data.source_run_id === sourceRunId &&
          proof.source_run_id === sourceRunId &&
          proof.reproduced_run_id === reproducedRun.run_id &&
          reproducedRun.run_id !== sourceRunId &&
          reproducedRun.environment === "backtest" &&
          reproducedRun.lifecycle === "completed" &&
          reproducedRun.result.status === "available" &&
          reproducedRun.result.reproduction_ref === proof.proof_ref &&
          proof.input_equivalent &&
          proof.output_equivalent &&
          proof.user_initiated &&
          !proof.automatic_retry_allowed &&
          !proof.automatic_remediation_allowed,
        "run_reproduction.data.identity",
      );
      assertIdentity(
        payload.boundaries.backtest_run_creation_allowed &&
          !payload.boundaries.sandbox_run_creation_allowed &&
          !payload.boundaries.live_run_creation_allowed &&
          !payload.boundaries.external_venue_connection &&
          !payload.boundaries.order_submission_allowed &&
          !payload.boundaries.order_mutation_allowed &&
          !payload.boundaries.automatic_retry_allowed &&
          !payload.boundaries.automatic_remediation_allowed &&
          !payload.boundaries.real_orders_submitted &&
          !payload.boundaries.trading_controls_enabled,
        "run_reproduction.boundaries",
      );
      return payload;
    },

    async listStrategies(
      query?: ListStrategiesQuery,
      signal?: AbortSignal,
    ): Promise<StrategyListResponse> {
      const payload = await resolveResponse(
        listStrategies({ client, query, signal }),
        zStrategyListResponse,
        "strategy_list",
        true,
      );
      assertStrategyListScope(payload, query);
      return payload;
    },

    async getStrategy(
      path: StrategyPath,
      signal?: AbortSignal,
    ): Promise<StrategyDetailResponse> {
      const payload = await resolveResponse(
        getStrategy({ client, path, signal }),
        zStrategyDetailResponse,
        "strategy_detail",
      );
      assertIdentity(
        payload.data.strategy_id === path.strategy_id,
        "strategy_detail.path.strategy_id",
      );
      return payload;
    },

    async listStrategyVersions(
      path: ListStrategyVersionsData["path"],
      query?: ListStrategyVersionsQuery,
      signal?: AbortSignal,
    ): Promise<StrategyVersionListResponse> {
      const payload = await resolveResponse(
        listStrategyVersions({ client, path, query, signal }),
        zStrategyVersionListResponse,
        "strategy_version_list",
        true,
      );
      assertVersionListScope(payload, path.strategy_id, query);
      return payload;
    },

    async getStrategyVersion(
      path: StrategyVersionPath,
      signal?: AbortSignal,
    ): Promise<StrategyVersionDetailResponse> {
      const payload = await resolveResponse(
        getStrategyVersion({ client, path, signal }),
        zStrategyVersionDetailResponse,
        "strategy_version_detail",
      );
      assertIdentity(
        payload.data.strategy_id === path.strategy_id,
        "strategy_version_detail.path.strategy_id",
      );
      assertIdentity(
        payload.data.strategy_version_id === path.version_id,
        "strategy_version_detail.path.version_id",
      );
      return payload;
    },

    async listRuns(
      query?: ListRunsQuery,
      signal?: AbortSignal,
    ): Promise<RunListResponse> {
      const payload = await resolveResponse(
        listRuns({ client, query, signal }),
        zRunListResponse,
        "run_list",
        true,
      );
      assertRunListScope(payload, query);
      return payload;
    },

    async getRun(
      path: RunPath,
      signal?: AbortSignal,
    ): Promise<RunDetailResponse> {
      const payload = await resolveResponse(
        getRun({ client, path, signal }),
        zRunDetailResponse,
        "run_detail",
      );
      assertIdentity(
        payload.data.run_id === path.run_id,
        "run_detail.path.run_id",
      );
      return payload;
    },

    async getDemoRunSnapshot(
      path: DemoRunSnapshotPath,
      signal?: AbortSignal,
    ): Promise<DemoRunSnapshotResponse> {
      const payload = await resolveResponse(
        getDemoRunSnapshot({ client, path, signal }),
        zDemoRunSnapshotResponse,
        "demo_run_snapshot",
      );
      const snapshot = payload.data;
      assertIdentity(
        snapshot.run_id === path.run_id,
        "demo_run_snapshot.path.run_id",
      );
      assertIdentity(
        payload.boundaries.read_only &&
          payload.boundaries.sandbox_only &&
          !payload.boundaries.live_run_creation_allowed &&
          !payload.boundaries.external_venue_connection &&
          !payload.boundaries.order_submission_allowed &&
          !payload.boundaries.order_mutation_allowed &&
          !payload.boundaries.automatic_retry_allowed &&
          !payload.boundaries.automatic_remediation_allowed &&
          !payload.boundaries.real_orders_submitted &&
          !payload.boundaries.trading_controls_enabled,
        "demo_run_snapshot.boundaries",
      );
      const hasRuntimeData =
        snapshot.market !== null && snapshot.session !== null;
      const hasNoRuntimeData =
        snapshot.market === null &&
        snapshot.session === null &&
        snapshot.latest_signal === null &&
        snapshot.latest_order_intent === null &&
        snapshot.latest_risk_decision === null &&
        snapshot.simulation === null;
      const hasSimulation =
        snapshot.simulation !== null &&
        snapshot.simulation.fills.length > 0 &&
        snapshot.simulation.positions.length > 0 &&
        snapshot.simulation.equity_curve.length > 0 &&
        snapshot.simulation.summary.fill_count ===
          snapshot.simulation.fills.length &&
        snapshot.simulation.summary.position_count ===
          snapshot.simulation.positions.length &&
        snapshot.simulation.summary.equity_point_count ===
          snapshot.simulation.equity_curve.length;
      assertIdentity(
        (snapshot.snapshot_status === "not_started" &&
          snapshot.lifecycle === "created" &&
          hasNoRuntimeData &&
          snapshot.provenance.result_ref === null &&
          snapshot.provenance.result_sha256 === null) ||
          (snapshot.snapshot_status === "running" &&
            ["running", "paused", "stopping"].includes(snapshot.lifecycle) &&
            hasRuntimeData &&
            hasSimulation &&
            snapshot.technical_health.status === "healthy" &&
            snapshot.provenance.result_ref === null &&
            snapshot.provenance.result_sha256 === null) ||
          (snapshot.snapshot_status === "frozen" &&
            ["stopped", "failed"].includes(snapshot.lifecycle) &&
            (snapshot.lifecycle === "failed" || hasSimulation) &&
            snapshot.provenance.result_ref !== null &&
            snapshot.provenance.result_sha256 !== null),
        "demo_run_snapshot.data.state",
      );
      assertIdentity(
        (snapshot.session?.actual_submission_count ?? 0) === 0 &&
          !snapshot.latest_order_intent?.submission_allowed &&
          !snapshot.latest_risk_decision?.actual_submission &&
          (snapshot.simulation === null ||
            snapshot.simulation.summary.boundaries.simulation_only) &&
          !snapshot.simulation?.summary.boundaries.external_venue_connection &&
          !snapshot.simulation?.summary.boundaries.order_submission_allowed &&
          !snapshot.simulation?.summary.boundaries.order_mutation_allowed &&
          !snapshot.simulation?.summary.boundaries.automatic_retry_allowed &&
          !snapshot.simulation?.summary.boundaries
            .automatic_remediation_allowed &&
          !snapshot.simulation?.summary.boundaries.real_orders_submitted &&
          !snapshot.simulation?.summary.boundaries.trading_controls_enabled,
        "demo_run_snapshot.data.submission",
      );
      return payload;
    },

    async getRunMetrics(
      path: RunMetricsPath,
      signal?: AbortSignal,
    ): Promise<RunMetricsResponse> {
      const payload = await resolveResponse(
        getRunMetrics({ client, path, signal }),
        zRunMetricsResponse,
        "run_metrics",
      );
      assertIdentity(
        payload.data.run_id === path.run_id,
        "run_metrics.path.run_id",
      );
      return payload;
    },

    async getRunReport(
      path: RunReportPath,
      signal?: AbortSignal,
    ): Promise<RunReportResponse> {
      const payload = await resolveResponse(
        getRunReport({ client, path, signal }),
        zRunReportResponse,
        "run_report",
      );
      assertIdentity(
        payload.data.run_id === path.run_id,
        "run_report.path.run_id",
      );
      return payload;
    },

    async getRunAnalysis(
      path: RunAnalysisPath,
      signal?: AbortSignal,
    ): Promise<RunAnalysisResponse> {
      const payload = await resolveResponse(
        getRunAnalysis({ client, path, signal }),
        zRunAnalysisResponse,
        "run_analysis",
      );
      assertIdentity(
        payload.data.run_id === path.run_id,
        "run_analysis.path.run_id",
      );
      assertIdentity(
        payload.data.analysis_ref ===
          `artifact://backtests/${path.run_id}/analysis.json` &&
          payload.data.provenance.summary_ref ===
            `artifact://backtests/${path.run_id}/summary.json` &&
          payload.data.provenance.details_ref ===
            `artifact://backtests/${path.run_id}/details.json`,
        "run_analysis.data.provenance",
      );
      return payload;
    },

    async getRunReproductionProof(
      path: RunReproductionPath,
      signal?: AbortSignal,
    ): Promise<RunReproductionProofResponse> {
      const payload = await resolveResponse(
        getRunReproductionProof({ client, path, signal }),
        zRunReproductionProofResponse,
        "run_reproduction_proof",
      );
      assertIdentity(
        payload.data.reproduced_run_id === path.run_id &&
          payload.data.source_run_id !== path.run_id &&
          payload.data.proof_ref ===
            `artifact://backtests/${path.run_id}/reproduction.json` &&
          payload.data.input_equivalent &&
          payload.data.output_equivalent &&
          payload.data.user_initiated &&
          !payload.data.automatic_retry_allowed &&
          !payload.data.automatic_remediation_allowed,
        "run_reproduction_proof.data.identity",
      );
      assertReadOnlyBoundaries(
        payload.boundaries,
        "run_reproduction_proof.boundaries",
      );
      return payload;
    },
  };
}

export const productApi = createProductApiClient();
