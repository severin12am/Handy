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
  const [listError, setListError] = useState("");

  const refreshWindows = async () => {
    try {
      const result = await commands.listDictationWindows();
      setWindows(result);
      setListError("");
    } catch {
      setListError(t("settings.debug.targetWindow.listFailed"));
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
          {target?.enabled
            ? t("settings.debug.targetWindow.pastingInto", {
                name: labelFor({
                  process_name: target.process_name,
                  title: target.title_substring,
                }),
              })
            : t("settings.debug.targetWindow.empty")}
        </div>
        <div className="text-xs text-text/70">
          {t("settings.debug.targetWindow.windowCount", {
            count: windows.length,
          })}
        </div>
        {listError ? (
          <p className="text-xs text-red-500">{listError}</p>
        ) : null}
        <div className="max-h-48 w-full overflow-y-auto rounded border border-mid-gray/40">
          {windows.map((window, index) => {
            const selected =
              target?.process_name === window.process_name &&
              target?.title_substring === window.title;
            return (
              <button
                key={`${window.process_name}-${window.title}-${index}`}
                type="button"
                className={`block w-full truncate px-2 py-1 text-left text-sm hover:bg-logo-primary/10 ${
                  selected ? "bg-logo-primary/20" : ""
                }`}
                onClick={() => {
                  void bind(window);
                }}
              >
                {labelFor(window)}
              </button>
            );
          })}
        </div>
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
