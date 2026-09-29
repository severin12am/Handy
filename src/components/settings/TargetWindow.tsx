import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands, type ListedWindow } from "@/bindings";
import { useOsType } from "../../hooks/useOsType";
import { useSettings } from "../../hooks/useSettings";
import { Button } from "../ui/Button";
import { SettingContainer } from "../ui/SettingContainer";
import { ToggleSwitch } from "../ui/ToggleSwitch";

interface TargetWindowProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const TargetWindowSetting: React.FC<TargetWindowProps> = ({
  descriptionMode = "inline",
  grouped = false,
}) => {
  const osType = useOsType();
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();
  const target = getSetting("dictation_target");
  const [windows, setWindows] = useState<ListedWindow[]>([]);
  const [busy, setBusy] = useState(false);

  const refreshWindows = async () => {
    const result = await commands.listDictationWindows();
    if (result.status === "ok") {
      setWindows(result.data);
    }
  };

  useEffect(() => {
    if (osType === "windows") {
      void refreshWindows();
    }
  }, [osType]);

  if (osType !== "windows") {
    return null;
  }

  const labelFor = (window: ListedWindow) =>
    `${window.process_name} — ${window.title}`;

  const bind = async (window: ListedWindow) => {
    await commands.changeDictationTargetSetting(
      window.process_name,
      window.title,
      target?.auto_enter ?? false,
      true,
    );
    await refreshSettings();
  };

  return (
    <SettingContainer
      title={t("settings.debug.targetWindow.title")}
      description={t("settings.debug.targetWindow.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
    >
      <div className="flex flex-col gap-2 w-full">
        <div className="text-sm text-text">
          {target
            ? `${target.process_name || target.title_substring}${
                target.enabled ? "" : " (off)"
              }`
            : t("settings.debug.targetWindow.empty")}
        </div>
        <select
          className="bg-background border border-mid-gray/40 rounded px-2 py-1 text-sm"
          value=""
          onChange={(event) => {
            const index = Number(event.target.value);
            const window = windows[index];
            if (window) {
              void bind(window);
            }
          }}
        >
          <option value="">{t("settings.debug.targetWindow.refresh")}</option>
          {windows.map((window, index) => (
            <option key={`${window.process_name}-${index}`} value={index}>
              {labelFor(window)}
            </option>
          ))}
        </select>
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            variant="secondary"
            size="sm"
            disabled={busy}
            onClick={() => {
              setBusy(true);
              void commands.armPickDictationWindow().finally(async () => {
                await refreshSettings();
                setBusy(false);
              });
            }}
          >
            {t("settings.debug.targetWindow.pick")}
          </Button>
          <Button
            type="button"
            variant="secondary"
            size="sm"
            onClick={() => {
              void refreshWindows();
            }}
          >
            {t("settings.debug.targetWindow.refresh")}
          </Button>
          <Button
            type="button"
            variant="secondary"
            size="sm"
            onClick={() => {
              void commands.clearDictationTargetSetting().then(() =>
                refreshSettings(),
              );
            }}
          >
            {t("settings.debug.targetWindow.clear")}
          </Button>
        </div>
        <ToggleSwitch
          checked={target?.enabled ?? false}
          onChange={(enabled) => {
            if (!target) return;
            void commands
              .changeDictationTargetSetting(
                target.process_name,
                target.title_substring,
                target.auto_enter ?? false,
                enabled,
              )
              .then(() => refreshSettings());
          }}
          label={t("settings.debug.targetWindow.enabled")}
          description={t("settings.debug.targetWindow.description")}
          descriptionMode="tooltip"
          grouped={false}
        />
        <ToggleSwitch
          checked={target?.auto_enter ?? false}
          onChange={(autoEnter) => {
            if (!target) return;
            void commands
              .changeDictationTargetSetting(
                target.process_name,
                target.title_substring,
                autoEnter,
                target.enabled ?? true,
              )
              .then(() => refreshSettings());
          }}
          label={t("settings.debug.targetWindow.autoEnter")}
          description={t("settings.debug.targetWindow.autoEnter")}
          descriptionMode="tooltip"
          grouped={false}
        />
      </div>
    </SettingContainer>
  );
};
