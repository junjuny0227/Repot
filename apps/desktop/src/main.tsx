import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "./styles.css";

// macOS gets an overlay title bar and sidebar vibrancy (tauri.macos.conf.json);
// styles key off this so other platforms keep opaque surfaces and the native title bar.
document.documentElement.dataset.platform = /Mac OS X|Macintosh/.test(navigator.userAgent) ? "mac" : "other";

createRoot(document.getElementById("root")!).render(<StrictMode><App /></StrictMode>);
