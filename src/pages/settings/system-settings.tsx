import React from "react";
import { Controller } from "react-hook-form";
import type { Control, UseFormSetValue } from "react-hook-form";
import { useTranslation } from "react-i18next";

import { Button, Divider, Form, Input, Radio, RadioGroup, Select, SelectItem, Slider, Switch } from "@heroui/react";
import { RiArrowRightLongLine } from "@remixicon/react";

import type { LANGUAGE_VALUE_LIST } from "@shared/locales";

import ColorPicker from "@/components/color-picker";
import FontSelect from "@/components/font-select";
import UpdateCheckButton from "@/components/update-check-button";
import { LANGUAGES } from "@shared/locales";
import { defaultAppSettings } from "@shared/settings/app-settings";

import ImportExport from "./export-import";

type SystemSettingsTabProps = {
  appVersion: string;
  audioQuality: AudioQuality;
  control: Control<AppSettings>;
  isUpdateAvailable: boolean;
  latestVersion?: string;
  setValue: UseFormSetValue<AppSettings>;
};

export const SystemSettingsTab = ({
  appVersion,
  audioQuality,
  control,
  isUpdateAvailable,
  latestVersion,
  setValue,
}: SystemSettingsTabProps) => {
  const { t } = useTranslation();
  return (
    <Form className="space-y-6">
      <h2>{t("settings.system.appearance.title")}</h2>
      {/* Language */}
      <div className="flex w-full items-center justify-between">
        <div className="mr-6 space-y-1">
          <div className="text-medium font-medium">{t("settings.system.appearance.language.label")}</div>
          <div className="text-sm text-zinc-500">{t("settings.system.appearance.language.description")}</div>
        </div>
        <div className="w-[140px]">
          <Controller
            control={control}
            name={t("pages.settings.system-settings.language")}
            render={({ field }) => (
              <Select
                aria-label={t("settings.system.appearance.language.label")}
                selectedKeys={[field.value]}
                onSelectionChange={keys => {
                  const value = Array.from(keys)[0] as (typeof LANGUAGE_VALUE_LIST)[number];
                  if (value) {
                    field.onChange(value);
                  }
                }}
              >
                {LANGUAGES.map(lang => (
                  <SelectItem key={lang.value}>{lang.label}</SelectItem>
                ))}
              </Select>
            )}
          />
        </div>
      </div>
      {/* 显示模式 */}
      <div className="flex w-full items-center justify-between">
        <div className="mr-6 space-y-1">
          <div className="text-medium font-medium">{t("settings.system.appearance.displayMode.label")}</div>
          <div className="text-sm text-zinc-500">{t("settings.system.appearance.displayMode.description")}</div>
        </div>
        <Controller
          control={control}
          name={t("pages.settings.system-settings.displaymode")}
          render={({ field }) => (
            <RadioGroup orientation="horizontal" value={field.value} onValueChange={field.onChange}>
              <Radio value="card">{t("settings.system.appearance.displayMode.card")}</Radio>
              <Radio value="list">{t("settings.system.appearance.displayMode.list")}</Radio>
            </RadioGroup>
          )}
        />
      </div>
      {/* 字体选择 */}
      <div className="flex w-full items-center justify-between">
        <div className="mr-6 space-y-1">
          <div className="text-medium font-medium">{t("settings.system.appearance.font.label")}</div>
          <div className="text-sm text-zinc-500">{t("settings.system.appearance.font.description")}</div>
        </div>
        <div className="w-[360px]">
          <Controller
            control={control}
            name={t("pages.settings.system-settings.fontfamily")}
            render={({ field }) => <FontSelect value={field.value} onChange={field.onChange} />}
          />
        </div>
      </div>

      <div className="flex w-full items-center justify-between">
        <div className="mr-6 space-y-1">
          <div className="text-medium font-medium">{t("settings.system.appearance.contentBackgroundColor.label")}</div>
          <div className="text-sm text-zinc-500">
            {t("settings.system.appearance.contentBackgroundColor.description")}
          </div>
        </div>
        <div className="flex w-[360px] justify-end">
          <Controller
            control={control}
            name={t("pages.settings.system-settings.contentbackgroundcol")}
            render={({ field }) => (
              <ColorPicker
                presets={[defaultAppSettings.contentBackgroundColor]}
                value={field.value}
                onChange={field.onChange}
              />
            )}
          />
        </div>
      </div>

      <div className="flex w-full items-center justify-between">
        <div className="mr-6 space-y-1">
          <div className="text-medium font-medium">{t("settings.system.appearance.backgroundColor.label")}</div>
          <div className="text-sm text-zinc-500">{t("settings.system.appearance.backgroundColor.description")}</div>
        </div>
        <div className="flex w-[360px] justify-end">
          <Controller
            control={control}
            name={t("pages.settings.system-settings.backgroundcolor")}
            render={({ field }) => (
              <ColorPicker
                presets={[defaultAppSettings.backgroundColor]}
                value={field.value}
                onChange={field.onChange}
              />
            )}
          />
        </div>
      </div>

      <div className="flex w-full items-center justify-between">
        <div className="mr-6 space-y-1">
          <div className="text-medium font-medium">{t("settings.system.appearance.themeColor.label")}</div>
          <div className="text-sm text-zinc-500">{t("settings.system.appearance.themeColor.description")}</div>
        </div>
        <div className="flex w-[360px] justify-end">
          <Controller
            control={control}
            name={t("pages.settings.system-settings.primarycolor")}
            render={({ field }) => (
              <ColorPicker
                presets={[defaultAppSettings.primaryColor, "#66cc8a", "#9353d3", "#ffffff", "#db924b"]}
                value={field.value}
                onChange={field.onChange}
              />
            )}
          />
        </div>
      </div>

      {/* 全局圆角设置 */}
      <div className="flex w-full items-center justify-between">
        <div className="mr-6 space-y-1">
          <div className="text-medium font-medium">{t("settings.system.appearance.borderRadius.label")}</div>
          <div className="text-sm text-zinc-500">{t("settings.system.appearance.borderRadius.description")}</div>
        </div>
        <div className="w-[360px]">
          <Controller
            control={control}
            name={t("pages.settings.system-settings.borderradius")}
            render={({ field }) => (
              <Slider
                showTooltip={false}
                size="sm"
                endContent={
                  <span>
                    {field.value}
                    {t("pages.settings.system-settings.px")}
                  </span>
                }
                aria-label={t("settings.system.appearance.borderRadius.label")}
                value={field.value}
                onChange={v => field.onChange(Number(v))}
                minValue={0}
                maxValue={24}
                step={1}
                classNames={{
                  thumb: "after:hidden",
                }}
              />
            )}
          />
        </div>
      </div>
      <Divider />
      <h2>{t("settings.system.playback.title")}</h2>
      {/* 音质选择 */}
      <div className="flex w-full items-center justify-between">
        <div className="mr-6 space-y-1">
          <div className="text-medium font-medium">{t("settings.system.playback.audioQuality.label")}</div>
          <div className="text-sm text-zinc-500">
            {audioQuality === "auto" && t("settings.system.playback.audioQuality.autoDescription")}
            {audioQuality === "lossless" && "FLAC / Hi-Res"}
            {audioQuality === "high" && "180-320 kbps"}
            {audioQuality === "medium" && "100-140 kbps"}
            {audioQuality === "low" && "60-80 kbps"}
          </div>
        </div>
        <div className="w-[140px]">
          <Controller
            control={control}
            name={t("pages.settings.system-settings.audioquality")}
            render={({ field }) => (
              <Select
                aria-label={t("settings.system.playback.audioQuality.label")}
                selectedKeys={[field.value]}
                onSelectionChange={keys => {
                  const value = Array.from(keys)[0] as AudioQuality;
                  field.onChange(value);
                }}
              >
                <SelectItem key="auto">{t("settings.system.playback.audioQuality.auto")}</SelectItem>
                <SelectItem key="lossless">{t("settings.system.playback.audioQuality.lossless")}</SelectItem>
                <SelectItem key="high">{t("settings.system.playback.audioQuality.high")}</SelectItem>
                <SelectItem key="medium">{t("settings.system.playback.audioQuality.medium")}</SelectItem>
                <SelectItem key="low">{t("settings.system.playback.audioQuality.low")}</SelectItem>
              </Select>
            )}
          />
        </div>
      </div>
      <Divider />
      <h2>{t("settings.system.download.title")}</h2>
      {/* 下载目录配置 */}
      <div className="flex w-full items-center justify-between">
        <div className="mr-6 space-y-1">
          <div className="text-medium font-medium">{t("settings.system.download.path.label")}</div>
          <div className="text-sm text-zinc-500">{t("settings.system.download.path.description")}</div>
        </div>
        <div className="w-[360px]">
          <Controller
            control={control}
            name={t("pages.settings.system-settings.downloadpath")}
            render={({ field }) => (
              <div className="flex items-center space-x-1">
                <Input
                  isDisabled
                  placeholder={t("settings.system.download.path.placeholder")}
                  value={field.value}
                  onValueChange={field.onChange}
                />

                <Button
                  variant="flat"
                  onPress={async () => {
                    const path = await window.electron.selectDirectory();
                    if (path) setValue("downloadPath", path, { shouldDirty: true, shouldTouch: true });
                  }}
                >
                  {t("common.select")}
                </Button>
              </div>
            )}
          />
        </div>
      </div>

      {/* FFmpeg 路径配置 */}
      <div className="flex w-full items-center justify-between">
        <div className="mr-6 space-y-1">
          <div className="text-medium font-medium">{t("settings.system.download.ffmpegPath.label")}</div>
          <div className="text-sm text-zinc-500">{t("settings.system.download.ffmpegPath.description")}</div>
        </div>
        <div className="w-[360px]">
          <Controller
            control={control}
            name={t("pages.settings.system-settings.ffmpegpath")}
            render={({ field }) => (
              <div className="flex items-center space-x-1">
                <Input
                  isDisabled
                  placeholder={t("settings.system.download.ffmpegPath.placeholder")}
                  value={field.value}
                  onValueChange={field.onChange}
                />

                <Button
                  variant="flat"
                  onPress={async () => {
                    const path = await window.electron.selectFile();
                    if (path) setValue("ffmpegPath", path, { shouldDirty: true, shouldTouch: true });
                  }}
                >
                  {t("common.select")}
                </Button>
              </div>
            )}
          />
        </div>
      </div>
      <Divider />
      <h2>{t("settings.system.system.title")}</h2>
      {/* 窗口关闭选项 */}
      <div className="flex w-full items-center justify-between">
        <div className="mr-6 space-y-1">
          <div className="text-medium font-medium">{t("settings.system.system.closeWindowOption.label")}</div>
          <div className="text-sm text-zinc-500">{t("settings.system.system.closeWindowOption.description")}</div>
        </div>
        <Controller
          control={control}
          name={t("pages.settings.system-settings.closewindowoption")}
          render={({ field }) => (
            <RadioGroup orientation="horizontal" value={field.value} onValueChange={field.onChange}>
              <Radio value="hide">{t("settings.system.system.closeWindowOption.hide")}</Radio>
              <Radio value="exit">{t("settings.system.system.closeWindowOption.exit")}</Radio>
            </RadioGroup>
          )}
        />
      </div>

      {/* 开机自启动开关 */}
      <div className="flex w-full items-center justify-between">
        <div className="mr-6 space-y-1">
          <div className="text-medium font-medium">{t("settings.system.system.autoStart.label")}</div>
          <div className="text-sm text-zinc-500">{t("settings.system.system.autoStart.description")}</div>
        </div>
        <div className="flex w-[360px] justify-end">
          <Controller
            control={control}
            name={t("pages.settings.system-settings.autostart")}
            render={({ field }) => <Switch isSelected={field.value} onValueChange={field.onChange} />}
          />
        </div>
      </div>

      <Divider />
      <h2>{t("settings.system.about.title")}</h2>
      <div className="flex w-full items-center justify-between">
        <div className="mr-6 flex items-center space-x-1">
          <span>{t("settings.system.about.version", { version: appVersion })}</span>
          {isUpdateAvailable && Boolean(latestVersion) && (
            <>
              <RiArrowRightLongLine size={16} />
              <span className="text-primary">{latestVersion}</span>
            </>
          )}
        </div>
        <UpdateCheckButton />
      </div>
      <ImportExport />
    </Form>
  );
};
