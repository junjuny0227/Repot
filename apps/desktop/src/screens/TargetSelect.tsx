import { osKey, targetOptions } from "../logic";
import { S, osLabel } from "../strings";
import type { Os } from "../types";
import { Chevrons } from "./icons";

export function TargetSelect({ value, onChange, disabled, className = "" }: { value: Os | null; onChange: (os: Os) => void; disabled?: boolean; className?: string }) {
  const options = targetOptions(value);
  const off = disabled === true || value === null;
  return (
    <label className={`popup-field ${className}`}>
      <span className="popup-label">{S.targetOs}</span>
      <span className={`popup ${off ? "disabled" : ""}`}>
        <select
          value={value === null ? "" : osKey(value)}
          disabled={off}
          onChange={(e) => {
            const next = options.find((o) => osKey(o) === e.target.value);
            if (next) onChange(next);
          }}
        >
          {value === null && <option value="">{S.detectingOs}</option>}
          {options.map((o) => <option key={osKey(o)} value={osKey(o)}>{osLabel(o)}</option>)}
        </select>
        <Chevrons />
      </span>
    </label>
  );
}
