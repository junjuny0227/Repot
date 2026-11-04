import { useCallback, useEffect, useReducer, useRef } from "react";
import { availabilityFor, chooseRepotFile, detectOs, openSnapshot, presenceFor, runScan, scanMachine } from "./api";
import { initialState, reducer } from "./state";
import { currentTarget, defaultTarget } from "./logic";
import { errorMessage } from "./errors";
import { S, openErrorText } from "./strings";
import { useLocale } from "./i18n";
import { StartScreen } from "./screens/StartScreen";
import { ScanScreen } from "./screens/ScanScreen";
import { ReviewScreen } from "./screens/ReviewScreen";
import { ExportScreen } from "./screens/ExportScreen";
import { DoneScreen } from "./screens/DoneScreen";
import { updates } from "./updateService";
import { UpdateNotice, UpdateStatus } from "./UpdateSettings";
import { ExclamationCircle } from "./screens/icons";

export default function App() {
  // Subscribing re-renders every screen when the language changes.
  useLocale();
  const [state, dispatch] = useReducer(reducer, initialState);
  // The scan is async; read the destination at the moment it finishes, not when it started.
  const latest = useRef(state);
  latest.current = state;

  useEffect(() => {
    updates.start();
    detectOs()
      .then((source) => dispatch({ type: "init", source }))
      .catch((e: unknown) => dispatch({ type: "error", message: errorMessage(e) }));
  }, []);

  const startScan = useCallback(async () => {
    if (updates.busy()) return;
    updates.setScreen("scanning");
    dispatch({ type: "scanStarted" });
    try {
      const scan = await runScan((report) => dispatch({ type: "progress", report }));
      const target = currentTarget(latest.current);
      if (target === null) throw new Error(S.detectingOs);
      const avail = await availabilityFor(target);
      dispatch({ type: "scanFinished", scan, avail });
    } catch (e) {
      dispatch({ type: "restart" });
      dispatch({ type: "error", message: errorMessage(e) });
    }
  }, []);

  // Opening a Repot file skips the scan. A failure leaves the start screen as it was.
  const openFile = useCallback(async () => {
    if (updates.busy()) return;
    try {
      const path = await chooseRepotFile();
      if (path === null) return;
      const file = await openSnapshot(path);
      const target = currentTarget(latest.current);
      if (target === null) throw new Error(S.detectingOs);
      const avail = await availabilityFor(target);
      dispatch({ type: "fileOpened", file, avail });
    } catch (e) {
      dispatch({ type: "error", message: openErrorText(e) });
    }
  }, []);

  // Scan this machine and judge the opened file's items against it, for this machine's OS.
  const compareWithMachine = useCallback(async () => {
    if (updates.busy()) return;
    const now = latest.current;
    if (now.source === null) return;
    const kept = currentTarget(now);
    const target = kept !== null && JSON.stringify(kept.os) === JSON.stringify(now.source) ? kept : defaultTarget(now.source);
    dispatch({ type: "compareStarted" });
    try {
      await scanMachine((report) => dispatch({ type: "progress", report }));
      const [avail, presence] = await Promise.all([availabilityFor(target), presenceFor(target)]);
      dispatch({ type: "compareFinished", target: target.os, packages: target.packages, avail, presence });
    } catch (e) {
      dispatch({ type: "compareFailed" });
      dispatch({ type: "error", message: errorMessage(e) });
    }
  }, []);

  return (
    <main className={`app screen-${state.screen}`}>
      {state.error !== null && (
        <div className="error" role="alert"><ExclamationCircle /><strong>{S.errorTitle}</strong> <span>{state.error}</span></div>
      )}
      {state.screen === "start" && <StartScreen state={state} dispatch={dispatch} onScan={startScan} onOpen={openFile} />}
      {state.screen === "scanning" && <ScanScreen state={state} />}
      {state.screen === "review" && <ReviewScreen state={state} dispatch={dispatch} onCompare={compareWithMachine} />}
      {state.screen === "export" && <ExportScreen state={state} dispatch={dispatch} />}
      {state.screen === "done" && <DoneScreen state={state} dispatch={dispatch} />}
      <UpdateNotice screen={state.screen} />
      <UpdateStatus screen={state.screen} />
    </main>
  );
}
