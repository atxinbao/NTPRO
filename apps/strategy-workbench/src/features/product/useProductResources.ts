import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { productApi } from "../../api/productApi";
import type {
  CreateBacktestRunRequest,
  CreateDemoRunRequest,
  CreateLiveRunCandidateRequest,
  DemoRunAction,
  LiveExecutionAdmissionRequest,
  LiveRunCandidateAction,
} from "../../api/generated/productApi";

const productQueryPolicy = {
  staleTime: 0,
  gcTime: 0,
  refetchOnMount: "always" as const,
};

export const productQueryKeys = {
  allRuns: ["product", "runs"] as const,
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
  demoSnapshot: (runId: string) =>
    ["product", "runs", runId, "demo-snapshot"] as const,
  runMetrics: (runId: string) => ["product", "runs", runId, "metrics"] as const,
  runReport: (runId: string) => ["product", "runs", runId, "report"] as const,
  runAnalysis: (runId: string) =>
    ["product", "runs", runId, "analysis"] as const,
  runComparison: (runIds: string[]) =>
    ["product", "run-comparison", ...runIds] as const,
  runReproduction: (runId: string) =>
    ["product", "runs", runId, "reproduction"] as const,
  liveAdmission: (strategyId: string, versionId: string) =>
    [
      "product",
      "strategies",
      strategyId,
      "versions",
      versionId,
      "live-admission",
    ] as const,
  liveRunCandidates: ["product", "live-run-candidates"] as const,
};

export function useCreateBacktestRun() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (request: CreateBacktestRunRequest) =>
      productApi.createBacktestRun(request),
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: productQueryKeys.allRuns,
      });
    },
  });
}

export function useCreateDemoRun() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (request: CreateDemoRunRequest) =>
      productApi.createDemoRun(request),
    retry: false,
    onSuccess: (response) => {
      void Promise.all([
        queryClient.invalidateQueries({ queryKey: productQueryKeys.allRuns }),
        queryClient.invalidateQueries({
          queryKey: productQueryKeys.demoSnapshot(response.data.run_id),
        }),
        queryClient.invalidateQueries({
          queryKey: productQueryKeys.run(response.data.run_id),
        }),
      ]);
    },
  });
}

export function useCreateLiveRunCandidate() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (request: CreateLiveRunCandidateRequest) =>
      productApi.createLiveRunCandidate(request),
    retry: false,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: productQueryKeys.liveRunCandidates,
        refetchType: "active",
      });
    },
  });
}

export function useLiveRunCandidateAction() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      runId,
      action,
    }: {
      runId: string;
      action: LiveRunCandidateAction;
    }) => productApi.actOnLiveRunCandidate(runId, action),
    retry: false,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: productQueryKeys.liveRunCandidates,
        refetchType: "active",
      });
    },
  });
}

export function useLiveExecutionOwnerApproval() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      runId,
      request,
    }: {
      runId: string;
      request: LiveExecutionAdmissionRequest;
    }) => productApi.approveLiveExecutionAsOwner(runId, request),
    retry: false,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: productQueryKeys.liveRunCandidates,
        refetchType: "active",
      });
    },
  });
}

export function useLiveRunCandidates() {
  return useQuery({
    ...productQueryPolicy,
    queryKey: productQueryKeys.liveRunCandidates,
    queryFn: ({ signal }) => productApi.listLiveRunCandidates(signal),
    retry: false,
  });
}

export function useDemoRunAction() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ runId, action }: { runId: string; action: DemoRunAction }) =>
      productApi.actOnDemoRun(runId, action),
    retry: false,
    onSuccess: async (response) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: productQueryKeys.allRuns }),
        queryClient.invalidateQueries({
          queryKey: productQueryKeys.demoSnapshot(response.data.run_id),
        }),
        queryClient.setQueryData(productQueryKeys.run(response.data.run_id), {
          schema_version: "ntpro.product_api.run_detail.response.v1",
          contract_version: response.contract_version,
          request_id: response.request_id,
          data: response.data.current_run,
          boundaries: {
            read_only: true,
            strategy_mutation_allowed: false,
            run_mutation_allowed: false,
            external_venue_connection: false,
            order_submission_allowed: false,
            order_mutation_allowed: false,
            automatic_retry_allowed: false,
            automatic_remediation_allowed: false,
            real_orders_submitted: false,
            trading_controls_enabled: false,
          },
        }),
        queryClient.invalidateQueries({
          queryKey: productQueryKeys.run(response.data.run_id),
        }),
      ]);
    },
  });
}

export function useReproduceBacktestRun() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (sourceRunId: string) =>
      productApi.reproduceBacktestRun(sourceRunId),
    retry: false,
    onSuccess: async (response) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: productQueryKeys.allRuns }),
        queryClient.invalidateQueries({
          queryKey: productQueryKeys.run(response.data.source_run_id),
        }),
        queryClient.invalidateQueries({
          queryKey: productQueryKeys.run(response.data.reproduced_run.run_id),
        }),
      ]);
    },
  });
}

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

export function useLiveAdmission(strategyId?: string, versionId?: string) {
  return useQuery({
    ...productQueryPolicy,
    queryKey: productQueryKeys.liveAdmission(strategyId ?? "", versionId ?? ""),
    queryFn: ({ signal }) =>
      productApi.getLiveAdmission(
        { strategy_id: strategyId!, version_id: versionId! },
        signal,
      ),
    enabled: Boolean(strategyId && versionId),
  });
}

