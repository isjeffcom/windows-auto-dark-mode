import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { motion, AnimatePresence } from "framer-motion";

import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import Dashboard from "./pages/Dashboard";
import Schedule from "./pages/Schedule";
import Settings from "./pages/Settings";
import About from "./pages/About";

type Page = "dashboard" | "schedule" | "settings" | "about";

const sidebarVariants = {
  hidden: { x: -28, opacity: 0 },
  visible: {
    x: 0,
    opacity: 1,
    transition: { duration: 0.45, ease: [0.22, 0.61, 0.36, 1], delay: 0.05 },
  },
};

const navItemVariants = {
  hidden: { x: -12, opacity: 0 },
  visible: (i: number) => ({
    x: 0,
    opacity: 1,
    transition: { duration: 0.35, ease: [0.22, 0.61, 0.36, 1], delay: 0.15 + i * 0.06 },
  }),
};

const mainVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { duration: 0.4, ease: [0.22, 0.61, 0.36, 1], delay: 0.2 },
  },
};

const pageVariants = {
  initial: { opacity: 0, y: 10 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -6 },
};

function App() {
  const { t } = useTranslation();
  const [page, setPage] = useState<Page>("dashboard");
  const [hasEntered, setHasEntered] = useState(false);

  useEffect(() => {
    const sync = () => {
      invoke<{ is_light: boolean }>("get_theme_state")
        .then((theme) => {
          document.documentElement.setAttribute("data-theme", theme.is_light ? "light" : "dark");
          getCurrentWindow().setTheme(theme.is_light ? "light" : "dark").catch(() => {});
        })
        .catch(() => {});
    };
    sync();
    const id = setInterval(sync, 5000);
    return () => clearInterval(id);
  }, []);

  useEffect(() => {
    const t = setTimeout(() => setHasEntered(true), 50);
    return () => clearTimeout(t);
  }, []);

  const navItems: { key: Page; labelKey: string }[] = [
    { key: "dashboard", labelKey: "nav.dashboard" },
    { key: "schedule", labelKey: "nav.schedule" },
    { key: "settings", labelKey: "nav.settings" },
    { key: "about", labelKey: "nav.about" },
  ];

  return (
    <div className="app">
      <motion.aside
        className="sidebar"
        variants={sidebarVariants}
        initial="hidden"
        animate={hasEntered ? "visible" : "hidden"}
      >
        <div className="sidebar-head">
          <motion.img
            src="/Tray_Icon.png"
            alt=""
            className="sidebar-logo"
            initial={{ scale: 0.8, opacity: 0 }}
            animate={hasEntered ? { scale: 1, opacity: 1 } : { scale: 0.8, opacity: 0 }}
            transition={{ duration: 0.4, delay: 0.1, ease: [0.22, 0.61, 0.36, 1] }}
          />
          <motion.span
            className="sidebar-title"
            initial={{ opacity: 0, y: 4 }}
            animate={hasEntered ? { opacity: 1, y: 0 } : { opacity: 0, y: 4 }}
            transition={{ duration: 0.3, delay: 0.18 }}
          >
            {t("app.title")}
          </motion.span>
        </div>
        <nav className="sidebar-nav">
          {navItems.map(({ key, labelKey }, i) => (
            <motion.button
              key={key}
              type="button"
              className={`sidebar-item ${page === key ? "active" : ""}`}
              onClick={() => setPage(key)}
              variants={navItemVariants}
              initial="hidden"
              animate={hasEntered ? "visible" : "hidden"}
              custom={i}
              whileTap={{ scale: 0.98 }}
              transition={{ type: "spring", stiffness: 400, damping: 25 }}
            >
              <span className="sidebar-item-label">{t(labelKey)}</span>
            </motion.button>
          ))}
        </nav>
      </motion.aside>
      <motion.div
        className="main-wrap"
        variants={mainVariants}
        initial="hidden"
        animate={hasEntered ? "visible" : "hidden"}
      >
        <main className="content">
          <AnimatePresence mode="wait">
            {page === "dashboard" && (
              <motion.div
                key="dashboard"
                variants={pageVariants}
                initial="initial"
                animate="animate"
                exit="exit"
                transition={{ duration: 0.22, ease: [0.22, 0.61, 0.36, 1] }}
              >
                <Dashboard />
              </motion.div>
            )}
            {page === "schedule" && (
              <motion.div
                key="schedule"
                variants={pageVariants}
                initial="initial"
                animate="animate"
                exit="exit"
                transition={{ duration: 0.22, ease: [0.22, 0.61, 0.36, 1] }}
              >
                <Schedule />
              </motion.div>
            )}
            {page === "settings" && (
              <motion.div
                key="settings"
                variants={pageVariants}
                initial="initial"
                animate="animate"
                exit="exit"
                transition={{ duration: 0.22, ease: [0.22, 0.61, 0.36, 1] }}
              >
                <Settings />
              </motion.div>
            )}
            {page === "about" && (
              <motion.div
                key="about"
                variants={pageVariants}
                initial="initial"
                animate="animate"
                exit="exit"
                transition={{ duration: 0.22, ease: [0.22, 0.61, 0.36, 1] }}
              >
                <About />
              </motion.div>
            )}
          </AnimatePresence>
        </main>
      </motion.div>
    </div>
  );
}

export default App;
