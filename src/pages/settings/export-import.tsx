import React, { useRef } from "react";
import { useTranslation } from "react-i18next";

import { Button, addToast } from "@heroui/react";
import { RiExportFill, RiImportFill } from "@remixicon/react";
import { merge } from "es-toolkit/object";

import { useSettings } from "@/store/settings";
import { defaultAppSettings } from "@shared/settings/app-settings";
import { StoreNameMap } from "@shared/store";

const ImportExport = () => {
  const { t } = useTranslation();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const updateSettings = useSettings(s => s.update);
  const getSettings = useSettings(s => s.getSettings);

  const handleExport = async () => {
    try {
      const settingStore = await window.electron.getStore(StoreNameMap.AppSettings);
      const blob = new Blob([JSON.stringify(settingStore?.appSettings ?? defaultAppSettings, null, 2)], {
        type: "application/json",
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `biu-settings-${new Date().toISOString().slice(0, 19).replace(/[:T]/g, "-")}.json`;
      document.body.appendChild(a);
      a.click();
      a.remove();
      URL.revokeObjectURL(url);
      addToast({ title: t("pages.settings.export-import..2"), description: "JSON 文件已下载", color: "success" });
    } catch (e) {
      addToast({ title: t("pages.settings.export-import..3"), description: String(e), color: "danger" });
    }
  };

  const handleImportClick = () => {
    fileInputRef.current?.click();
  };

  const handleImportFileChange: React.ChangeEventHandler<HTMLInputElement> = async e => {
    const file = e.target.files?.[0];
    e.target.value = "";
    if (!file) return;
    try {
      const text = await file.text();
      const data = JSON.parse(text) as Record<string, unknown>;

      const patch: Record<string, unknown> = {};
      for (const key of Object.keys(defaultAppSettings)) {
        patch[key] = data[key];
      }

      const merged = merge(getSettings(), patch);

      updateSettings(merged);
      addToast({ title: t("pages.settings.export-import..4"), description: "设置已应用", color: "success" });
      window.location.reload();
    } catch {
      addToast({
        title: t("pages.settings.export-import..5"),
        description: "文件解析错误或格式不正确",
        color: "danger",
      });
    }
  };

  return (
    <div className="flex items-center space-x-2">
      <input ref={fileInputRef} type="file" accept="application/json" hidden onChange={handleImportFileChange} />
      <Button size="sm" radius="md" startContent={<RiExportFill size={16} />} onPress={handleExport}>
        {t("pages.settings.export-import.")}
      </Button>
      <Button size="sm" radius="md" startContent={<RiImportFill size={16} />} onPress={handleImportClick}>
        {t("pages.settings.export-import..1")}
      </Button>
    </div>
  );
};

export default ImportExport;
