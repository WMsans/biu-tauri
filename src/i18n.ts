import { initReactI18next } from "react-i18next";

import i18n from "i18next";

const resources = Object.entries(import.meta.glob("@shared/locales/*/translation.json", { eager: true })).reduce(
  (acc, [path, translation]) => {
    const lang = path.split("/")[2];
    acc[lang] = {
      translation,
    };
    return acc;
  },
  {} as Record<string, { translation: any }>,
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
