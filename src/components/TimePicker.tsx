import { useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";

function parseTime(value: string): { hour: number; minute: number } {
  const [h, m] = value.split(":").map(Number);
  return { hour: Math.min(23, Math.max(0, isNaN(h) ? 0 : h)), minute: Math.min(59, Math.max(0, isNaN(m) ? 0 : m)) };
}

function formatTime(hour: number, minute: number): string {
  return `${String(hour).padStart(2, "0")}:${String(minute).padStart(2, "0")}`;
}

interface TimePickerProps {
  value: string;
  onChange: (value: string) => void;
  "aria-label"?: string;
}

export default function TimePicker({ value, onChange, "aria-label": ariaLabel }: TimePickerProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const { hour, minute } = parseTime(value);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const setHour = (h: number) => onChange(formatTime(Math.min(23, Math.max(0, h)), minute));
  const setMinute = (m: number) => onChange(formatTime(hour, Math.min(59, Math.max(0, m))));

  return (
    <div className="time-picker" ref={ref}>
      <motion.button
        type="button"
        className="time-picker-trigger"
        onClick={() => setOpen((o) => !o)}
        aria-label={ariaLabel}
        aria-expanded={open}
        aria-haspopup="dialog"
        whileHover={{ scale: 1.01 }}
        whileTap={{ scale: 0.98 }}
        transition={{ type: "spring", stiffness: 400, damping: 25 }}
      >
        <span className="time-picker-icon" aria-hidden>🕐</span>
        <motion.span
          className="time-picker-value"
          key={value}
          initial={{ opacity: 0.6, y: 2 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.2 }}
        >
          {value || "00:00"}
        </motion.span>
        <motion.span
          className="time-picker-chevron"
          animate={{ rotate: open ? 180 : 0 }}
          transition={{ duration: 0.25, ease: [0.22, 0.61, 0.36, 1] }}
        >
          ▼
        </motion.span>
      </motion.button>

      <AnimatePresence>
        {open && (
          <motion.div
            className="time-picker-popover"
            role="dialog"
            aria-label={ariaLabel}
            initial={{ opacity: 0, y: -8, scale: 0.96 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -6, scale: 0.98 }}
            transition={{ duration: 0.22, ease: [0.22, 0.61, 0.36, 1] }}
          >
            <div className="time-picker-row">
              <div className="time-picker-cell">
                <span className="time-picker-label">H</span>
                <div className="time-picker-stepper">
                  <motion.button
                    type="button"
                    className="time-picker-btn"
                    onClick={() => setHour(hour - 1)}
                    whileTap={{ scale: 0.92 }}
                    transition={{ type: "spring", stiffness: 400, damping: 25 }}
                  >
                    −
                  </motion.button>
                  <motion.span
                    className="time-picker-num"
                    key={`h-${hour}`}
                    initial={{ opacity: 0, y: 4 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.18 }}
                  >
                    {String(hour).padStart(2, "0")}
                  </motion.span>
                  <motion.button
                    type="button"
                    className="time-picker-btn"
                    onClick={() => setHour(hour + 1)}
                    whileTap={{ scale: 0.92 }}
                    transition={{ type: "spring", stiffness: 400, damping: 25 }}
                  >
                    +
                  </motion.button>
                </div>
              </div>
              <span className="time-picker-sep">:</span>
              <div className="time-picker-cell">
                <span className="time-picker-label">M</span>
                <div className="time-picker-stepper">
                  <motion.button
                    type="button"
                    className="time-picker-btn"
                    onClick={() => setMinute(minute - 1)}
                    whileTap={{ scale: 0.92 }}
                    transition={{ type: "spring", stiffness: 400, damping: 25 }}
                  >
                    −
                  </motion.button>
                  <motion.span
                    className="time-picker-num"
                    key={`m-${minute}`}
                    initial={{ opacity: 0, y: 4 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.18 }}
                  >
                    {String(minute).padStart(2, "0")}
                  </motion.span>
                  <motion.button
                    type="button"
                    className="time-picker-btn"
                    onClick={() => setMinute(minute + 1)}
                    whileTap={{ scale: 0.92 }}
                    transition={{ type: "spring", stiffness: 400, damping: 25 }}
                  >
                    +
                  </motion.button>
                </div>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
