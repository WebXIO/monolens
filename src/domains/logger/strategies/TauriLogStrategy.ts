import { debug, error, info, trace, warn } from "@tauri-apps/plugin-log";
import { LogStrategy } from "./LogStrategy";
import { LogLevel } from "../Logger";

export class TauriLogStrategy implements LogStrategy {
  log(level: LogLevel, ...message: any[]): void {
    const computedMessage = this.transformMessage(...message);
    switch (level) {
      case LogLevel.TRACE:
        trace(computedMessage);
        break

      case LogLevel.DEBUG:
        debug(computedMessage);
        break

      case LogLevel.INFO:
        info(computedMessage);
        break

      case LogLevel.WARN:
        warn(computedMessage);
        break

      case LogLevel.ERROR:
        error(computedMessage);
        break
    }
  }

  private transformMessage(...message: string[]): string {
    const parts = message.map((chunk: any) => {
      switch(typeof chunk) {
        case "object":
          return JSON.stringify(chunk)
        case "function":
          return `[Function: ${chunk.name}]`
        default:
          return chunk
      }
    })

    return parts.join(" ")
  }
}
