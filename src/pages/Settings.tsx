import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";

interface AppConfig {
  switchSystem: boolean;
  switchApps: boolean;
  autostart: boolean;
  showTray: boolean;
  notifyOnSwitch: boolean;
  language?: string;
}

const defaultConfig: AppConfig = {
  switchSystem: true,
  switchApps: true,
  autostart: false,
  showTray: true,
  notifyOnSwitch: true,
  language: "en",
};

export default function Settings() {
  const { t, i18n } = useTranslation();
  const [config, setConfig] = useState<AppConfig>(defaultConfig);

  useEffect(() => {
    invoke<AppConfig & Record<string, unknown>>("get_config")
      .then((c) => {
        const lang = (c.language as string) ?? "en";
        setConfig({
          switchSystem: c.switchSystem ?? true,
          switchApps: c.switchApps ?? true,
          autostart: c.autostart ?? false,
          showTray: c.showTray ?? true,
          notifyOnSwitch: c.notifyOnSwitch ?? true,
          language: lang,
        });
        i18n.changeLanguage(lang);
      })
      .catch(() => setConfig(defaultConfig));
  }, [i18n]);

  const exitApp = () => void invoke("exit_app");

  /** Immediately persist a partial config change. */
  const saveWith = (patch: Partial<AppConfig>) => {
    const newConfig = { ...config, ...patch };
    setConfig(newConfig);
    invoke<AppConfig & Record<string, unknown>>("get_config")
      .then((full) => invoke("save_config", { config: { ...full, ...newConfig } }))
      .catch(console.error);
  };

  const itemAnim = { initial: { opacity: 0, y: 8 }, animate: { opacity: 1, y: 0 }, transition: { duration: 0.3, ease: [0.22, 0.61, 0.36, 1] } };
  const listVariants = { initial: {}, animate: { transition: { staggerChildren: 0.05, delayChildren: 0.05 } } };

  return (
    <motion.div initial="initial" animate="animate" variants={listVariants}>
      <motion.div className="panel" variants={{ initial: { opacity: 0, y: 16 }, animate: { opacity: 1, y: 0 } }} transition={{ duration: 0.4, ease: [0.22, 0.61, 0.36, 1] }}>
        <div className="panel-title">{t("settings.title")}</div>

        <motion.div className="toggle-wrap settings-row" variants={itemAnim}>
          <div>
            <span className="section-label">{t("settings.switchSystem")}</span>
            <p className="section-hint">{t("settings.switchSystemHint")}</p>
          </div>
          <motion.button
            type="button"
            role="switch"
            aria-checked={config.switchSystem}
            className={`toggle ${config.switchSystem ? "on" : ""}`}
            onClick={() => saveWith({ switchSystem: !config.switchSystem })}
            whileTap={{ scale: 0.95 }}
          />
        </motion.div>
        <motion.div className="toggle-wrap settings-row" variants={itemAnim}>
          <div>
            <span className="section-label">{t("settings.switchApps")}</span>
            <p className="section-hint">{t("settings.switchAppsHint")}</p>
          </div>
          <motion.button
            type="button"
            role="switch"
            aria-checked={config.switchApps}
            className={`toggle ${config.switchApps ? "on" : ""}`}
            onClick={() => saveWith({ switchApps: !config.switchApps })}
            whileTap={{ scale: 0.95 }}
          />
        </motion.div>
        <motion.div className="toggle-wrap settings-row" variants={itemAnim}>
          <span className="section-label">{t("settings.autostart")}</span>
          <motion.button
            type="button"
            role="switch"
            aria-checked={config.autostart}
            className={`toggle ${config.autostart ? "on" : ""}`}
            onClick={() => saveWith({ autostart: !config.autostart })}
            whileTap={{ scale: 0.95 }}
          />
        </motion.div>
        <motion.div className="toggle-wrap settings-row" variants={itemAnim}>
          <span className="section-label">{t("settings.showTray")}</span>
          <motion.button
            type="button"
            role="switch"
            aria-checked={config.showTray}
            className={`toggle ${config.showTray ? "on" : ""}`}
            onClick={() => saveWith({ showTray: !config.showTray })}
            whileTap={{ scale: 0.95 }}
          />
        </motion.div>
        <motion.div className="toggle-wrap settings-row" variants={itemAnim}>
          <span className="section-label">{t("settings.notifyOnSwitch")}</span>
          <motion.button
            type="button"
            role="switch"
            aria-checked={config.notifyOnSwitch}
            className={`toggle ${config.notifyOnSwitch ? "on" : ""}`}
            onClick={() => saveWith({ notifyOnSwitch: !config.notifyOnSwitch })}
            whileTap={{ scale: 0.95 }}
          />
        </motion.div>

        <motion.div className="toggle-wrap settings-row" variants={itemAnim}>
          <span className="section-label">{t("settings.language")}</span>
          <select
            className="lang-select"
            value={config.language ?? i18n.language}
            onChange={(e) => {
              const lang = e.target.value;
              i18n.changeLanguage(lang);
              saveWith({ language: lang });
              invoke("update_tray_labels", { lang }).catch(() => {});
            }}
          >
            <option value="en">English</option>
            <option value="zh">中文</option>
          </select>
        </motion.div>
      </motion.div>

      <motion.div className="settings-actions" initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.2 }}>
        <motion.button
          type="button"
          className="btn btn-exit"
          onClick={exitApp}
          whileHover={{ scale: 1.01 }}
          whileTap={{ scale: 0.98 }}
        >
          {t("common.exitApp")}
        </motion.button>
      </motion.div>
    </motion.div>
  );
}
