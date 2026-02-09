import { AuthenticationKind } from "./authentication-kind";

export type AuthenticationOptions = {
  kind: AuthenticationKind;
  username: string;
  password: string;
  database: string;
};
