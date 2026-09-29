import { listen } from "@tauri-apps/api/event";
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { Button } from "../ui/Button";
import { SettingContainer } from "../ui/SettingContainer";

interface MicLevelMeterProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const MicLevelMeter: React.FC<MicLevelMeterProps> = ({
  descriptionMode = "inline",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const [level, setLevel] = useState(0);
  const [testing, setTesting] = useState(false);
  const [message, setMessage] = useState("");
  const [silence, setSilence] = useState(false);

  useEffect(() => {
    const unlisten = listen<number>("mic-test-level", (event) => {
      const peak = Number(event.payload) || 0;
      setLevel(Math.max(0, Math.min(1, peak)));
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const percent = Math.min(100, Math.round(level * 350));

  return (
    <SettingContainer
      title={t("settings.sound.micTest.title")}
      description={t("settings.sound.micTest.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
    >
      <div className="flex flex-col gap-2 w-full min-w-56">
        <div className="h-2 w-full rounded bg-mid-gray/30 overflow-hidden">
          <div
            className={`h-full ${silence ? "bg-red-500" : "bg-logo-primary"}`}
            style={{ width: `${percent}%` }}
          />
        </div>
        <div className="flex items-center gap-2">
          <Button
            type="button"
            variant="secondary"
            size="sm"
            disabled={testing}
            onClick={() => {
              setTesting(true);
              setMessage("");
              setSilence(false);
              setLevel(0);
              void commands
                .testMicrophone()
                .then((result) => {
                  if (result.status !== "ok") {
                    setMessage(t("settings.sound.micTest.failed"));
                    setSilence(true);
                    return;
                  }
                  setLevel(result.data.peak);
                  setSilence(result.data.digital_silence);
                  setMessage(
                    result.data.digital_silence
                      ? t("settings.sound.micTest.silence")
                      : t("settings.sound.micTest.signal", {
                          peak: result.data.peak.toFixed(3),
                          format: `${result.data.sample_format} ${result.data.sample_rate} Hz`,
                        }),
                  );
                })
                .finally(() => setTesting(false));
            }}
          >
            {testing
              ? t("settings.sound.micTest.testing")
              : t("settings.sound.micTest.button")}
          </Button>
        </div>
        {message ? (
          <p className={`text-xs ${silence ? "text-red-500" : "text-text/80"}`}>
            {message}
          </p>
        ) : null}
      </div>
    </SettingContainer>
  );
};
