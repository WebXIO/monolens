import { createPinia } from "pinia";
import { App } from "vue";
import { shortcuts } from "./shortcuts";

export function registerPlugins (app: App) {
   app.use(createPinia()).use(shortcuts);
}