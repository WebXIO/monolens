import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";
import { TabKind } from "@/domains/tabs";
import { useTabsStore } from "../tabsStore";

const users = { connectionId: "c1", database: "shop", collection: "users" };
const orders = { connectionId: "c1", database: "shop", collection: "orders" };
const otherServer = { connectionId: "c2", database: "shop", collection: "users" };

beforeEach(() => {
  setActivePinia(createPinia());
});

function open(store: ReturnType<typeof useTabsStore>, context: typeof users) {
  const id = store.addTab(context, TabKind.QUERY);
  store.switchTab(id);
  return id;
}

describe("tabsStore", () => {
  it("finds an open tab by connection, database and collection", () => {
    const store = useTabsStore();
    const id = open(store, users);

    expect(store.hasContext(users, TabKind.QUERY)).toBe(id);
    expect(store.hasContext(orders, TabKind.QUERY)).toBeNull();
    expect(store.hasContext(otherServer, TabKind.QUERY)).toBeNull();
    expect(store.hasContext(users, TabKind.INDEXES)).toBeNull();
  });

  it("returns to the tab used before when the active one closes", () => {
    const store = useTabsStore();
    const first = open(store, users);
    const second = open(store, orders);
    const third = open(store, otherServer);
    store.switchTab(first);

    store.deleteTab(first);
    expect(store.activeTab).toBe(third);

    store.deleteTab(third);
    expect(store.activeTab).toBe(second);
  });

  it("keeps the active tab when another one closes", () => {
    const store = useTabsStore();
    const first = open(store, users);
    const second = open(store, orders);

    store.deleteTab(first);

    expect(store.activeTab).toBe(second);
  });

  it("has no active tab after the last one closed", () => {
    const store = useTabsStore();
    open(store, users);

    store.closeActiveTab();

    expect(store.tabs.size).toBe(0);
    expect(store.activeTab).toBeNull();
    expect(store.getActiveTab()).toBeNull();
  });

  it("closes only the tabs of one connection", () => {
    const store = useTabsStore();
    open(store, users);
    open(store, orders);
    const kept = open(store, otherServer);

    store.closeTabsOf("c1");

    expect([...store.tabs.keys()]).toEqual([kept]);
    expect(store.activeTab).toBe(kept);
  });

  it("switches to a remaining tab when the active connection's tabs close", () => {
    const store = useTabsStore();
    const kept = open(store, otherServer);
    open(store, users);

    store.closeTabsOf("c1");

    expect(store.activeTab).toBe(kept);
  });

  it("ignores unknown tabs", () => {
    const store = useTabsStore();
    const id = open(store, users);
    store.switchTab("nope");
    store.deleteTab("nope");
    expect(store.activeTab).toBe(id);
  });
});
