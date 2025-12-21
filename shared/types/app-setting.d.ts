import type { LANGUAGE_VALUE_LIST } from "@shared/locales";

type AudioQuality = "auto" | "lossless" | "high" | "medium" | "low";

interface AppSettings {
  language: (typeof LANGUAGE_VALUE_LIST)[number];
  fontFamily: string;
  backgroundColor: string;
  contentBackgroundColor: string;
  primaryColor: string;
  borderRadius: number;
  downloadPath?: string;
  closeWindowOption: "hide" | "exit";
  autoStart: boolean;
  audioQuality: AudioQuality;
  hiddenMenuKeys: string[];
  displayMode: "card" | "list";
  ffmpegPath?: string;
}
