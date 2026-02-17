import { Context } from "@/domains/context";
import { TabKind } from "./tab.kind";

export type Tab = {
  id: string;
  context: Context;
  type: TabKind;
};
