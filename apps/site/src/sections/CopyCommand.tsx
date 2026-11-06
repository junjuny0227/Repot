import { useEffect, useState } from 'react';
import { useContent } from '../content';

// A terminal command with a copy button. The command stays selectable, so a browser
// without clipboard access (or a plain-HTTP preview) can still copy it by hand.
export function CopyCommand({ command }: { command: string }) {
  const C = useContent();
  const [status, setStatus] = useState<'idle' | 'copied' | 'failed'>('idle');

  useEffect(() => {
    if (status !== 'copied') return;
    const timer = setTimeout(() => setStatus('idle'), 2000);
    return () => clearTimeout(timer);
  }, [status]);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(command);
      setStatus('copied');
    } catch {
      setStatus('failed');
    }
  };

  return (
    <div className="command-wrap">
      <div className="command">
        <pre>
          <code>{command}</code>
        </pre>
        <button type="button" onClick={copy}>
          {status === 'copied' ? C.copy.copied : C.copy.copy}
        </button>
      </div>
      {/* Announces the result; "Copied" is already visible on the button, so only a failure shows here. */}
      <p className={status === 'failed' ? 'copy-status' : 'visually-hidden'} role="status">
        {status === 'copied' ? C.copy.copied : status === 'failed' ? C.copy.failed : ''}
      </p>
    </div>
  );
}
