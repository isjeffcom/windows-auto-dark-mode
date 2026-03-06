import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";

export default function About() {
  const { t } = useTranslation();

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
        Github: <a style={{ color: "var(--text-secondary)" }} href="https://github.com/isjeffcom/windows-auto-dark-mode" target="_blank" rel="noopener noreferrer">https://github.com/isjeffcom/windows-auto-dark-mode</a>
      </motion.p>
      <motion.p
        className="section-hint"
        style={{ marginTop: 24 }}
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.15 }}
      >
        {t("about.version")} 1.0.0
      </motion.p>
    </motion.div>
  );
}
