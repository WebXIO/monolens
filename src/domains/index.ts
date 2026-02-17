import { App, inject } from "vue";
import { ConnectionRepository, ConnectionServiceIPC } from "./connections";
import { ConnectionRepositoryIPC } from "./connections/repositories/ConnectionRepositoryIPC";
import { CommandRegistry } from "./shortcuts";
import { useLogger } from "@/composables/useLogger";

const prefix = "$providers";
const logger = useLogger('Domain');

function provide(create: (context: any) => any) {
  return { create };
}

export function registerDomains(app: App) {
  const providers = [
    provide((_context) => new CommandRegistry()),
    provide((_context) => new ConnectionServiceIPC()),
    provide(
      (context) =>
        new ConnectionRepositoryIPC({
          connectionService: context.read(ConnectionServiceIPC),
        }) as ConnectionRepository,
    ),
  ];

  (app as any).read = <T>(token: new (...args: any[]) => T): T => {
    const searchFor = token.name;
    const returnVal = app.config.globalProperties[`${prefix}.${searchFor}`] as
      | T
      | undefined;

    if (!returnVal) throw new Error("Search term not found " + searchFor);

    return returnVal;
  };

  for (const provider of providers) {
    const result = provider.create(app);

    logger.debug(`Register domain ${prefix}.${result.constructor.name}`)

    app.provide(`${prefix}.${result.constructor.name}`, result);
    app.config.globalProperties[`${prefix}.${result.constructor.name}`] =
      result;
  }

  delete (app as any).read;
}

export function useDomain<T>(token: new (...args: any[]) => T, app?: App): T {
  return (
    app
      ? app.config.globalProperties[`${prefix}.${token.name}`]
      : inject(`${prefix}.${token.name}`)
  ) as T;
}
