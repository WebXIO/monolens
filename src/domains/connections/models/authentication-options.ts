import { AuthenticationKind } from "./authentication-kind";

export type AuthenticationOptions = {
  kind: AuthenticationKind;
  username: string | null;
  password: string | null;
  database: string | null;
};
