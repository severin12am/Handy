import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";

interface VoiceCommandsProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const VoiceCommands: React.FC<VoiceCommandsProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();
    const enabled = getSetting("voice_commands_enabled") ?? false;

    return (
      <ToggleSwitch
        checked={enabled}
        onChange={(next) => updateSetting("voice_commands_enabled", next)}
        isUpdating={isUpdating("voice_commands_enabled")}
        label={t("settings.debug.voiceCommands.label")}
        description={t("settings.debug.voiceCommands.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
    );
  },
);
