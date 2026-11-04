import { describe, expect, it, vi } from 'vitest';
import {
  automaticUpdates,
  canOfferUpdate,
  supportsUpdater,
  type UpdateAsset,
  UpdateController,
} from '../updater';

function setup(supported = true) {
  const asset: UpdateAsset = {
    version: '0.1.99',
    body: 'Notes',
    close: vi.fn(async () => {}),
    downloadAndInstall: vi.fn(async (event) => {
      event({ event: 'Started', data: { contentLength: 100 } });
      event({ event: 'Progress', data: { chunkLength: 50 } });
      event({ event: 'Finished' });
    }),
  };
  const port = {
    supported: vi.fn(async () => supported),
    check: vi.fn(async (): Promise<UpdateAsset | null> => asset),
    restart: vi.fn(async () => {}),
    save: vi.fn(),
  };
  const controller = new UpdateController(port, true);
  return { controller, port, asset };
}

describe('updater', () => {
  it('defaults on, and persists an explicit opt-out', () => {
    expect(automaticUpdates(null)).toBe(true);
    expect(automaticUpdates('false')).toBe(false);
    const { controller, port } = setup();
    controller.setAutomatic(false);
    expect(port.save).toHaveBeenCalledWith(false);
    expect(controller.getSnapshot().automatic).toBe(false);
  });
  it('checks once on launch and respects automatic off', async () => {
    const { controller, port } = setup();
    controller.start();
    controller.start();
    await vi.waitFor(() => expect(port.check).toHaveBeenCalledTimes(1));
    const off = setup();
    off.controller.setAutomatic(false);
    off.controller.start();
    expect(off.port.supported).not.toHaveBeenCalled();
  });
  it('only offers updates at start or done, never during scan/review/export', () => {
    for (const screen of ['scanning', 'review', 'export'] as const)
      expect(canOfferUpdate(screen)).toBe(false);
    expect(canOfferUpdate('start')).toBe(true);
    expect(canOfferUpdate('done')).toBe(true);
  });
  it('defers a result and installation until a safe screen', async () => {
    const { controller, port } = setup();
    controller.setScreen('scanning');
    await controller.check();
    expect(controller.getSnapshot().phase).toBe('available');
    expect(controller.shouldOffer()).toBe(false);
    await controller.install();
    expect(port.restart).not.toHaveBeenCalled();
    controller.setScreen('done');
    expect(controller.shouldOffer()).toBe(true);
    controller.later();
    expect(controller.shouldOffer()).toBe(false);
  });
  it('never calls updater for deb/rpm; AppImage is supported', async () => {
    expect(supportsUpdater('linux', false)).toBe(false);
    expect(supportsUpdater('linux', true)).toBe(true);
    expect(supportsUpdater('macos', false)).toBe(true);
    const { controller, port } = setup(false);
    await controller.check();
    expect(controller.getSnapshot().phase).toBe('unsupported');
    expect(port.check).not.toHaveBeenCalled();
  });
  it('ignores duplicate checks, releases a dismissed asset, and allows manual recheck', async () => {
    const { controller, port, asset } = setup();
    await Promise.all([controller.check(), controller.check()]);
    expect(port.check).toHaveBeenCalledTimes(1);
    controller.later();
    await controller.check();
    expect(asset.close).toHaveBeenCalledTimes(1);
    expect(controller.shouldOffer()).toBe(true);
  });
  it('reports progress, blocks duplicate installs and restarts only after success', async () => {
    const { controller, port, asset } = setup();
    await controller.check();
    const seen: number[] = [];
    controller.subscribe(() => seen.push(controller.getSnapshot().downloaded));
    await Promise.all([controller.install(), controller.install()]);
    expect(asset.downloadAndInstall).toHaveBeenCalledTimes(1);
    expect(seen).toContain(50);
    expect(controller.getSnapshot().phase).toBe('restarting');
    expect(port.restart).toHaveBeenCalledTimes(1);
  });
  it('recovers from check failure; no update is not an error', async () => {
    const { controller, port } = setup();
    port.check.mockRejectedValueOnce(new Error('offline'));
    await controller.check();
    expect(controller.getSnapshot().phase).toBe('error');
    port.check.mockResolvedValueOnce(null);
    await controller.check();
    expect(controller.getSnapshot().phase).toBe('current');
  });
  it('never restarts after download/signature failure', async () => {
    const { controller, port, asset } = setup();
    vi.mocked(asset.downloadAndInstall).mockRejectedValueOnce(new Error('invalid signature'));
    await controller.check();
    await controller.install();
    expect(controller.getSnapshot().phase).toBe('error');
    expect(controller.getSnapshot().error).toContain('invalid signature');
    expect(port.restart).not.toHaveBeenCalled();
  });
});
