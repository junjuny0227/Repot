import type { DownloadEvent } from '@tauri-apps/plugin-updater';
import { errorMessage } from './errors';
import type { Screen } from './state';

export interface UpdateAsset {
  version: string;
  body?: string;
  close(): Promise<void>;
  downloadAndInstall(event: (event: DownloadEvent) => void): Promise<void>;
}
interface Port {
  supported(): Promise<boolean>;
  check(): Promise<UpdateAsset | null>;
  restart(): Promise<void>;
  save(automatic: boolean): void;
}
export type UpdatePhase =
  | 'idle'
  | 'checking'
  | 'available'
  | 'current'
  | 'unsupported'
  | 'downloading'
  | 'restarting'
  | 'error';
export interface UpdateState {
  phase: UpdatePhase;
  automatic: boolean;
  version: string;
  notes: string;
  downloaded: number;
  total?: number;
  error: string;
  dismissed: boolean;
}
export const automaticUpdates = (saved: string | null): boolean => saved !== 'false';
export const supportsUpdater = (platform: string, appImage: boolean): boolean =>
  platform !== 'linux' || appImage;
export const canOfferUpdate = (screen: Screen): boolean => screen === 'start' || screen === 'done';

/** Independent of React and the transport, so async races and deferral are testable. */
export class UpdateController {
  private state: UpdateState;
  private screen: Screen = 'start';
  private asset: UpdateAsset | null = null;
  private listeners = new Set<() => void>();
  private started = false;
  constructor(
    private port: Port,
    automatic: boolean,
  ) {
    this.state = {
      phase: 'idle',
      automatic,
      version: '',
      notes: '',
      downloaded: 0,
      error: '',
      dismissed: false,
    };
  }
  getSnapshot = (): UpdateState => this.state;
  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };
  private change(patch: Partial<UpdateState>) {
    this.state = { ...this.state, ...patch };
    for (const listener of this.listeners) listener();
  }
  setScreen(screen: Screen) {
    this.screen = screen;
  }
  busy() {
    return this.state.phase === 'downloading' || this.state.phase === 'restarting';
  }
  shouldOffer() {
    return (
      canOfferUpdate(this.screen) &&
      !this.state.dismissed &&
      (this.state.phase === 'available' || this.busy())
    );
  }
  start() {
    if (this.started) return;
    this.started = true;
    if (this.state.automatic) void this.check();
  }
  setAutomatic(automatic: boolean) {
    this.port.save(automatic);
    this.change({ automatic });
  }
  later() {
    if (!this.busy()) this.change({ dismissed: true });
  }
  async check() {
    if (this.state.phase === 'checking' || this.busy()) return;
    this.change({ phase: 'checking', error: '', dismissed: false, version: '', notes: '' });
    try {
      if (this.asset) {
        await this.asset.close();
        this.asset = null;
      }
      if (!(await this.port.supported())) {
        this.change({ phase: 'unsupported' });
        return;
      }
      this.asset = await this.port.check();
      this.change(
        this.asset
          ? {
              phase: 'available',
              version: this.asset.version,
              notes: (this.asset.body ?? '').slice(0, 1200),
            }
          : { phase: 'current' },
      );
    } catch (error) {
      this.change({ phase: 'error', error: errorMessage(error) });
    }
  }
  async install() {
    if (!this.asset || this.state.phase !== 'available' || !canOfferUpdate(this.screen)) return;
    this.change({ phase: 'downloading', downloaded: 0, total: undefined, error: '' });
    try {
      await this.asset.downloadAndInstall((event) => {
        if (event.event === 'Started') this.change({ total: event.data.contentLength });
        if (event.event === 'Progress')
          this.change({ downloaded: this.state.downloaded + event.data.chunkLength });
      });
      this.change({ phase: 'restarting' });
      await this.port.restart();
    } catch (error) {
      this.change({ phase: 'error', error: errorMessage(error) });
    }
  }
}
