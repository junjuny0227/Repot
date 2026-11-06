import appIcon from '../../../desktop/app-icon-macos.svg';
import { useContent } from '../content';
import { type DesktopOs, downloadsFor, type Platform, RELEASE_URL, VERSION } from '../downloads';
import { DownloadArrow, WarningTriangle } from '../icons';
import { CopyCommand } from './CopyCommand';
import { Screenshot } from './Screenshot';

// The CLI installs from a package manager, not from the download above, so it gets its own line.
function Cli({ os }: { os: DesktopOs }) {
  const C = useContent();
  return (
    <div className="hero-cli">
      <p>{C.hero.cli}</p>
      <CopyCommand command={C.hero.cliCommand[os]} />
    </div>
  );
}

function Buttons({ platform, macArch }: { platform: Platform; macArch: 'mac-arm' | 'mac-intel' }) {
  const C = useContent();
  if (platform === 'unknown') {
    return (
      <div className="downloads">
        <a className="button primary large" href="#install">
          {C.hero.unknown}
        </a>
        <p className="hint">{C.hero.unknownHint}</p>
      </div>
    );
  }
  if (platform === 'mac') {
    const mac = downloadsFor('mac');
    const ordered = macArch === 'mac-intel' ? [...mac].reverse() : mac;
    return (
      <div className="downloads">
        <p className="download-title" id="mac-download">
          {C.hero.downloadFor.mac}
        </p>
        {/* biome-ignore lint/a11y/useSemanticElements: a fieldset would change the layout of the link row. */}
        <div className="button-row" role="group" aria-labelledby="mac-download">
          {ordered.map((d, i) => (
            <a key={d.id} className={`button large${i === 0 ? ' primary' : ''}`} href={d.url}>
              <DownloadArrow />
              {d.id === 'mac-arm' ? C.hero.macArm : C.hero.macIntel}
            </a>
          ))}
        </div>
        <p className="hint">{C.hero.macHelp}</p>
        <Cli os="mac" />
      </div>
    );
  }
  const [first] = downloadsFor(platform);
  return (
    <div className="downloads">
      <div className="button-row">
        <a className="button primary large" href={first.url}>
          <DownloadArrow />
          {C.hero.downloadFor[platform]}
        </a>
      </div>
      <p className="hint">{C.hero.fileHint[platform]}</p>
      <Cli os={platform} />
    </div>
  );
}

export function Hero({
  platform,
  macArch,
}: {
  platform: Platform;
  macArch: 'mac-arm' | 'mac-intel';
}) {
  const C = useContent();
  return (
    <section className="hero" id="top" aria-labelledby="hero-title">
      <img className="hero-icon" src={appIcon} alt="" width={128} height={128} />
      <h1 id="hero-title">{C.name}</h1>
      <p className="tagline">{C.tagline}</p>
      <Buttons platform={platform} macArch={macArch} />
      <p className="meta">
        <span>{C.hero.version(VERSION)}</span>
        <span aria-hidden="true">·</span>
        <a href="#install">{C.hero.otherDownloads}</a>
        <span aria-hidden="true">·</span>
        <a href={RELEASE_URL}>{C.hero.releaseNotes}</a>
      </p>
      <p className="beta">
        <WarningTriangle />
        <span>
          <strong>{C.hero.beta}</strong>
          {C.gap}
          {C.hero.betaNote}
          {C.gap}
          <a href="#install">{C.hero.howToOpen}</a>
        </span>
      </p>
      <div className="hero-shot">
        <Screenshot
          name="review"
          alt={C.hero.screenshotAlt}
          sizes="(min-width: 1148px) 1100px, calc(100vw - 48px)"
          eager
        />
      </div>
    </section>
  );
}
