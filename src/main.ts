import { createApp } from "vue";
import App from "./App.vue";
import { registerPlugins } from "./plugins";
import { registerDomains } from "./domains";

const app = createApp(App);

registerPlugins(app);
registerDomains(app);

app.mount("#app");
