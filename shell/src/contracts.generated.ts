/* Generated from Python/Pydantic. Run npm run contracts; do not edit. */

export type BloomMessage = StateEvent | ErrorEvent | Authenticate | StartCommand | ChoiceCommand | ApprovalCommand;
export type Type = 'state';
export type Phase = 'idle' | 'starting' | 'choice' | 'approval';
export type RequestId = string;
export type ActionId = string;
export type Label = string;
export type Prob = number;
export type Candidates = Candidate[];
export type DecidedBy = string;
export type LatencyMs = number;
export type RunId = string;
export type ActionId1 = string;
/**
 * Ordered by "damage if we get it wrong". The floor is declared by the action;
 * a model can raise it, never lower it (policy.effective_criticality).
 */
export type Criticality = 0 | 1 | 2 | 3;
export type Preview = string;
export type Reasons = string[];
export type ExpiresAt = string;
export type RunId1 = string;
export type Id = string;
export type Kind = 'hotkey';
export type At = string;
export type RunStatus =
  'observing' | 'awaiting_choice' | 'awaiting_approval' | 'executing' | 'done' | 'cancelled' | 'failed';
export type ActionId2 = string;
export type Preview1 = string;
export type GeneratedBy = string | null;
export type GateReasons = string[];
export type RunId2 = string;
export type Verdict = 'approve' | 'reject' | 'edit';
export type EditedArgs = {
  [k: string]: unknown;
} | null;
export type Ok = boolean;
export type Detail = string;
export type UndoToken = string | null;
export type Error = string | null;
export type History = RunState[];
export type HistoryWarning = string | null;
export type Type1 = 'error';
export type Message = string;
export type Type2 = 'authenticate';
export type Token = string;
export type Type3 = 'start';
export type Type4 = 'choice';
export type RequestId1 = string;
export type ActionId3 = string | null;
export type ShownAt = string;
export type Type5 = 'approval';

export interface StateEvent {
  type: Type;
  state: BridgeState;
}
export interface BridgeState {
  phase: Phase;
  plan: ActionPlan | null;
  approval: ApprovalRequest | null;
  history: History;
  history_warning: HistoryWarning;
}
/**
 * What comes out of ranking and goes to the pie menu.
 */
export interface ActionPlan {
  request_id: RequestId;
  candidates: Candidates;
  decided_by: DecidedBy;
  latency_ms: LatencyMs;
}
export interface Candidate {
  action_id: ActionId;
  label: Label;
  prob: Prob;
}
export interface ApprovalRequest {
  run_id: RunId;
  action_id: ActionId1;
  effective: Criticality;
  preview: Preview;
  reasons: Reasons;
  expires_at: ExpiresAt;
}
/**
 * Serializable from day one: this is what will let a run be suspended and resumed.
 */
export interface RunState {
  run_id: RunId1;
  trigger: Trigger;
  status: RunStatus;
  plan: ActionPlan | null;
  chosen: ResolvedAction | null;
  effective: Criticality | null;
  gate_reasons: GateReasons;
  approval: ApprovalResponse | null;
  result: ActionResult | null;
  error: Error;
  timings_ms: TimingsMs;
}
export interface Trigger {
  id: Id;
  kind: Kind;
  at: At;
}
export interface ResolvedAction {
  action_id: ActionId2;
  args: Args;
  preview: Preview1;
  generated_by: GeneratedBy;
}
export interface Args {
  [k: string]: unknown;
}
export interface ApprovalResponse {
  run_id: RunId2;
  verdict: Verdict;
  edited_args: EditedArgs;
}
export interface ActionResult {
  ok: Ok;
  detail: Detail;
  undo_token: UndoToken;
}
export interface TimingsMs {
  [k: string]: number;
}
export interface ErrorEvent {
  type: Type1;
  message: Message;
}
export interface Authenticate {
  type: Type2;
  token: Token;
}
export interface StartCommand {
  type: Type3;
}
export interface ChoiceCommand {
  type: Type4;
  request_id: RequestId1;
  choice: MenuChoice;
}
export interface MenuChoice {
  action_id: ActionId3;
  shown_at: ShownAt;
}
export interface ApprovalCommand {
  type: Type5;
  response: ApprovalResponse;
}
