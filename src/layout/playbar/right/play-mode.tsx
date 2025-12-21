import React from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@heroui/react";

import { getPlayModeList } from "@/common/constants/audio";
import { usePlayList } from "@/store/play-list";

import { PlayBarIconSize } from "../constants";

const PlayModeList = getPlayModeList(PlayBarIconSize.SideIconSize);

const PlayModeSwitch = () => {
  const { t } = useTranslation();
  const playMode = usePlayList(s => s.playMode);
  const togglePlayMode = usePlayList(s => s.togglePlayMode);

  return (
    <Button
      isIconOnly
      variant="light"
      size="sm"
      className="hover:text-primary min-w-fit text-[18px]"
      aria-label={t("layout.playbar.right.play-mode.")}
      onPress={togglePlayMode}
    >
      {PlayModeList.find(item => item.value === playMode)?.icon}
    </Button>
  );
};

export default PlayModeSwitch;
