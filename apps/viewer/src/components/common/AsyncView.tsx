import type { ReactNode } from "react";

import type { AsyncState } from "@/hooks/use-async-data";
import { cn } from "@/lib/utils";

import { ErrorState } from "./ErrorState";

interface AsyncViewProps<T> {
  state: AsyncState<T>;
  skeleton: ReactNode;
  children: (data: T) => ReactNode;
}

export function AsyncView<T>({ state, skeleton, children }: AsyncViewProps<T>) {
  if (state.data === null) {
    if (state.error) {
      return <ErrorState error={state.error} onRetry={state.reload} />;
    }
    return <>{skeleton}</>;
  }

  return (
    <div
      className={cn(
        "flex flex-col gap-10 transition-opacity duration-150",
        state.loading && "opacity-60",
      )}
    >
      {state.error ? (
        <ErrorState error={state.error} onRetry={state.reload} />
      ) : null}
      {children(state.data)}
    </div>
  );
}
