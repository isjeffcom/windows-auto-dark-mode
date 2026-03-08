import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

const SWITCH_COOLDOWN_MS = 10000;
const PROGRESS_INTERVAL_MS = 50;
const POLL_THEME_INTERVAL_MS = 400;

interface ThemeState {
  is_light: boolean;
  apps_light: boolean;
  system_light: boolean;
}

type NextSwitch = [string, boolean] | null;

// Orb via CSS div: gradient + box-shadow glow — no SVG filter, no bleed outside bounds
function ThemeOrb({ isLight, isSwitching }: { isLight: boolean; isSwitching: boolean }) {
  const darkGrad =
    "linear-gradient(135deg, #141414 0%, #707070 52%, #d8d8d8 100%)";
  const lightGrad =
    "linear-gradient(135deg, #b0b0b0 0%, #e8e8e8 50%, #ffffff 100%)";

  const darkGlow =
    "0 0 18px 5px rgba(200,200,200,0.13), 0 2px 8px rgba(0,0,0,0.45)";
  const lightGlow =
    "0 0 18px 5px rgba(255,255,255,0.20), 0 2px 6px rgba(0,0,0,0.18)";

  return (
    <motion.div
      className="orb"
      style={{
        background: isLight ? lightGrad : darkGrad,
        boxShadow: isLight ? lightGlow : darkGlow,
      }}
      animate={isSwitching ? { scale: [1, 0.93, 1] } : { scale: 1 }}
      transition={
        isSwitching
          ? { duration: 1.1, repeat: Infinity, ease: "easeInOut" }
          : { duration: 0.38, ease: [0.34, 1.56, 0.64, 1] }
      }
    />
  );
}

