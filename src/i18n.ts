import { initReactI18next } from "react-i18next";

import i18n from "i18next";

import translationEN from "./locales/en/translation.json";
import translationZHCN from "./locales/zh-CN/translation.json";
import translationZHTW from "./locales/zh-TW/translation.json";

const resources = {
  en: {
    translation: translationEN,
  },
  "zh-CN": {
    translation: translationZHCN,
  },
  "zh-TW": {
    translation: translationZHTW,
  },
};

i18n.use(initReactI18next).init({
  resources,
  fallbackLng: "en",
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
