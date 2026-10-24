import { EmptyState } from "@/components/common/EmptyState";
import { ScrollArea } from "@/components/ui/scroll-area";
import { formatDateTime } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { SnapshotInfo } from "@/types/monitor";

interface SnapshotListProps {
  snapshots: SnapshotInfo[];
  selectedId: number | null;
  onSelect: (id: number) => void;
}

export function SnapshotList({
  snapshots,
  selectedId,
  onSelect,
}: SnapshotListProps) {
  if (snapshots.length === 0) {
    return (
      <EmptyState
        title="スナップショットはありません"
        description="アラートが発生すると、前後の記録がここに保存されます。"
      />
    );
  }

  return (
    <ScrollArea className="h-72 lg:h-[calc(100svh-9rem)]">
      <ul className="flex flex-col gap-1 pr-3">
        {snapshots.map((snapshot) => {
          const selected = snapshot.id === selectedId;
          return (
            <li key={snapshot.id}>
              <button
                type="button"
                aria-current={selected ? "true" : undefined}
                onClick={() => onSelect(snapshot.id)}
                className={cn(
                  "hover:bg-accent flex w-full flex-col gap-0.5 rounded-md border-l-2 border-transparent py-2.5 pr-2 pl-3 text-left transition-colors duration-150",
                  selected && "border-foreground bg-accent",
                )}
              >
                <span className="text-foreground text-sm font-medium tabular-nums">
                  {formatDateTime(snapshot.trigger_ms)}
                </span>
                <span className="truncate text-sm">{snapshot.rule}</span>
                <span className="text-muted-foreground line-clamp-2 text-xs">
                  {snapshot.message}
                </span>
              </button>
            </li>
          );
        })}
      </ul>
    </ScrollArea>
  );
}
