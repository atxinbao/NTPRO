import { useQuery } from "@tanstack/react-query";

import { productApi } from "../../api/productApi";

const productQueryPolicy = {
  staleTime: 0,
  gcTime: 0,
  refetchOnMount: "always" as const,
};

export const productQueryKeys = {
  strategies: ["product", "strategies"] as const,
  strategy: (strategyId: string) =>
    ["product", "strategies", strategyId] as const,
  versions: (strategyId: string) =>
    ["product", "strategies", strategyId, "versions"] as const,
  version: (strategyId: string, versionId: string) =>
    ["product", "strategies", strategyId, "versions", versionId] as const,
  runs: (strategyId: string, versionId: string) =>
    ["product", "runs", { strategyId, versionId }] as const,
  run: (runId: string) => ["product", "runs", runId] as const,
};

export function useStrategies() {
  return useQuery({
    ...productQueryPolicy,
    queryKey: productQueryKeys.strategies,
    queryFn: ({ signal }) =>
      productApi.listStrategies(
        { limit: 20, sort: "updated_at", order: "desc" },
        signal,
      ),
  });
}

export function useStrategy(strategyId?: string) {
  return useQuery({
    ...productQueryPolicy,
    queryKey: productQueryKeys.strategy(strategyId ?? ""),
    queryFn: ({ signal }) =>
      productApi.getStrategy({ strategy_id: strategyId! }, signal),
    enabled: Boolean(strategyId),
  });
}

export function useStrategyVersions(strategyId?: string) {
  return useQuery({
    ...productQueryPolicy,
    queryKey: productQueryKeys.versions(strategyId ?? ""),
    queryFn: ({ signal }) =>
      productApi.listStrategyVersions(
        { strategy_id: strategyId! },
        { limit: 20, sort: "created_at", order: "desc" },
        signal,
      ),
    enabled: Boolean(strategyId),
  });
}

export function useStrategyVersion(strategyId?: string, versionId?: string) {
  return useQuery({
    ...productQueryPolicy,
    queryKey: productQueryKeys.version(strategyId ?? "", versionId ?? ""),
    queryFn: ({ signal }) =>
      productApi.getStrategyVersion(
        { strategy_id: strategyId!, version_id: versionId! },
        signal,
      ),
    enabled: Boolean(strategyId && versionId),
  });
}

export function useRuns(strategyId?: string, versionId?: string) {
  return useQuery({
    ...productQueryPolicy,
    queryKey: productQueryKeys.runs(strategyId ?? "", versionId ?? ""),
    queryFn: ({ signal }) =>
      productApi.listRuns(
        {
          limit: 50,
          sort: "updated_at",
          order: "desc",
          strategy_id: strategyId,
          strategy_version_id: versionId,
        },
        signal,
      ),
    enabled: Boolean(strategyId && versionId),
  });
}

export function useRun(runId?: string) {
  return useQuery({
    ...productQueryPolicy,
    queryKey: productQueryKeys.run(runId ?? ""),
    queryFn: ({ signal }) => productApi.getRun({ run_id: runId! }, signal),
    enabled: Boolean(runId),
  });
}

export function useOverviewProductContext() {
  const strategies = useStrategies();
  const strategyId = strategies.data?.data[0]?.strategy_id;
  const strategy = useStrategy(strategyId);
  const versionId = strategy.data?.data.default_version_id;
  const versions = useStrategyVersions(strategyId);
  const version = useStrategyVersion(strategyId, versionId);
  const runs = useRuns(strategyId, versionId);
  const error =
    strategies.error ??
    strategy.error ??
    versions.error ??
    version.error ??
    runs.error;
  const isVerifying =
    strategies.isPending ||
    strategies.isFetching ||
    (Boolean(strategyId) &&
      (strategy.isPending ||
        strategy.isFetching ||
        versions.isPending ||
        versions.isFetching)) ||
    (Boolean(versionId) &&
      (version.isPending ||
        version.isFetching ||
        runs.isPending ||
        runs.isFetching));
  const isReady =
    !error &&
    !isVerifying &&
    Boolean(strategies.data) &&
    (!strategyId ||
      Boolean(
        strategy.data &&
        versions.data &&
        versionId &&
        version.data &&
        runs.data,
      ));

  return {
    error,
    isReady,
    isVerifying,
    strategies: isReady ? strategies.data : undefined,
    strategy: isReady ? strategy.data?.data : undefined,
    versions: isReady ? versions.data : undefined,
    version: isReady ? version.data?.data : undefined,
    runs: isReady ? runs.data : undefined,
  };
}

export function useRunProductContext(runId?: string) {
  const run = useRun(runId);
  const strategyId = run.data?.data.strategy_id;
  const versionId = run.data?.data.strategy_version_id;
  const strategy = useStrategy(strategyId);
  const version = useStrategyVersion(strategyId, versionId);
  const error = run.error ?? strategy.error ?? version.error;
  const isVerifying = Boolean(
    runId &&
    (run.isPending ||
      run.isFetching ||
      (run.data &&
        (strategy.isPending ||
          strategy.isFetching ||
          version.isPending ||
          version.isFetching))),
  );
  const isReady = Boolean(
    runId &&
    !error &&
    !isVerifying &&
    run.data &&
    strategy.data &&
    version.data,
  );

  return {
    error,
    isReady,
    isVerifying,
    run: isReady ? run.data?.data : undefined,
    strategy: isReady ? strategy.data?.data : undefined,
    version: isReady ? version.data?.data : undefined,
    requestId: isReady ? run.data?.request_id : undefined,
  };
}
