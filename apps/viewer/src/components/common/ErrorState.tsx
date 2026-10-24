import { RefreshCw } from "lucide-react";

import { Button } from "@/components/ui/button";
import type { AppError } from "@/services/AppError";

import { StatusText } from "./StatusText";

interface ErrorStateProps {
  error: AppError;
  onRetry?: () => void;
}

export function ErrorState({ error, onRetry }: ErrorStateProps) {
  return (
    <div
      role="alert"
      className="bg-muted flex flex-col items-start gap-2 rounded-md p-4 text-sm"
    >
      <StatusText level="critical">データを取得できませんでした</StatusText>
      <p className="text-muted-foreground text-xs break-all">{error.message}</p>
      <p>
        データベースのファイルが存在するかを確認して、もう一度操作してください。
      </p>
      {onRetry ? (
        <Button variant="outline" size="sm" onClick={onRetry}>
          <RefreshCw />
          再試行
        </Button>
      ) : null}
    </div>
  );
}
