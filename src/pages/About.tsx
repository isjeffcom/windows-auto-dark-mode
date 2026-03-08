import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";

function GitHubIcon() {
  return (
    <svg className="about-github-icon" viewBox="0 0 24 24" fill="currentColor" aria-hidden>
      <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
    </svg>
  );
}

export default function About() {
  const { t } = useTranslation();
  const [version, setVersion] = useState("–");
  const [updateStatus, setUpdateStatus] = useState<"idle" | "checking" | "has_update" | "current" | "error">("idle");
  const [latestVersion, setLatestVersion] = useState<string | null>(null);

  useEffect(() => {
    invoke<string>("get_version").then(setVersion).catch(() => setVersion("–"));
  }, []);

  const checkUpdate = useCallback(async () => {
    setUpdateStatus("checking");
    try {
      const result = await invoke<{ has_update: boolean; latest_version: string | null }>("check_for_update");
      if (result.has_update && result.latest_version) {
        setLatestVersion(result.latest_version);
        setUpdateStatus("has_update");
      } else {
        setUpdateStatus("current");
      }
    } catch {
      setUpdateStatus("error");
    }
  }, []);

  // Auto-check for updates once when About is opened.
  useEffect(() => {
    checkUpdate();
  }, [checkUpdate]);

  const openReleasePage = () => {
    invoke("open_release_page").catch(() => {});
  };

  const openInBrowser = (url: string) => {
    invoke("open_url", { url }).catch(() => {});
  };

  return (
    <motion.div
      className="panel about-panel"
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4, ease: [0.22, 0.61, 0.36, 1] }}
    >
      <div className="panel-title">{t("about.title")}</div>
      <motion.p
        className="section-hint about-desc"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.1 }}
      >
        {t("about.description")}
      </motion.p>
      <motion.div
        className="about-links"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.1 }}
      >
        <button
          type="button"
          className="about-link about-link-icon"
          onClick={() => openInBrowser("https://github.com/isjeffcom/windows-auto-dark-mode")}
          title="GitHub"
          aria-label="GitHub"
        >
          <GitHubIcon />
        </button>
        <button
          type="button"
          className="about-link"
          onClick={() => openInBrowser("https://soda-game.com")}
        >
          {t("about.sodaGame")}
        </button>
      </motion.div>

      <motion.div
        className="about-footer"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.15 }}
      >
        <p className="section-hint about-version">
          {t("about.version")} {version}
        </p>
        <div className="about-update-section">
        <div className="about-update-row">
          <motion.button
            type="button"
            className="btn"
            onClick={checkUpdate}
            disabled={updateStatus === "checking"}
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
          >
            {updateStatus === "checking" ? t("about.checking") : t("about.checkForUpdates")}
          </motion.button>
          {(updateStatus === "current" || updateStatus === "error") && (
            <p className="section-hint about-update-status">
              {updateStatus === "current" && t("about.youAreUpToDate")}
              {updateStatus === "error" && (
                <span style={{ color: "var(--text-muted)" }}>{t("about.checkFailed")}</span>
              )}
            </p>
          )}
        </div>
        {updateStatus === "has_update" && latestVersion && (
          <motion.div
            className="about-update-available"
            initial={{ opacity: 0, y: 4 }}
            animate={{ opacity: 1, y: 0 }}
          >
            <motion.button
              type="button"
              className="btn btn-primary btn-full"
              onClick={openReleasePage}
              whileHover={{ scale: 1.02 }}
              whileTap={{ scale: 0.98 }}
            >
              {t("about.updateAvailable")} (v{latestVersion})
            </motion.button>
            <p className="section-hint about-update-hint">
              {t("about.updateHint")}
            </p>
          </motion.div>
        )}
        </div>
      </motion.div>
    </motion.div>
  );
}
