import { invoke } from '@tauri-apps/api/core';
import { relaunch } from '@tauri-apps/plugin-process';
import { check } from '@tauri-apps/plugin-updater';
import { automaticUpdates, supportsUpdater, UpdateController } from './updater';

const key = 'repot.updates.automatic';
function saved() {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}
export const updates = new UpdateController(
  {
    supported: async () => {
      const platform = await invoke<{ platform: string; appImage: boolean }>('update_platform');
      return supportsUpdater(platform.platform, platform.appImage);
    },
    check: () => check({ timeout: 15000 }),
    restart: relaunch,
    save: (automatic) => {
      try {
        localStorage.setItem(key, String(automatic));
      } catch {
        // Unavailable storage: preference lasts for this session, like the language choice.
      }
    },
  },
  automaticUpdates(saved()),
);
