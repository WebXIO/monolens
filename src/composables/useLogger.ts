import { Logger } from "@/domains/logger/Logger";
import { LogStrategy } from "@/domains/logger/strategies/LogStrategy";
import { TauriLogStrategy } from "@/domains/logger/strategies/TauriLogStrategy";


export function useLogger(context?: string, strategy?: LogStrategy) {
   return new Logger(`WebView::${context ?? 'App'}`, strategy ?? new TauriLogStrategy());
}