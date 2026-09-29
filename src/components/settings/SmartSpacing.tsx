import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";

interface SmartSpacingProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const SmartSpacing: React.FC<SmartSpacingProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();
    const enabled = getSetting("smart_spacing") ?? true;

    return (
      <ToggleSwitch
        checked={enabled}
        onChange={(next) => updateSetting("smart_spacing", next)}
        isUpdating={isUpdating("smart_spacing")}
        label={t("settings.debug.smartSpacing.label")}
        description={t("settings.debug.smartSpacing.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
    );
  },
);
