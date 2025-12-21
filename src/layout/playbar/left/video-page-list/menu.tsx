import { useTranslation } from "react-i18next";

import { Button, Dropdown, DropdownItem, DropdownMenu, DropdownTrigger, useDisclosure } from "@heroui/react";
import { RiDeleteBinLine, RiExternalLinkLine, RiMoreFill } from "@remixicon/react";

import { openBiliVideoLink } from "@/common/utils/url";
import { usePlayList, type PlayData } from "@/store/play-list";

interface Props {
  data: PlayData;
}

const Menus = ({ data }: Props) => {
  const { t } = useTranslation();
  const delPage = usePlayList(state => state.delPage);
  const { isOpen, onOpenChange } = useDisclosure();

  return (
    <>
      <Dropdown
        disableAnimation
        isOpen={isOpen}
        onOpenChange={onOpenChange}
        classNames={{
          content: "min-w-fit",
        }}
      >
        <DropdownTrigger>
          <Button
            isIconOnly
            variant="light"
            size="sm"
            className={`flex-none transition-opacity duration-200 ${isOpen ? "pointer-events-auto opacity-100" : "pointer-events-none opacity-0"} group-hover:pointer-events-auto group-hover:opacity-100`}
          >
            <RiMoreFill size={16} />
          </Button>
        </DropdownTrigger>
        <DropdownMenu aria-label={t("layout.playbar.left.video-page-list.menu.")}>
          <DropdownItem
            key="bililink"
            startContent={<RiExternalLinkLine size={16} />}
            onPress={() => {
              openBiliVideoLink(data);
            }}
          >
            {t("layout.playbar.left.video-page-list.menu.b")}
          </DropdownItem>
          <DropdownItem
            key="del"
            color="danger"
            startContent={<RiDeleteBinLine size={16} />}
            onPress={() => delPage(data.id)}
          >
            {t("layout.playbar.left.video-page-list.menu..1")}
          </DropdownItem>
        </DropdownMenu>
      </Dropdown>
    </>
  );
};

export default Menus;
