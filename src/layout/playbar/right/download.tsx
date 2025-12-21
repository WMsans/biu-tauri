import { useTranslation } from "react-i18next";

import { addToast, Button, Dropdown, DropdownItem, DropdownMenu, DropdownTrigger } from "@heroui/react";
import { RiDownload2Fill } from "@remixicon/react";

import { ReactComponent as AudioDownloadIcon } from "@/assets/icons/audio-download.svg";
import { ReactComponent as VideoDownloadIcon } from "@/assets/icons/video-download.svg";
import AsyncButton from "@/components/async-button";
import { usePlayList } from "@/store/play-list";

import { PlayBarIconSize } from "../constants";

const Download = () => {
  const { t } = useTranslation();
  const list = usePlayList(s => s.list);
  const playId = usePlayList(s => s.playId);
  const playItem = list.find(item => item.id === playId);

  const downloadAudio = async () => {
    await window.electron.addMediaDownloadTask({
      outputFileType: "audio",
      title: playItem?.pageTitle || playItem?.title || `audio-${Date.now()}`,
      cover: playItem?.pageCover || playItem?.cover,
      bvid: playItem?.bvid,
      cid: playItem?.cid,
      sid: playItem?.sid,
    });

    addToast({
      title: t("layout.playbar.right.download..3"),
      color: "success",
    });
  };

  const downloadVideo = async () => {
    await window.electron.addMediaDownloadTask({
      outputFileType: "video",
      title: playItem?.pageTitle || playItem?.title || `video-${Date.now()}`,
      cover: playItem?.pageCover || playItem?.cover,
      bvid: playItem?.bvid,
      cid: playItem?.cid,
    });

    addToast({
      title: t("layout.playbar.right.download..3"),
      color: "success",
    });
  };

  if (playItem?.sid) {
    return (
      <AsyncButton isIconOnly size="sm" variant="light" className="hover:text-primary" onPress={downloadAudio}>
        <RiDownload2Fill size={PlayBarIconSize.SideIconSize} />
      </AsyncButton>
    );
  }

  return (
    <Dropdown
      classNames={{
        content: "min-w-fit",
      }}
    >
      <DropdownTrigger>
        <Button isIconOnly size="sm" variant="light" className="hover:text-primary">
          <RiDownload2Fill size={PlayBarIconSize.SideIconSize} />
        </Button>
      </DropdownTrigger>
      <DropdownMenu aria-label={t("layout.playbar.right.download.")}>
        <DropdownItem
          key="downloadAudio"
          startContent={<AudioDownloadIcon className="relative top-px left-px h-[15px] w-[15px]" />}
          onPress={downloadAudio}
        >
          {t("layout.playbar.right.download..1")}
        </DropdownItem>
        <DropdownItem
          key="downloadVideo"
          startContent={<VideoDownloadIcon className="relative top-px left-px h-[15px] w-[15px]" />}
          onPress={downloadVideo}
        >
          {t("layout.playbar.right.download..2")}
        </DropdownItem>
      </DropdownMenu>
    </Dropdown>
  );
};

export default Download;
