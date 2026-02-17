import { useLogger } from "@/composables/useLogger";
import { Commands, ShortcutCommand, ShortcutMap } from "../models";

export class CommandRegistry {
  commands: ShortcutMap;
  disabled: boolean;
  subscribers: Map<string, ((...args: any[]) => void)[]>;
  logger;

  constructor() {
    this.commands = new Map();
    this.disabled = false;
    this.subscribers = new Map();
    this.logger = useLogger(CommandRegistry.name);
  }

  registerCommand(command: ShortcutCommand) {
    this.commands.set(command.id, command);
  }

  unregisterCommand(id: string) {
    this.commands.delete(id);
  }

  executeCommand(id: string) {
    if (this.disabled) return;

    
    const command = this.commands.get(id);
    this.logger.debug(`Execute Command: `, command);

    if (command) {
      command.execute();
      this.notify(id, command);
    }
  }

  notify(event: string, ...args: any[]) {
    const subs = this.subscribers.get(event);

    if (subs?.length) {
      subs.forEach((cb) => cb(...args));
    }
  }

  subscribe(event: Commands, subscriber: (...args: any[]) => void) {
    const subs = this.subscribers.get(event) ?? [];

    subs.push(subscriber);

    this.subscribers.set(event, subs);
  }

  unsubscribe(event: Commands, subscriber: (...args: any[]) => void) {
    const subs = this.subscribers.get(event);

    if (!subs?.length) return;

    const idx = subs.findIndex(subscriber);

    if (idx === -1) return;

    subs.splice(idx, 1);

    this.subscribers.set(event, subs);
  }

  disableExecution() {
    this.disabled = true;
  }

  enableExecution() {
    this.disabled = false;
  }
}
