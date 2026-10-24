import { EmptyState } from "@/components/common/EmptyState";
import { Section } from "@/components/common/Section";
import { StatusText } from "@/components/common/StatusText";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { lossLevel } from "@/config/status";
import { formatCount, formatMs, formatPercent } from "@/lib/format";
import type { SourceSummary } from "@/types/monitor";

export function PingSummaryTable({
  summaries,
}: {
  summaries: SourceSummary[];
}) {
  return (
    <Section title="pingの統計">
      {summaries.length === 0 ? (
        <EmptyState
          title="この期間のpingデータはありません"
          description="期間を広げて、もう一度確認してください。"
        />
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>宛先</TableHead>
              <TableHead className="text-right">送信数</TableHead>
              <TableHead className="text-right">ロス</TableHead>
              <TableHead className="text-right">平均</TableHead>
              <TableHead className="text-right">P95</TableHead>
              <TableHead className="text-right">P99</TableHead>
              <TableHead className="text-right">最大</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {summaries.map((summary) => (
              <TableRow key={summary.source}>
                <TableCell className="text-foreground font-medium">
                  {summary.source.replace(/^ping:/, "")}
                </TableCell>
                <TableCell className="text-right tabular-nums">
                  {formatCount(summary.samples)}
                </TableCell>
                <TableCell className="text-right tabular-nums">
                  <StatusText level={lossLevel(summary.loss_pct)}>
                    {formatCount(summary.lost)}(
                    {formatPercent(summary.loss_pct, 2)})
                  </StatusText>
                </TableCell>
                <TableCell className="text-right tabular-nums">
                  {formatMs(summary.avg_ms)}
                </TableCell>
                <TableCell className="text-right tabular-nums">
                  {formatMs(summary.p95_ms)}
                </TableCell>
                <TableCell className="text-right tabular-nums">
                  {formatMs(summary.p99_ms)}
                </TableCell>
                <TableCell className="text-right tabular-nums">
                  {formatMs(summary.max_ms)}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </Section>
  );
}
