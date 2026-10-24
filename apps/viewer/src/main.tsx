import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import "./index.css";
import App from "./App.tsx";
import { DatabaseProvider } from "./contexts/DatabaseProvider.tsx";
import { ThemeProvider } from "./contexts/ThemeProvider.tsx";
import { TimeRangeProvider } from "./contexts/TimeRangeProvider.tsx";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ThemeProvider>
      <DatabaseProvider>
        <TimeRangeProvider>
          <App />
        </TimeRangeProvider>
      </DatabaseProvider>
    </ThemeProvider>
  </StrictMode>,
);
