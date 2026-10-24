import { EmptyState } from "@/components/common/EmptyState";
import { Section } from "@/components/common/Section";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  extractTopProcesses,
  findNearestEvent,
  type ProcessEntry,
  type ProcessRanking,
} from "@/lib/events";
import { formatNumber } from "@/lib/format";
import type { MonitorEvent } from "@/types/monitor";

interface ProcessRankingCardProps {
  events: MonitorEvent[];
  triggerMs: number;
}

function RankingTable({
  title,
  entries,
  ranking,
}: {
  title: string;
  entries: ProcessEntry[];
  ranking: ProcessRanking;
}) {
  return (
    <div className="flex min-w-0 flex-col gap-2">
      <h3 className="text-sm font-semibold">{title}</h3>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead className="w-10">順位</TableHead>
            <TableHead>プロセス</TableHead>
            <TableHead className="text-right">PID</TableHead>
            <TableHead className="text-right">
              {ranking === "cpu" ? "CPU" : "メモリ"}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {entries.map((entry) => (
            <TableRow key={entry.rank}>
              <TableCell className="tabular-nums">{entry.rank}</TableCell>
              <TableCell className="text-foreground max-w-48 truncate font-medium">
                {entry.name}
              </TableCell>
              <TableCell className="text-right tabular-nums">
                {entry.pid ?? "-"}
              </TableCell>
              <TableCell className="text-right tabular-nums">
                {ranking === "cpu"
                  ? `${formatNumber(entry.value)}%`
                  : `${formatNumber(entry.value, 0)}MB`}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

export function ProcessRankingCard({
  events,
  triggerMs,
}: ProcessRankingCardProps) {
  const event = findNearestEvent(events, "process", triggerMs);
  const cpu = event ? extractTopProcesses(event, "cpu") : [];
  const mem = event ? extractTopProcesses(event, "mem") : [];

  return (
    <Section title="プロセスの上位">
      {cpu.length === 0 && mem.length === 0 ? (
        <EmptyState
          title="プロセスの記録はありません"
          description="別のスナップショットを選んで確認してください。"
        />
      ) : (
        <div className="grid gap-8 lg:grid-cols-2">
          <RankingTable title="CPU使用率" entries={cpu} ranking="cpu" />
          <RankingTable title="メモリ使用量" entries={mem} ranking="mem" />
        </div>
      )}
    </Section>
  );
}
