import { LogLevel } from "../Logger";

export interface LogStrategy {
   log(level: LogLevel, ...message: any[]): void;
}