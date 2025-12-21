import { initReactI18next } from "react-i18next";

import i18n from "i18next";

import { LANGUAGES } from "@shared/locales";
import en from "@shared/locales/en/translation.json";
import zhCN from "@shared/locales/zh-CN/translation.json";
import zhTW from "@shared/locales/zh-TW/translation.json";

// Add new translations here
const translations: Record<string, unknown> = {
  en,
  "zh-CN": zhCN,
  "zh-TW": zhTW,
};

const resources = LANGUAGES.reduce(
  (acc, { value }) => {
    if (translations[value]) {
      acc[value] = {
        translation: translations[value],
      };
    }
    return acc;
  },
  {} as Record<string, { translation: unknown }>,
);

i18n.use(initReactI18next).init({
  resources,
  lng: "zh-CN",
  fallbackLng: "zh-CN",
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
