import { useQuery } from "@tanstack/react-query";

import { fetchMvpStatus } from "../../api/mvpStatus";

export const mvpStatusQueryKey = ["mvp", "status"] as const;

export function useMvpStatus() {
  return useQuery({
    queryKey: mvpStatusQueryKey,
    queryFn: ({ signal }) => fetchMvpStatus(signal),
    retry: false,
    staleTime: 5_000,
    refetchInterval: 15_000,
    refetchIntervalInBackground: false,
  });
}
