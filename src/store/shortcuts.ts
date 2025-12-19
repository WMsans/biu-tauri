import { create } from "zustand";
import { persist } from "zustand/middleware";

import { tauriAdapter } from "@/utils/tauri-adapter";
import { defaultShortcutSettings } from "@shared/settings/shortcut-settings";
import { StoreNameMap } from "@shared/store";

interface ShortcutActions {
  refresh: () => Promise<void>;
  update: (patch: Partial<ShortcutSettings>) => void;
  reset: () => void;
}

export const useShortcutSettings = create<ShortcutSettings & ShortcutActions>()(
  persist(
    set => ({
      ...defaultShortcutSettings,
      refresh: async () => {
        const store = await tauriAdapter.getStore(StoreNameMap.ShortcutSettings);
        if (store) {
          set(store);
        }
      },
      update: (patch: Partial<ShortcutSettings>) => {
        set(patch);
      },
      reset: () => {
        set(defaultShortcutSettings);
      },
    }),
    {
      name: "shortcut-settings",
      storage: {
        getItem: async () => {
          const store = await tauriAdapter.getStore(StoreNameMap.ShortcutSettings);

          return {
            state: store,
          };
        },

        setItem: async (_, value) => {
          if (value.state) {
            await tauriAdapter.setStore(StoreNameMap.ShortcutSettings, value.state);
          }
        },

        removeItem: async () => {
          await tauriAdapter.clearStore(StoreNameMap.ShortcutSettings);
        },
      },
      partialize: state => ({
        shortcuts: state.shortcuts,
        globalShortcuts: state.globalShortcuts,
        enableGlobalShortcuts: state.enableGlobalShortcuts,
      }),
    },
  ),
);
