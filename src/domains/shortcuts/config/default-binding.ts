import { Commands, KeyBinding } from "../models";

export const DefaultBindings: KeyBinding[] = [
  {
    commandId: Commands.TAB_CLOSE,
    keyCode: "mod+w",
  },
  {
    commandId: Commands.CONNECTION_DISCONNECT,
    keyCode: "mod+shift+d",
  },
  {
   commandId: Commands.EVENT_EXECUTE,
   keyCode: "mod+enter"
  },
];
