import { createPinia } from "pinia";
import { App } from "vue";

export function registerPlugins (app: App) {
   app.use(createPinia());
}