import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";

interface AppConfig {
  enabled: boolean;
  mode: "Fixed" | "Location";
  sunriseTime: string;
  sunsetTime: string;
  location: { enabled: boolean; lat: number; lon: number };
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

  useEffect(() => {
    invoke<AppConfig>("get_config")
      .then((c) => {
        setConfig({
          ...defaultConfig,
          ...c,
          mode: c.mode === "Location" ? "Location" : "Fixed",
        });
      })
      .catch(() => setConfig(defaultConfig));
  }, []);

  const save = async () => {
    try {
      const full = await invoke<AppConfig>("get_config").catch(() => defaultConfig);
      const merged: AppConfig = { ...full, ...config };
      await invoke("save_config", { config: merged });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error(e);
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
              <input
                type="text"
                value={config.sunriseTime}
                onChange={(e) => update({ sunriseTime: e.target.value })}
                placeholder="07:00"
              />
            </label>
            <label>
              {t("schedule.sunset")}
              <input
                type="text"
                value={config.sunsetTime}
                onChange={(e) => update({ sunsetTime: e.target.value })}
                placeholder="19:00"
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
        <motion.button
          className="btn btn-primary"
          onClick={save}
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
        >
          {saved ? "✓ " : ""}{t("common.save")}
        </motion.button>
      </motion.div>
    </motion.div>
  );
}
