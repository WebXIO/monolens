import { Commands, KeyBinding } from "../models";

export const DefaultBindings: KeyBinding[] = [
  {
    commandId: Commands.TAB_CLOSE,
    keyCode: "ctrl+w",
  },
  {
    commandId: Commands.CONNECTION_DISCONNECT,
    keyCode: "ctrl+shift+d",
  },
  {
   commandId: Commands.EVENT_EXECUTE,
   keyCode: "ctrl+enter"
  },
];
