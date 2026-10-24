import { useMemo } from "react";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { eventKindOptions } from "@/config/eventKinds";
import type { EventFilter } from "@/hooks/use-events";
import { sourceGroups } from "@/lib/events";
import type { EventKind } from "@/types/monitor";

interface EventFilterBarProps {
  filter: EventFilter;
  sources: string[];
  onChange: (filter: EventFilter) => void;
}

const ALL = "all";
const LIMITS = [200, 1000, 5000];

export function EventFilterBar({
  filter,
  sources,
  onChange,
}: EventFilterBarProps) {
  const sourceOptions = useMemo(
    () => [...sourceGroups(sources), ...sources],
    [sources],
  );

  return (
    <div className="flex flex-wrap gap-2">
      <Select
        value={filter.source ?? ALL}
        onValueChange={(value) =>
          onChange({ ...filter, source: value === ALL ? null : value })
        }
      >
        <SelectTrigger className="w-48" aria-label="ソース">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value={ALL}>すべてのソース</SelectItem>
          {sourceOptions.map((source) => (
            <SelectItem key={source} value={source}>
              {source}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Select
        value={filter.kind ?? ALL}
        onValueChange={(value) =>
          onChange({
            ...filter,
            kind: value === ALL ? null : (value as EventKind),
          })
        }
      >
        <SelectTrigger className="w-40" aria-label="種別">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value={ALL}>すべての種別</SelectItem>
          {eventKindOptions.map((option) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Select
        value={String(filter.limit)}
        onValueChange={(value) => onChange({ ...filter, limit: Number(value) })}
      >
        <SelectTrigger className="w-36" aria-label="表示件数">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {LIMITS.map((limit) => (
            <SelectItem key={limit} value={String(limit)}>
              最新{limit}件
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
