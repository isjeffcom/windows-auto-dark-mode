import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";

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

  return (
    <motion.div
      className="panel"
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4, ease: [0.22, 0.61, 0.36, 1] }}
    >
      <div className="panel-title">{t("about.title")}</div>
      <motion.p
        className="section-hint"
        style={{ lineHeight: 1.65, marginBottom: "var(--space-4)" }}
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.1 }}
      >
        {t("about.description")}
      </motion.p>
      <motion.p
        className="section-hint"
        style={{ lineHeight: 1.65, marginBottom: "var(--space-4)" }}
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.1 }}
      >
        Github:{" "}
        <a
          style={{ color: "var(--text-secondary)" }}
          href="https://github.com/isjeffcom/windows-auto-dark-mode"
          target="_blank"
          rel="noopener noreferrer"
        >
          https://github.com/isjeffcom/windows-auto-dark-mode
        </a>
      </motion.p>
      <motion.p
        className="section-hint"
        style={{ marginTop: 24 }}
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.15 }}
      >
        {t("about.version")} {version}
      </motion.p>

      <motion.div
        className="section-hint"
        style={{ marginTop: "var(--space-5)", display: "flex", flexDirection: "column", gap: "var(--space-3)" }}
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.2 }}
      >
        <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)", alignItems: "flex-start" }}>
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
            <p className="section-hint" style={{ margin: 0, marginTop: "var(--space-1)" }}>
              {updateStatus === "current" && t("about.youAreUpToDate")}
              {updateStatus === "error" && (
                <span style={{ color: "var(--text-muted)" }}>{t("about.checkFailed")}</span>
              )}
            </p>
          )}
        </div>
        {updateStatus === "has_update" && latestVersion && (
          <motion.div
            initial={{ opacity: 0, y: 4 }}
            animate={{ opacity: 1, y: 0 }}
            style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)", marginTop: "var(--space-2)" }}
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
            <p className="section-hint" style={{ margin: 0, fontSize: "0.9em" }}>
              {t("about.updateHint")}
            </p>
          </motion.div>
        )}
      </motion.div>
    </motion.div>
  );
}
