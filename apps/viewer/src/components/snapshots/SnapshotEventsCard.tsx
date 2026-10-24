import { EventTable } from "@/components/events/EventTable";
import { Section } from "@/components/common/Section";
import type { MonitorEvent } from "@/types/monitor";

export function SnapshotEventsCard({ events }: { events: MonitorEvent[] }) {
  return (
    <Section title="保存されたイベント">
      <div className="max-h-96 overflow-auto">
        <EventTable events={events} />
      </div>
    </Section>
  );
}
