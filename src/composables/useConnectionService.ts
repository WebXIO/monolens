import { getCurrentInstance, hasInjectionContext } from "vue";
import { ConnectionServiceIPC } from "@/domains/connections";

export function useConnectionService(): ConnectionServiceIPC {
  const context = hasInjectionContext();

  console.log(context);

  const vm = getCurrentInstance();
  if (!vm) throw new Error("useConnectionService must be called inside setup()");

  return (vm.appContext.config.globalProperties as any)["$providers.ConnectionServiceIPC"];
}
