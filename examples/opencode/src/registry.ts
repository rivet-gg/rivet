import { setup } from "rivetkit";
import { opencode } from "./actor.js";

export const registry = setup({ use: { opencode } });
