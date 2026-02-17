export type ShortcutCommand = {
   id: string;
   name: string;
   execute: () => void;
}

export type ShortcutMap = Map<string, ShortcutCommand>;