import { LogStrategy } from "./strategies/LogStrategy";

export enum LogLevel {
  TRACE = 1,
  DEBUG = 2,
  INFO = 3,
  WARN = 4,
  ERROR = 5,
}

export class Logger {
  constructor(private readonly context: string, private readonly logStrategy: LogStrategy) {}

  log(level: LogLevel, ...message: any[]): void {
    this.logStrategy.log(level, ...["[" + this.context + "]"].concat(...message));
  }

  trace(...message: any[]) {
    this.log(LogLevel.TRACE, ...message);
  }

  debug(...message: any[]) {
    this.log(LogLevel.DEBUG, ...message);
  }

  info(...message: any[]) {
    this.log(LogLevel.INFO, ...message);
  }

  warn(...message: any[]) {
    this.log(LogLevel.WARN, ...message);
  }

  error(...message: any[]) {
    this.log(LogLevel.ERROR, ...message);
  }
}
