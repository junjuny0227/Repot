import { useCallback, useEffect, useReducer, useRef } from "react";
import { availabilityFor, detectOs, runScan } from "./api";
import { initialState, reducer } from "./state";
import { errorMessage } from "./errors";
import { S } from "./strings";
import { StartScreen } from "./screens/StartScreen";
import { ScanScreen } from "./screens/ScanScreen";
import { ReviewScreen } from "./screens/ReviewScreen";
import { ExportScreen } from "./screens/ExportScreen";
import { DoneScreen } from "./screens/DoneScreen";
import { ExclamationCircle } from "./screens/icons";


export default function App() {
  const [state, dispatch] = useReducer(reducer, initialState);
  // The scan is async; read the destination at the moment it finishes, not when it started.
  const latest = useRef(state);
  latest.current = state;

  useEffect(() => {
    detectOs()
      .then((source) => dispatch({ type: "init", source }))
      .catch((e: unknown) => dispatch({ type: "error", message: errorMessage(e) }));
  }, []);

  const startScan = useCallback(async () => {
    dispatch({ type: "scanStarted" });
    try {
      const scan = await runScan((report) => dispatch({ type: "progress", report }));
      const target = latest.current.target ?? latest.current.source;
      if (target === null) throw new Error(S.detectingOs);
      const avail = await availabilityFor(target);
      dispatch({ type: "scanFinished", scan, avail });
    } catch (e) {
      dispatch({ type: "restart" });
      dispatch({ type: "error", message: errorMessage(e) });
    }
  }, []);

  return (
    <main className={`app screen-${state.screen}`}>
      {state.error !== null && (
        <div className="error" role="alert"><ExclamationCircle /><strong>{S.errorTitle}</strong> <span>{state.error}</span></div>
      )}
      {state.screen === "start" && <StartScreen state={state} dispatch={dispatch} onScan={startScan} />}
      {state.screen === "scanning" && <ScanScreen state={state} />}
      {state.screen === "review" && <ReviewScreen state={state} dispatch={dispatch} />}
      {state.screen === "export" && <ExportScreen state={state} dispatch={dispatch} />}
      {state.screen === "done" && <DoneScreen state={state} dispatch={dispatch} />}
    </main>
  );
}
