import { createClient } from "./generated/productApi/client";
import {
  getRun,
  getStrategy,
  getStrategyVersion,
  listRuns,
  listStrategies,
  listStrategyVersions,
  type GetRunData,
  type GetStrategyData,
  type GetStrategyVersionData,
  type ListRunsData,
  type ListStrategiesData,
  type ListStrategyVersionsData,
  type ProductErrorResponse,
  type RunDetailResponse,
  type RunListResponse,
  type StrategyDetailResponse,
  type StrategyListResponse,
  type StrategyVersionDetailResponse,
  type StrategyVersionListResponse,
} from "./generated/productApi";
import {
  zProductErrorResponse,
  zRunDetailResponse,
  zRunListResponse,
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
  };
}

export const productApi = createProductApiClient();
