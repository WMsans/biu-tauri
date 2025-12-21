import { t } from "i18next";
export enum SearchType {
  Video = "video",
  User = "bili_user",
}

export const SearchTypeOptions = [
  {
    label: t("pages.download-list.index..4"),
    value: SearchType.Video,
  },
  {
    label: t("pages.search.search-type."),
    value: SearchType.User,
  },
];
