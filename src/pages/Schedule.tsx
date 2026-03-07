import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import TimePicker from "../components/TimePicker";

/** Must match backend AppConfig (camelCase from serde). */
interface AppConfig {
  enabled: boolean;
  mode: "Fixed" | "Location";
  sunriseTime: string;
  sunsetTime: string;
  location: { enabled: boolean; lat: number; lon: number };
  switchSystem?: boolean;
  switchApps?: boolean;
  autostart?: boolean;
  showTray?: boolean;
  notifyOnSwitch?: boolean;
  language?: string;
  manualOverrideUntil?: number | null;
}

const defaultConfig: AppConfig = {
  enabled: true,
  mode: "Fixed",
  sunriseTime: "07:00",
  sunsetTime: "19:00",
  location: { enabled: false, lat: 39.9, lon: 116.4 },
};

export default function Schedule() {
  const { t } = useTranslation();
  const [config, setConfig] = useState<AppConfig>(defaultConfig);
  const [saved, setSaved] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  const loadConfig = () => {
    setSaveError(null);
    invoke<AppConfig>("get_config")
      .then((c) => {
        const rawMode = (c as { mode?: string }).mode;
        const mode = rawMode === "location" || rawMode === "Location" ? "Location" : "Fixed";
        setConfig({
          ...defaultConfig,
          ...c,
          mode,
        });
      })
      .catch(() => setConfig(defaultConfig));
  };

  useEffect(() => {
    loadConfig();
  }, []);

  const save = async () => {
    setSaveError(null);
    setSaving(true);
    try {
      const full = await invoke<AppConfig>("get_config").catch(() => defaultConfig);
      const modeForBackend = config.mode === "Location" ? "location" : "fixed";
      const merged = {
        ...defaultConfig,
        ...full,
        ...config,
        mode: modeForBackend,
        sunriseTime: config.sunriseTime ?? defaultConfig.sunriseTime,
        sunsetTime: config.sunsetTime ?? defaultConfig.sunsetTime,
        location: config.location ?? defaultConfig.location,
      };
      await invoke("save_config", { config: merged });
      setSaved(true);
      setTimeout(() => setSaved(false), 2500);
      loadConfig();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setSaveError(msg);
      console.error("Schedule save failed:", e);
    } finally {
      setSaving(false);
    }
  };

  const update = (patch: Partial<AppConfig>) => {
    setConfig((c) => ({ ...c, ...patch }));
  };

  const updateLocation = (patch: Partial<{ enabled: boolean; lat: number; lon: number }>) => {
    setConfig((c) => ({
      ...c,
      location: { ...c.location, ...patch },
    }));
  };

  const panelAnim = { initial: { opacity: 0, y: 8 }, animate: { opacity: 1, y: 0 }, transition: { duration: 0.3, ease: [0.22, 0.61, 0.36, 1] } };
  const containerVariants = { initial: {}, animate: { transition: { staggerChildren: 0.05, delayChildren: 0 } } };

  return (
    <motion.div initial="initial" animate="animate" variants={containerVariants}>
      <motion.div className="panel" variants={panelAnim}>
        <div className="panel-title">{t("schedule.mode")}</div>
        <div style={{ display: "flex", justifyContent: "center" }}>
          <div className="theme-btn-group" style={{ maxWidth: 400 }}>
            <motion.button
              type="button"
              className={`theme-btn${config.mode === "Fixed" ? " active" : ""}`}
              onClick={() => update({ mode: "Fixed" })}
              whileTap={{ scale: 0.97 }}
            >
              {t("schedule.fixed")}
            </motion.button>
            <motion.button
              type="button"
              className={`theme-btn${config.mode === "Location" ? " active" : ""}`}
              onClick={() => update({ mode: "Location" })}
              whileTap={{ scale: 0.97 }}
            >
              {t("schedule.location")}
            </motion.button>
          </div>
        </div>
      </motion.div>

      {config.mode === "Fixed" && (
        <motion.div className="panel" variants={panelAnim} initial="initial" animate="animate" transition={{ duration: 0.2 }}>
          <div className="panel-title">{t("schedule.sunrise")} / {t("schedule.sunset")}</div>
          <div className="form-row">
            <label>
              {t("schedule.sunrise")}
              <TimePicker
                value={config.sunriseTime}
                onChange={(v) => update({ sunriseTime: v })}
                aria-label={t("schedule.sunrise")}
              />
            </label>
            <label>
              {t("schedule.sunset")}
              <TimePicker
                value={config.sunsetTime}
                onChange={(v) => update({ sunsetTime: v })}
                aria-label={t("schedule.sunset")}
              />
            </label>
          </div>
        </motion.div>
      )}

      {config.mode === "Location" && (
        <motion.div className="panel" variants={panelAnim} initial="initial" animate="animate" transition={{ duration: 0.2 }}>
          <div className="panel-title">{t("schedule.useLocation")}</div>
          <p className="section-hint" style={{ marginBottom: "var(--space-3)" }}>
            {t("schedule.latLonHint")}
          </p>
          <div className="toggle-wrap" style={{ marginBottom: "var(--space-3)" }}>
            <span className="section-label">{t("schedule.useLocation")}</span>
            <button
              type="button"
              role="switch"
              aria-checked={config.location.enabled}
              className={`toggle ${config.location.enabled ? "on" : ""}`}
              onClick={() => updateLocation({ enabled: !config.location.enabled })}
            />
          </div>
          <div className="form-row">
            <label>
              {t("schedule.latitude")}
              <input
                type="number"
                step="any"
                value={config.location.lat}
                onChange={(e) => updateLocation({ lat: parseFloat(e.target.value) || 0 })}
              />
            </label>
            <label>
              {t("schedule.longitude")}
              <input
                type="number"
                step="any"
                value={config.location.lon}
                onChange={(e) => updateLocation({ lon: parseFloat(e.target.value) || 0 })}
              />
            </label>
          </div>
        </motion.div>
      )}

      <motion.div style={{ marginTop: "var(--space-4)" }} initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.15 }}>
        {saveError && (
          <p className="schedule-save-error" role="alert">
            {saveError}
          </p>
        )}
        <motion.button
          type="button"
          className="btn btn-primary btn-full"
          onClick={save}
          disabled={saving}
          whileHover={saving ? {} : { scale: 1.02 }}
          whileTap={saving ? {} : { scale: 0.98 }}
        >
          {saving ? t("schedule.saving") : saved ? "✓ " + t("schedule.saved") : t("common.save")}
        </motion.button>
      </motion.div>
    </motion.div>
  );
}
