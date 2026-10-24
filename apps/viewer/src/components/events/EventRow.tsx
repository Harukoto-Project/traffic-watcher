import { ChevronDown, ChevronRight } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { TableCell, TableRow } from "@/components/ui/table";
import { summarizeFields } from "@/lib/events";
import { formatDateTime } from "@/lib/format";
import type { MonitorEvent } from "@/types/monitor";

import { EventKindLabel } from "./EventKindLabel";

export function EventRow({ event }: { event: MonitorEvent }) {
  const [open, setOpen] = useState(false);
  const Chevron = open ? ChevronDown : ChevronRight;

  return (
    <>
      <TableRow
        className="cursor-pointer"
        onClick={() => setOpen((value) => !value)}
      >
        <TableCell className="w-8 pr-0">
          <Button
            variant="ghost"
            size="icon-xs"
            aria-expanded={open}
            aria-label={open ? "詳細を閉じる" : "詳細を開く"}
            onClick={(clickEvent) => {
              clickEvent.stopPropagation();
              setOpen((value) => !value);
            }}
          >
            <Chevron />
          </Button>
        </TableCell>
        <TableCell className="whitespace-nowrap tabular-nums">
          {formatDateTime(event.ts_ms)}
        </TableCell>
        <TableCell className="whitespace-nowrap">
          <EventKindLabel kind={event.kind} />
        </TableCell>
        <TableCell className="text-foreground whitespace-nowrap font-medium">
          {event.source}
        </TableCell>
        <TableCell className="w-full max-w-0 truncate font-mono text-xs">
          {summarizeFields(event.fields)}
        </TableCell>
      </TableRow>
      {open ? (
        <TableRow>
          <TableCell colSpan={5}>
            <pre className="bg-muted overflow-x-auto rounded-md p-3 text-xs">
              {JSON.stringify(event.fields, null, 2)}
            </pre>
          </TableCell>
        </TableRow>
      ) : null}
    </>
  );
}
