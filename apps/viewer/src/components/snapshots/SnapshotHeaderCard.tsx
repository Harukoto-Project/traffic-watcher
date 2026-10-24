import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { formatCount, formatDateTime, formatDuration } from "@/lib/format";
import type { SnapshotInfo } from "@/types/monitor";

function Field({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex min-w-0 flex-col gap-0.5">
      <dt className="text-muted-foreground text-sm">{label}</dt>
      <dd className="text-foreground text-sm font-medium tabular-nums">
        {value}
      </dd>
    </div>
  );
}

export function SnapshotHeaderCard({ info }: { info: SnapshotInfo }) {
  const windowMs = info.window_end_ms - info.window_start_ms;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="font-semibold">{info.rule}</CardTitle>
        <CardDescription className="break-words">
          {info.message}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <dl className="grid grid-cols-2 gap-x-6 gap-y-4 xl:grid-cols-4">
          <Field label="発生時刻" value={formatDateTime(info.trigger_ms)} />
          <Field label="ソース" value={info.source} />
          <Field
            label="保存範囲"
            value={`${formatDateTime(info.window_start_ms)} 〜 ${formatDateTime(info.window_end_ms)}`}
          />
          <Field
            label="イベント数"
            value={`${formatCount(info.event_count)}件(${formatDuration(windowMs)})`}
          />
        </dl>
      </CardContent>
    </Card>
  );
}
