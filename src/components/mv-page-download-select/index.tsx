import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Modal, ModalContent, ModalHeader, ModalBody, ModalFooter, Checkbox, Spinner, addToast } from "@heroui/react";
import { useRequest } from "ahooks";

import { getPlayerPagelist } from "@/service/player-pagelist";

import AsyncButton from "../async-button";
import ScrollContainer from "../scroll-container";

interface Props {
  outputFileType: MediaDownloadOutputFileType;
  title: string;
  cover?: string;
  bvid: string;
  isOpen: boolean;
  onOpenChange: (isOpen: boolean) => void;
}

const MvPageDownloadSelect = ({ outputFileType, title, cover, bvid, isOpen, onOpenChange }: Props) => {
  const { t } = useTranslation();
  const [selectedCids, setSelectedCids] = useState<string[]>([]);

  const { data, loading } = useRequest(
    async () => {
      const res = await getPlayerPagelist({
        bvid,
      });

      if (res?.data?.length > 1) {
        setSelectedCids(res?.data?.map(item => String(item.cid)) || []);
      } else if (res?.data?.[0]?.cid) {
        const cid = String(res.data[0].cid);
        await window.electron.addMediaDownloadTask({
          outputFileType,
          cover,
          title,
          bvid,
          cid,
        });
        onOpenChange(false);
        addToast({
          title: t("components.mv-action.index..5"),
          color: "success",
        });
      }

      return res?.data || [];
    },
    {
      ready: isOpen,
      refreshDeps: [bvid],
    },
  );

  const downloadSelected = async () => {
    await window.electron.addMediaDownloadTaskList(
      data!
        .filter(item => selectedCids.includes(String(item.cid)))
        .map(item => ({
          outputFileType,
          title: item.part || title,
          bvid,
          cover: item.first_frame || cover,
          cid: String(item.cid),
        })),
    );
    onOpenChange(false);
    addToast({
      title: t("components.mv-action.index..5"),
      color: "success",
    });
  };

  return (
    <Modal scrollBehavior="inside" isOpen={isOpen} onOpenChange={onOpenChange}>
      <ModalContent>
        {Boolean(data?.length) && <ModalHeader>{t("components.mv-page-download-select.index.")}</ModalHeader>}
        <ModalBody className="p-0">
          {loading ? (
            <div className="flex h-60 items-center justify-center">
              <Spinner label={t("components.mv-page-download-select.index..1")} />
            </div>
          ) : (
            <ScrollContainer>
              <div className="flex flex-col space-y-1">
                {data?.map(item => {
                  const isSelected = selectedCids.includes(String(item.cid));

                  return (
                    <Checkbox
                      disableAnimation
                      key={item.cid}
                      aria-label={item.part}
                      isSelected={isSelected}
                      onValueChange={isSelected => {
                        if (isSelected) {
                          setSelectedCids([...selectedCids, String(item.cid)]);
                        } else {
                          setSelectedCids(selectedCids.filter(cid => cid !== String(item.cid)));
                        }
                      }}
                      className="hover:bg-content2 m-0 flex w-full max-w-full truncate px-6 py-4"
                      classNames={{
                        label: t("components.mv-page-download-select.index.truncate"),
                      }}
                    >
                      {item.part}
                    </Checkbox>
                  );
                })}
              </div>
            </ScrollContainer>
          )}
        </ModalBody>
        {Boolean(data?.length) && (
          <ModalFooter>
            <Checkbox
              aria-label={t("components.mv-page-download-select.index..2")}
              isSelected={selectedCids.length === data?.length}
              onValueChange={isSelected => {
                if (isSelected) {
                  setSelectedCids(data?.map(item => String(item.cid)) || []);
                } else {
                  setSelectedCids([]);
                }
              }}
              className="px-4"
            >
              {t("components.mv-page-download-select.index..2")}
            </Checkbox>
            <AsyncButton color="primary" isDisabled={!selectedCids.length} onPress={downloadSelected}>
              {t("settings.system.download.title")}
            </AsyncButton>
          </ModalFooter>
        )}
      </ModalContent>
    </Modal>
  );
};

export default MvPageDownloadSelect;
