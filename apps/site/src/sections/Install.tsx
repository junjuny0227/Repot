import { useState } from 'react';
import { useContent } from '../content';
import { type DesktopOs, downloadsFor, type Platform, RELEASES_URL } from '../downloads';
import { DownloadArrow } from '../icons';

const ORDER: DesktopOs[] = ['mac', 'windows', 'linux'];

export function Install({ platform }: { platform: Platform }) {
  const C = useContent();
  const [os, setOs] = useState<DesktopOs>(platform === 'unknown' ? 'mac' : platform);
  const steps = C.install.steps[os];
  return (
    <section className="section" id="install" aria-labelledby="install-title">
      <div className="section-head">
        <h2 id="install-title">{C.install.title}</h2>
        <p className="lead">{C.install.lead}</p>
      </div>
      <fieldset className="segmented">
        <legend className="visually-hidden">{C.install.osLabel}</legend>
        {ORDER.map((o) => (
          <label key={o}>
            <input
              type="radio"
              name="install-os"
              value={o}
              checked={os === o}
              onChange={() => setOs(o)}
            />
            <span>{C.install.os[o]}</span>
          </label>
        ))}
      </fieldset>
      <div className="install" aria-live="polite">
        <div>
          <h3 className="group-title">{C.install.downloads}</h3>
          <ul className="group files">
            {downloadsFor(os).map((d) => (
              <li key={d.id} className="file-row">
                <div className="file-label">
                  <span>{C.install.files[d.id]}</span>
                  <code>{d.file}</code>
                </div>
                <a className="button" href={d.url} aria-label={C.install.downloadFile(d.file)}>
                  <DownloadArrow />
                  {C.install.download}
                </a>
              </li>
            ))}
          </ul>
          <p className="footnote">
            <a href={RELEASES_URL}>{C.install.allReleases}</a>
          </p>
          <div className="footnote">
            <p>
              <strong>{C.install.cliTitle}</strong> {C.install.cliText}
            </p>
            <pre>
              <code>{C.install.cliCommand[os]}</code>
            </pre>
          </div>
        </div>
        <div>
          <h3 className="group-title">{C.install.firstLaunch}</h3>
          <ol className="group launch">
            {steps.map((s, i) => (
              <li key={s}>
                <span className="step-number small" aria-hidden="true">
                  {i + 1}
                </span>
                <div>
                  <p>{s}</p>
                  {os === 'linux' && i === 0 && (
                    <pre>
                      <code>{C.install.linuxCommand}</code>
                    </pre>
                  )}
                </div>
              </li>
            ))}
          </ol>
          {os === 'mac' && (
            <div className="footnote">
              <p>{C.install.macNote}</p>
              <pre>
                <code>{C.install.macCommand}</code>
              </pre>
            </div>
          )}
        </div>
      </div>
    </section>
  );
}
