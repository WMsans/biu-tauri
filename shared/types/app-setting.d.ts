type AudioQuality = "auto" | "lossless" | "high" | "medium" | "low";

interface AppSettings {
  language: "en" | "zh-CN" | "zh-TW";
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
