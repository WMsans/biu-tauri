import { addToast } from "@heroui/react";

import type { PlayMode } from "@/common/constants/audio";

import { usePlayList } from "@/store/play-list";
import { usePlayProgress } from "@/store/play-progress";
import { tauriAdapter } from "@/utils/tauri-adapter";

export type MiniPlayerCommandFromMini = "init" | "seek" | "togglePlayMode" | "next" | "prev" | "togglePlay";

export interface MiniPlayerMainStateSnapshot {
  isSingle: boolean;
  isPlaying: boolean;
  title?: string;
  cover?: string;
  currentTime: number;
  duration: number;
  playMode?: PlayMode;
  playId?: string;
}

export interface MiniPlayerMessageFromMini {
  from: "mini";
  data?: {
    type: MiniPlayerCommandFromMini;
    state?: any;
  };
  ts?: number;
}

export interface MiniPlayerMessageFromMain {
  from: "main";
  state: Partial<MiniPlayerMainStateSnapshot>;
  ts: number;
}

export interface MiniPlayerMainSyncState {
  isBroadcasting: boolean;
}

let bc: BroadcastChannel | null = null;
let unsubscribePlayList: VoidFunction | null = null;
let unsubscribePlayProgress: VoidFunction | null = null;
let isBroadcasting = false;

// Track the last sent state to avoid redundant messages and calculate diffs
let lastSentState: Partial<MiniPlayerMainStateSnapshot> = {};

export function createBroadcastChannel() {
  return new BroadcastChannel("play-list-store-sync-channel");
}

function getMainStateSnapshot(): MiniPlayerMainStateSnapshot {
  const { list, isPlaying, playMode, duration, playId, getPlayItem } = usePlayList.getState();
  const currentTime = usePlayProgress.getState().currentTime;
  const playItem = getPlayItem();

  return {
    isSingle: list.length === 1,
    title: playItem?.pageTitle || playItem?.title,
    cover: playItem?.pageCover || playItem?.cover,
    playId,
    isPlaying,
    currentTime: Number(currentTime ?? 0),
    playMode,
    duration: Number(duration ?? 0),
  };
}

function postMainState(channel: BroadcastChannel, partialState?: Partial<MiniPlayerMainStateSnapshot>) {
  const fullState = getMainStateSnapshot();
  const stateToSend = partialState || fullState;

  // Update local tracker
  lastSentState = { ...lastSentState, ...stateToSend };

  const message: MiniPlayerMessageFromMain = {
    from: "main",
    state: stateToSend,
    ts: Date.now(),
  };
  channel.postMessage(message);
}

function handleMessageFromMini(message: MiniPlayerMessageFromMini, channel: BroadcastChannel) {
  const data = message.data;
  if (!data) return;

  const type = data.type;
  switch (type) {
    case "init": {
      // Send full state on init
      postMainState(channel);
      break;
    }
    case "seek": {
      const t = data.state?.currentTime;
      if (typeof t === "number" && Number.isFinite(t)) {
        usePlayList.getState().seek(t);
      }
      break;
    }
    case "togglePlayMode": {
      usePlayList.getState().togglePlayMode();
      break;
    }
    case "next": {
      void usePlayList.getState().next();
      break;
    }
    case "prev": {
      void usePlayList.getState().prev();
      break;
    }
    case "togglePlay": {
      usePlayList.getState().togglePlay();
      break;
    }
    default: {
      break;
    }
  }
}

/**
 * 启动主窗口 -> mini 播放器的状态同步通道。
 *
 * - 只会启动一次；重复调用会直接返回。
 * - 收到 mini 端的 `init/seek/next/prev/togglePlay/togglePlayMode` 会转发到主播放状态。
 * - 主播放状态发生变化会推送给 mini 端更新 UI。
 */
export function startMiniPlayerMainSync() {
  if (isBroadcasting) return;

  bc = createBroadcastChannel();
  isBroadcasting = true;
  lastSentState = getMainStateSnapshot(); // Initialize with current state

  bc.onmessage = ev => {
    const data = ev.data as MiniPlayerMessageFromMini;
    if (data?.from !== "mini") return;
    handleMessageFromMini(data, bc as BroadcastChannel);
  };

  unsubscribePlayList = usePlayList.subscribe(() => {
    // "title" and "cover" are derived from playId/getPlayItem, so playId change usually covers them.
    // However, we should check the actual snapshot values to be safe or re-derive.
    // Simpler: Compare the new snapshot with lastSentState.

    const currentSnapshot = getMainStateSnapshot();
    const diff: Partial<MiniPlayerMainStateSnapshot> = {};
    let hasChanges = false;

    // We only check fields managed by PlayList store (everything except currentTime)
    // currentTime is handled by unsubscribePlayProgress
    for (const key in currentSnapshot) {
      const k = key as keyof MiniPlayerMainStateSnapshot;
      if (k === "currentTime") continue;

      if (currentSnapshot[k] !== lastSentState[k]) {
        diff[k] = currentSnapshot[k] as any;
        hasChanges = true;
      }
    }

    if (hasChanges) {
      postMainState(bc as BroadcastChannel, diff);
    }
  });

  unsubscribePlayProgress = usePlayProgress.subscribe(state => {
    const currentTime = state.currentTime;
    const lastTime = lastSentState.currentTime ?? -1;
    const isPlaying = usePlayList.getState().isPlaying;

    // Throttle:
    // 1. If not playing (scrubbing), sync immediately (or at higher rate).
    // 2. If playing, sync only if diff > 1s (Dead Reckoning correction).
    const shouldSync = !isPlaying || Math.abs(currentTime - lastTime) >= 1;

    if (shouldSync) {
      postMainState(bc as BroadcastChannel, { currentTime });
    }
  });
}

/**
 * 停止主窗口 -> mini 播放器的状态同步通道。
 */
export function stopMiniPlayerMainSync() {
  if (!isBroadcasting) return;

  unsubscribePlayList?.();
  unsubscribePlayList = null;
  unsubscribePlayProgress?.();
  unsubscribePlayProgress = null;
  bc?.close();
  bc = null;
  isBroadcasting = false;
}

/**
 * 切换mini/完整播放模式
 */
export async function toggleMiniMode() {
  try {
    const isMiniWindow = window.location.hash.includes("mini-player");

    if (isMiniWindow) {
      stopMiniPlayerMainSync();
    } else {
      startMiniPlayerMainSync();
    }

    await tauriAdapter.toggleMiniPlayer();
  } catch {
    addToast({
      title: "切换出错",
      color: "danger",
    });
  }
}
