import { EmptyState } from "@/components/common/EmptyState";
import {
  Table,
  TableBody,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { MonitorEvent } from "@/types/monitor";

import { EventRow } from "./EventRow";

export function EventTable({ events }: { events: MonitorEvent[] }) {
  if (events.length === 0) {
    return (
      <EmptyState
        title="条件に合うイベントはありません"
        description="ソースや種別の条件を変更するか、期間を広げてください。"
      />
    );
  }

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead className="w-8">
            <span className="sr-only">詳細</span>
          </TableHead>
          <TableHead>時刻</TableHead>
          <TableHead>種別</TableHead>
          <TableHead>ソース</TableHead>
          <TableHead>内容</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {events.map((event, index) => (
          <EventRow
            key={`${event.ts_ms}-${event.source}-${index}`}
            event={event}
          />
        ))}
      </TableBody>
    </Table>
  );
}
