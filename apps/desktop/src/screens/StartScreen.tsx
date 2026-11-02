import type { Dispatch } from "react";
import type { Action, State } from "../state";
import { S, osLabel } from "../strings";
import { TargetSelect } from "./TargetSelect";

export function StartScreen({ state, dispatch, onScan }: { state: State; dispatch: Dispatch<Action>; onScan: () => void }) {
  const ready = state.source !== null && state.target !== null;
  return (
    <section className="screen start">
      <h1>{S.appTitle}</h1>
      <p className="muted">{S.startIntro}</p>
      <div className="card form">
        <div className="field">
          <span>{S.currentOs}</span>
          <strong>{state.source === null ? S.detectingOs : osLabel(state.source)}</strong>
        </div>
        <TargetSelect value={state.target} onChange={(target) => dispatch({ type: "setTarget", target })} />
        <button className="primary" disabled={!ready} onClick={onScan}>{S.scanButton}</button>
      </div>
    </section>
  );
}
