import {
  RiMvLine,
  RiMvFill,
  RiGroupLine,
  RiGroupFill,
  RiDiscLine,
  RiDiscFill,
  RiUserFollowLine,
  RiUserFollowFill,
  RiFileDownloadLine,
  RiFileDownloadFill,
  RiTimeLine,
  RiTimeFill,
  RiHistoryLine,
  RiHistoryFill,
} from "@remixicon/react";
import { t } from "i18next";

import { type MenuItemProps } from "@/components/menu/menu-item";

export const DefaultMenuList: (MenuItemProps & { needLogin?: boolean })[] = [
  {
    title: t("pages.music-rank.index."),
    href: "/",
    icon: RiMvLine,
    activeIcon: RiMvFill,
  },
  {
    title: t("pages.artist-rank.index..1"),
    href: "/artist-rank",
    icon: RiGroupLine,
    activeIcon: RiGroupFill,
  },
  {
    title: t("common.constants.menus."),
    href: "/music-recommend",
    icon: RiDiscLine,
    activeIcon: RiDiscFill,
  },
  {
    title: t("pages.follow-list.index."),
    href: "/follow",
    needLogin: true,
    icon: RiUserFollowLine,
    activeIcon: RiUserFollowFill,
  },
  {
    title: t("pages.later.index."),
    href: "/later",
    needLogin: true,
    icon: RiTimeLine,
    activeIcon: RiTimeFill,
  },
  {
    title: t("pages.history.index..1"),
    href: "/history",
    needLogin: true,
    icon: RiHistoryLine,
    activeIcon: RiHistoryFill,
  },
  {
    title: t("pages.download-list.index."),
    href: "/download-list",
    icon: RiFileDownloadLine,
    activeIcon: RiFileDownloadFill,
  },
];
