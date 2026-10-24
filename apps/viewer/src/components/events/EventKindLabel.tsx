import { StatusText } from "@/components/common/StatusText";
import { EVENT_KIND_LABELS, EVENT_KIND_LEVELS } from "@/config/eventKinds";
import type { EventKind } from "@/types/monitor";

export function EventKindLabel({ kind }: { kind: EventKind }) {
  return (
    <StatusText level={EVENT_KIND_LEVELS[kind]}>
      {EVENT_KIND_LABELS[kind]}
    </StatusText>
  );
}
