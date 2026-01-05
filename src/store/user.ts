import { create } from "zustand";
import { persist } from "zustand/middleware";

import { getUserInfo, type UserInfo } from "@/service/user-info";
import { tauriAdapter } from "@/utils/tauri-adapter";
import { StoreNameMap } from "@shared/store";

interface UserState {
  user: UserInfo | null;
}

interface Action {
  updateUser: () => Promise<void>;
  clear: () => void;
}
export const useUser = create<UserState & Action>()(
  persist(
    set => ({
      user: null,
      updateUser: async () => {
        const res = await getUserInfo();

        if (res.code === 0 && res.data?.isLogin) {
          set(() => ({ user: res.data }));
        } else {
          set(() => ({ user: null }));
        }
      },
      clear: () => {
        set(() => ({
          user: null,
        }));
      },
    }),
    {
      name: "user",
      partialize: state => state.user,
      storage: {
        getItem: async () => {
          const store = await tauriAdapter.getStore(StoreNameMap.UserLoginInfo);

          return {
            state: store,
          };
        },

        setItem: async (_, value) => {
          if (value.state) {
            await tauriAdapter.setStore(StoreNameMap.UserLoginInfo, value.state);
          }
        },

        removeItem: async () => {
          await tauriAdapter.clearStore(StoreNameMap.UserLoginInfo);
        },
      },
    },
  ),
);
