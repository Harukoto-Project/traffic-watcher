import { useState } from "react";

import { AsyncView } from "@/components/common/AsyncView";
import { PageHeader } from "@/components/common/PageHeader";
import { TableSkeleton } from "@/components/common/TableSkeleton";
import { EventFilterBar } from "@/components/events/EventFilterBar";
import { EventTable } from "@/components/events/EventTable";
import { getPageTitle } from "@/config/navigation";
import { useDatabase } from "@/hooks/use-database";
import { useEvents, type EventFilter } from "@/hooks/use-events";

const INITIAL_FILTER: EventFilter = { source: null, kind: null, limit: 200 };

export function EventsPage() {
  const { info } = useDatabase();
  const [filter, setFilter] = useState<EventFilter>(INITIAL_FILTER);
  const events = useEvents(filter);

  return (
    <>
      <PageHeader title={getPageTitle("events")} showRange>
        <EventFilterBar
          filter={filter}
          sources={info?.sources ?? []}
          onChange={setFilter}
        />
      </PageHeader>
      <AsyncView state={events} skeleton={<TableSkeleton rows={10} />}>
        {(data) => <EventTable events={data} />}
      </AsyncView>
    </>
  );
}
