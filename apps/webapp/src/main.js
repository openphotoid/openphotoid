/* Token layers, imported individually so this build can substitute its
   own font declarations for the shared file's 28 — see src/fonts.css. */
import "$design/tokens/colors.css";
import "$design/tokens/typography.css";
import "$design/tokens/spacing.css";
import "$design/tokens/platform.css";
import "$design/tokens/radius.css";
import "$design/tokens/elevation.css";
import "$design/tokens/motion.css";
import "$design/tokens/base.css";
import "./fonts.css";
import "./app.css";

import { mount } from "svelte";
import App from "./App.svelte";

export default mount(App, { target: document.getElementById("app") });
