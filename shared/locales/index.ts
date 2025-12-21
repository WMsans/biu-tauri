// TODO: generate this file from locales folder
export const LANGUAGES = [
  {
    value: "en",
    label: "English",
  },
  {
    value: "zh-CN",
    label: "简体中文",
  },
  {
    value: "zh-TW",
    label: "繁體中文",
  },
];

export const LANGUAGE_MAP = LANGUAGES.reduce(
  (acc, cur) => {
    acc[cur.value] = cur.label;
    return acc;
  },
  {} as Record<string, string>,
);

export const LANGUAGE_VALUE_LIST = LANGUAGES.map(item => item.value);