export function useRefreshLiveAccount() {
  return useMutation({
    mutationFn: ({
      strategyId,
      versionId,
    }: {
      strategyId: string;
      versionId: string;
    }) =>
      productApi.refreshLiveAccount({
        strategy_id: strategyId,
        version_id: versionId,
      }),
    retry: false,
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

export function useRunMetrics(runId?: string, enabled = false) {
  return useQuery({
    ...productQueryPolicy,
    queryKey: productQueryKeys.runMetrics(runId ?? ""),
    queryFn: ({ signal }) =>
      productApi.getRunMetrics({ run_id: runId! }, signal),
    enabled: Boolean(runId && enabled),
  });
}

export function useDemoRunSnapshot(runId?: string, enabled = false) {
  return useQuery({
    ...productQueryPolicy,
    queryKey: productQueryKeys.demoSnapshot(runId ?? ""),
    queryFn: ({ signal }) =>
      productApi.getDemoRunSnapshot({ run_id: runId! }, signal),
    enabled: Boolean(runId && enabled),
    refetchInterval: (query) =>
      query.state.data?.data.snapshot_status === "running" ? 2_000 : false,
  });
}

export function useRunReport(runId?: string, enabled = false) {
  return useQuery({
    ...productQueryPolicy,
    queryKey: productQueryKeys.runReport(runId ?? ""),
    queryFn: ({ signal }) =>
      productApi.getRunReport({ run_id: runId! }, signal),
    enabled: Boolean(runId && enabled),
  });
}

export function useRunAnalysis(runId?: string, enabled = false) {
  return useQuery({
    ...productQueryPolicy,
    queryKey: productQueryKeys.runAnalysis(runId ?? ""),
    queryFn: ({ signal }) =>
      productApi.getRunAnalysis({ run_id: runId! }, signal),
    enabled: Boolean(runId && enabled),
  });
}

export function useRunComparison(runIds: string[]) {
  return useQuery({
    ...productQueryPolicy,
    queryKey: productQueryKeys.runComparison(runIds),
    queryFn: ({ signal }) => productApi.compareRuns(runIds, signal),
    enabled:
      runIds.length >= 2 &&
      runIds.length <= 4 &&
      new Set(runIds).size === runIds.length,
  });
}

export function useRunReproductionProof(runId?: string, enabled = false) {
  return useQuery({
    ...productQueryPolicy,
    queryKey: productQueryKeys.runReproduction(runId ?? ""),
    queryFn: ({ signal }) =>
      productApi.getRunReproductionProof({ run_id: runId! }, signal),
    enabled: Boolean(runId && enabled),
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
  const expectsDemoSnapshot = run.data?.data.environment === "sandbox";
  const demoSnapshot = useDemoRunSnapshot(runId, expectsDemoSnapshot);
  const expectsMetrics = Boolean(
    run.data?.data.environment === "backtest" &&
    run.data.data.lifecycle === "completed" &&
    run.data.data.result.status === "available",
  );
  const metrics = useRunMetrics(runId, expectsMetrics);
  const expectsReport = Boolean(
    expectsMetrics && run.data?.data.result.report_ref,
  );
  const report = useRunReport(runId, expectsReport);
  const expectsAnalysis = Boolean(
    expectsMetrics && run.data?.data.result.analysis_ref,
  );
  const analysis = useRunAnalysis(runId, expectsAnalysis);
  const expectsReproductionProof = Boolean(
    expectsMetrics && run.data?.data.result.reproduction_ref,
  );
  const reproduction = useRunReproductionProof(runId, expectsReproductionProof);
  const error =
    run.error ??
    strategy.error ??
    version.error ??
    metrics.error ??
    demoSnapshot.error;
  const isVerifying = Boolean(
    runId &&
    (run.isPending ||
      run.isFetching ||
      (run.data &&
        (strategy.isPending ||
          strategy.isFetching ||
          version.isPending ||
          version.isFetching ||
          (expectsDemoSnapshot &&
            (demoSnapshot.isPending ||
              (!demoSnapshot.data && demoSnapshot.isFetching))) ||
          (expectsMetrics && (metrics.isPending || metrics.isFetching))))),
  );
  const isReady = Boolean(
    runId &&
    !error &&
    !isVerifying &&
    run.data &&
    strategy.data &&
    version.data &&
    (!expectsDemoSnapshot || demoSnapshot.data) &&
    (!expectsMetrics || metrics.data),
  );

  return {
    error,
    isReady,
    isVerifying,
    run: isReady ? run.data?.data : undefined,
    strategy: isReady ? strategy.data?.data : undefined,
    version: isReady ? version.data?.data : undefined,
    demoSnapshot:
      isReady && expectsDemoSnapshot ? demoSnapshot.data?.data : undefined,
    metrics: isReady ? metrics.data?.data : undefined,
    report: isReady && expectsReport ? report.data?.data : undefined,
    reportError: isReady && expectsReport ? report.error : null,
    isReportVerifying: Boolean(
      isReady && expectsReport && (report.isPending || report.isFetching),
    ),
    retryReport: report.refetch,
    analysis: isReady && expectsAnalysis ? analysis.data?.data : undefined,
    analysisError: isReady && expectsAnalysis ? analysis.error : null,
    isAnalysisVerifying: Boolean(
      isReady && expectsAnalysis && (analysis.isPending || analysis.isFetching),
    ),
    retryAnalysis: analysis.refetch,
    reproduction:
      isReady && expectsReproductionProof ? reproduction.data?.data : undefined,
    reproductionError:
      isReady && expectsReproductionProof ? reproduction.error : null,
    isReproductionVerifying: Boolean(
      isReady &&
      expectsReproductionProof &&
      (reproduction.isPending || reproduction.isFetching),
    ),
    retryReproduction: reproduction.refetch,
    requestId: isReady ? run.data?.request_id : undefined,
  };
}
