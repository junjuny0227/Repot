import { useEffect, useState } from 'react';
import { useContent } from '../content';
import { Checkmark, CopyGlyph } from '../icons';

export type CopyStatus = 'idle' | 'copied' | 'failed';

/** Writes the command to the clipboard; any refusal (no API, no permission) is a failure. */
export async function copyCommand(
  clipboard: Pick<Clipboard, 'writeText'> | undefined,
  command: string,
): Promise<Exclude<CopyStatus, 'idle'>> {
  try {
    if (!clipboard) return 'failed';
    await clipboard.writeText(command);
    return 'copied';
  } catch {
    return 'failed';
  }
}

/** How long the button says "Copied" before it offers to copy again. */
export const COPIED_MS = 2000;

/**
 * A command in a code block with a copy button. Success is quiet (a checkmark and a neutral
 * announcement); only a failure is shown in red, with what to do instead.
 */
export function CopyCommand({
  command,
  initialStatus = 'idle',
}: {
  command: string;
  initialStatus?: CopyStatus;
}) {
  const C = useContent();
  const [status, setStatus] = useState<CopyStatus>(initialStatus);
  useEffect(() => {
    if (status !== 'copied') return;
    const timer = setTimeout(() => setStatus('idle'), COPIED_MS);
    return () => clearTimeout(timer);
  }, [status]);
  return (
    <div className={`command ${status}`}>
      <div className="command-box">
        <code>{command}</code>
        <button
          type="button"
          className="button"
          aria-label={C.copy.label}
          onClick={async () =>
            setStatus(await copyCommand(globalThis.navigator?.clipboard, command))
          }
        >
          {status === 'copied' ? <Checkmark /> : <CopyGlyph />}
          <span aria-hidden="true">{status === 'copied' ? C.copy.copied : C.copy.copy}</span>
        </button>
      </div>
      <p className="copy-status" role="status">
        {status === 'copied' ? (
          <span className="visually-hidden">{C.copy.copiedStatus}</span>
        ) : status === 'failed' ? (
          C.copy.failedStatus
        ) : null}
      </p>
    </div>
  );
}
