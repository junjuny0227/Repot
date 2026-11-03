import { WINDOWS_PACKAGE_MANAGERS, osKey, targetOptions } from "../logic";
import { S, managerLabel, osLabel } from "../strings";
import type { Os, PackageManager } from "../types";
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

/** Windows destination package manager (winget, Scoop, Chocolatey), shown next to the destination. */
export function PackageSelect({ value, onChange }: { value: PackageManager | null; onChange: (m: PackageManager) => void }) {
  return (
    <label className="popup-field package-field">
      <span className="popup-label">{S.packageManager}</span>
      <span className="popup">
        <select
          value={value ?? ""}
          onChange={(e) => {
            const next = WINDOWS_PACKAGE_MANAGERS.find((m) => m === e.target.value);
            if (next) onChange(next);
          }}
        >
          {WINDOWS_PACKAGE_MANAGERS.map((m) => <option key={m} value={m}>{managerLabel(m)}</option>)}
        </select>
        <Chevrons />
      </span>
    </label>
  );
}
