import { C } from "../content";
import { Screenshot } from "./Screenshot";

export function HowItWorks() {
  return (
    <section className="section" id="how" aria-labelledby="how-title">
      <div className="section-head">
        <h2 id="how-title">{C.how.title}</h2>
        <p className="lead">{C.how.lead}</p>
      </div>
      <div className="how">
        <ol className="group steps">
          {C.how.steps.map((s, i) => (
            <li key={s.title} className="step">
              <span className="step-number" aria-hidden="true">{i + 1}</span>
              <div>
                <h3>{s.title}</h3>
                <p>{s.text}</p>
              </div>
            </li>
          ))}
        </ol>
        <div className="how-shot">
          <Screenshot name="export" alt={C.how.screenshotAlt} sizes="(min-width: 1128px) 620px, (min-width: 880px) 56vw, calc(100vw - 48px)" />
        </div>
      </div>
    </section>
  );
}
