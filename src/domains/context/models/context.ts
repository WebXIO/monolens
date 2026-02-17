import { Connection } from "@/domains/connections"

export type Context = {
   connection: Connection
   database: string;
   collection: string;
}