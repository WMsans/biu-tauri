import { useTranslation } from "react-i18next";

import { Dropdown, DropdownTrigger, Button, DropdownMenu, DropdownItem, Switch } from "@heroui/react";
import { RiSettings3Line } from "@remixicon/react";

import { usePlayList } from "@/store/play-list";

const Settings = () => {
  const { t } = useTranslation();
  const shouldKeepPagesOrderInRandomPlayMode = usePlayList(s => s.shouldKeepPagesOrderInRandomPlayMode);
  const setShouldKeepPagesOrderInRandomPlayMode = usePlayList(s => s.setShouldKeepPagesOrderInRandomPlayMode);

  return (
    <Dropdown>
      <DropdownTrigger>
        <Button isIconOnly size="sm" variant="light">
          <RiSettings3Line size={16} />
        </Button>
      </DropdownTrigger>
      <DropdownMenu aria-label={t("layout.playbar.right.play-list-drawer.settings.")}>
        <DropdownItem key="shouldKeepPagesOrderInRandomPlayMode">
          <Switch
            size="sm"
            isSelected={shouldKeepPagesOrderInRandomPlayMode}
            onValueChange={setShouldKeepPagesOrderInRandomPlayMode}
          >
            {t("layout.playbar.right.play-list-drawer.settings..1")}
          </Switch>
        </DropdownItem>
      </DropdownMenu>
    </Dropdown>
  );
};

export default Settings;
