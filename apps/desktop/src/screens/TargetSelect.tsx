import { osKey, targetOptions } from "../logic";
import { S, osLabel } from "../strings";
import type { Os } from "../types";

export function TargetSelect({ value, onChange, disabled }: { value: Os | null; onChange: (os: Os) => void; disabled?: boolean }) {
  const options = targetOptions(value);
  return (
    <label className="field">
      <span>{S.targetOs}</span>
      <select
        value={value === null ? "" : osKey(value)}
        disabled={disabled || value === null}
        onChange={(e) => {
          const next = options.find((o) => osKey(o) === e.target.value);
          if (next) onChange(next);
        }}
      >
        {value === null && <option value="">{S.detectingOs}</option>}
        {options.map((o) => <option key={osKey(o)} value={osKey(o)}>{osLabel(o)}</option>)}
      </select>
    </label>
  );
}
