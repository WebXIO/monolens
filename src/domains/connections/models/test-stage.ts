export type StageStatus = "pending" | "passed" | "failed" | "skipped";

export interface TestStage {
  status: StageStatus;
  title: string;
  /** e.g. the detected version, or why a stage was skipped */
  detail: string | null;
}
