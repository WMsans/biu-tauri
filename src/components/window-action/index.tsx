import { useEffect, useState } from "react";

import {
  RiCloseLine,
  RiFullscreenExitLine,
  RiFullscreenLine,
  RiPictureInPicture2Line,
  RiSubtractLine,
} from "@remixicon/react";

import { toggleMiniMode } from "@/common/utils/mini-player";
import IconButton from "@/components/icon-button";
import { tauriAdapter } from "@/utils/tauri-adapter";

const WindowAction = () => {
  const [isMaximized, setIsMaximized] = useState(false);
  const [isFullScreen, setIsFullScreen] = useState(false);

  useEffect(() => {
    tauriAdapter.isMaximized().then(setIsMaximized);
    tauriAdapter.isFullScreen().then(setIsFullScreen);
    const unlistenMaximize = tauriAdapter.onWindowMaximizeChange(setIsMaximized);
    const unlistenFullScreen = tauriAdapter.onWindowFullScreenChange(setIsFullScreen);

    return () => {
      unlistenMaximize();
      unlistenFullScreen();
    };
  }, []);

  const handleMinimize = () => {
    tauriAdapter.minimizeWindow();
  };

  const handleMaximize = () => {
    tauriAdapter.toggleMaximizeWindow();
  };

  const handleClose = () => {
    tauriAdapter.closeWindow();
  };

  return (
    <div className="window-no-drag flex items-center justify-center">
      {!isFullScreen && (
        <>
          <IconButton title="切换到迷你播放器" onPress={toggleMiniMode}>
            <RiPictureInPicture2Line size={16} />
          </IconButton>
          <IconButton onPress={handleMinimize}>
            <RiSubtractLine size={18} />
          </IconButton>
          <IconButton onPress={handleMaximize}>
            {isMaximized ? <RiFullscreenExitLine size={14} /> : <RiFullscreenLine size={14} />}
          </IconButton>
          <IconButton onPress={handleClose}>
            <RiCloseLine size={18} />
          </IconButton>
        </>
      )}
    </div>
  );
};

export default WindowAction;