export default function Dashboard() {
  const { t } = useTranslation();
  const [theme, setTheme] = useState<ThemeState | null>(null);
  const [nextSwitch, setNextSwitch] = useState<NextSwitch | null>(null);
  const [enabled, setEnabled] = useState(true);
  const [loading, setLoading] = useState(true);
  const [isSwitching, setIsSwitching] = useState(false);
  const [switchProgress, setSwitchProgress] = useState(0);
  const progressIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const mountedRef = useRef(true);

  const refresh = useCallback(async () => {
    try {
      const [themeRes, configRes, nextRes] = await Promise.all([
        invoke<ThemeState>("get_theme_state"),
        invoke<{ enabled: boolean }>("get_config").then((c: unknown) => (c as { enabled: boolean })),
        invoke<NextSwitch | null>("get_next_switch"),
      ]);
      setTheme(themeRes);
      setEnabled(configRes.enabled);
      setNextSwitch(nextRes ?? null);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
    const id = setInterval(refresh, 10000);
    return () => clearInterval(id);
  }, [refresh]);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
      if (progressIntervalRef.current) {
        clearInterval(progressIntervalRef.current);
        progressIntervalRef.current = null;
      }
    };
  }, []);

  const setThemeTo = useCallback(
    async (light: boolean) => {
      if (isSwitching) return;

      // Our app switches immediately; system (taskbar, etc.) updates in the background.
      document.documentElement.setAttribute("data-theme", light ? "light" : "dark");
      getCurrentWindow().setTheme(light ? "light" : "dark").catch(() => {});
      setTheme((prev) =>
        prev ? { ...prev, is_light: light, apps_light: light, system_light: light } : null
      );

      setIsSwitching(true);
      setSwitchProgress(0);

      const start = Date.now();
      progressIntervalRef.current = setInterval(() => {
        const elapsed = Date.now() - start;
        const p = Math.min(100, (elapsed / SWITCH_COOLDOWN_MS) * 100);
        setSwitchProgress(p);
      }, PROGRESS_INTERVAL_MS);

      try {
        await invoke("set_theme", { light, is_manual: true });
      } catch (e) {
        console.error(e);
      }

      const stopProgressAndDismiss = () => {
        if (!mountedRef.current) return;
        if (progressIntervalRef.current) {
          clearInterval(progressIntervalRef.current);
          progressIntervalRef.current = null;
        }
        setSwitchProgress(100);
        setIsSwitching(false);
        refresh();
      };

      const pollUntilConfirmed = async () => {
        while (mountedRef.current) {
          await new Promise((r) => setTimeout(r, POLL_THEME_INTERVAL_MS));
          if (!mountedRef.current) return;
          try {
            const state = await invoke<ThemeState>("get_theme_state");
            if (state.is_light === light) {
              stopProgressAndDismiss();
              return;
            }
          } catch {
            // ignore
          }
        }
      };
      pollUntilConfirmed();
    },
    [isSwitching, refresh]
  );

  if (loading || !theme) {
    return (
      <div className="panel">
        <div className="panel-title">—</div>
        <p style={{ color: "var(--text-secondary)", fontSize: "13px" }}>{t("common.loading")}</p>
      </div>
    );
  }

  const nextStr = nextSwitch && Array.isArray(nextSwitch)
    ? t("dashboard.at", { time: nextSwitch[0] }) +
      " " +
      (nextSwitch[1] ? t("dashboard.toDark") : t("dashboard.toLight"))
    : t("dashboard.manual");

  const panelTransition = { duration: 0.38, ease: [0.22, 0.61, 0.36, 1] };
  const containerVariants = { hidden: {}, visible: { transition: { staggerChildren: 0.08, delayChildren: 0.05 } } };

  return (
    <motion.div initial="hidden" animate="visible" variants={containerVariants}>
      {/* ── Orb + switcher section ── */}
      <motion.div
        className="panel"
        variants={{ hidden: { opacity: 0, y: 10 }, visible: { opacity: 1, y: 0 } }}
        transition={panelTransition}
      >
        <div className="panel-title">{t("dashboard.currentTheme")}</div>
        <div className="theme-display">
          <AnimatePresence mode="wait">
            <motion.div
              key={theme.is_light ? "light-orb" : "dark-orb"}
              initial={{ scale: 0.72, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.72, opacity: 0 }}
              transition={{ duration: 0.36, ease: [0.34, 1.56, 0.64, 1] }}
            >
              <ThemeOrb isLight={theme.is_light} isSwitching={isSwitching} />
            </motion.div>
          </AnimatePresence>

          <AnimatePresence mode="wait">
            <motion.div
              key={theme.is_light ? "light-label" : "dark-label"}
              className="orb-label"
              style={{ marginTop:22 }}
              initial={{ opacity: 0, y: 5 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -5 }}
              transition={{ duration: 0.22 }}
            >
              {theme.is_light ? t("dashboard.light") : t("dashboard.dark")}
            </motion.div>
          </AnimatePresence>

          {isSwitching ? (
            <div className="theme-switching">
              <p className="theme-switching-text">{t("dashboard.switching")}</p>
              <p className="theme-switching-hint">{t("dashboard.switchingHint")}</p>
              <div className="progress-track" role="progressbar" aria-valuenow={switchProgress} aria-valuemin={0} aria-valuemax={100}>
                <motion.div
                  className="progress-fill"
                  initial={{ width: 0 }}
                  animate={{ width: `${switchProgress}%` }}
                  transition={{ duration: 0.05 }}
                />
              </div>
              <span className="progress-label">{Math.round(switchProgress)}%</span>
            </div>
          ) : (
            <div className="theme-btn-group">
              <motion.button
                className={`theme-btn ${!theme.is_light ? "active" : ""}`}
                onClick={() => setThemeTo(false)}
                disabled={isSwitching}
                whileTap={{ scale: 0.97 }}
              >
                {t("dashboard.dark")}
              </motion.button>
              <motion.button
                className={`theme-btn ${theme.is_light ? "active" : ""}`}
                onClick={() => setThemeTo(true)}
                disabled={isSwitching}
                whileTap={{ scale: 0.97 }}
              >
                {t("dashboard.light")}
              </motion.button>
            </div>
          )}
        </div>
      </motion.div>

      {/* ── Schedule status section ── */}
      <motion.div
        className="panel"
        variants={{ hidden: { opacity: 0, y: 10 }, visible: { opacity: 1, y: 0 } }}
        transition={panelTransition}
      >
        <div className="panel-title">{t("dashboard.enabled")}</div>
        <div className="section-hint" style={{ marginBottom: 6 }}>
          {t(enabled ? "dashboard.on" : "dashboard.off")} — {t("dashboard.scheduleHint")}
        </div>
        <div className="next-switch">
          {t("dashboard.nextSwitch")}: <strong>{nextStr}</strong>
        </div>
      </motion.div>
    </motion.div>
  );
}
