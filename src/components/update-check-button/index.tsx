import { useTranslation } from "react-i18next";

import { addToast, useDisclosure } from "@heroui/react";

import { useAppUpdateStore } from "@/store/app-update";

import AsyncButton from "../async-button";
import ReleaseNoteModal from "../release-note-modal";

const UpdateCheckButton = () => {
  const { t } = useTranslation();
  const isUpdateAvailable = useAppUpdateStore(s => s.isUpdateAvailable);

  const {
    isOpen: isReleaseNoteModalOpen,
    onOpen: onReleaseNoteModalOpen,
    onOpenChange: onReleaseNoteModalOpenChange,
  } = useDisclosure();

  const checkUpdate = async () => {
    if (isUpdateAvailable) {
      onReleaseNoteModalOpen();

      return;
    }

    const res = await window.electron.checkAppUpdate();

    if (res?.error) {
      addToast({
        title: t("components.update-check-button.index."),
        description: res.error,
        color: "danger",
      });
    } else if (res?.isUpdateAvailable) {
      onReleaseNoteModalOpen();
    } else {
      addToast({
        title: t("components.update-check-button.index..1"),
      });
    }
  };

  return (
    <>
      <AsyncButton onPress={checkUpdate}>{isUpdateAvailable ? "查看更新内容" : "检查更新"}</AsyncButton>
      <ReleaseNoteModal isOpen={isReleaseNoteModalOpen} onOpenChange={onReleaseNoteModalOpenChange} />
    </>
  );
};

export default UpdateCheckButton;
