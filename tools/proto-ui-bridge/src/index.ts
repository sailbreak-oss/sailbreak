import { definePrototype, tw } from '@proto.ui/core';
import type { CapEntries, Prototype, TemplateChildren } from '@proto.ui/core';
import { createRuntimeSession, type RuntimeSession } from '@proto.ui/runtime';
import {
  AS_TRIGGER_GET_GROUP_EVENT_TARGET_CAP,
  AS_TRIGGER_GET_PROTO_CAP,
  AS_TRIGGER_INSTANCE_CAP,
  AS_TRIGGER_MERGE_GROUP_CAP,
  AS_TRIGGER_PARENT_CAP,
} from '@proto.ui/module-as-trigger';
import {
  EVENT_CANCEL_DEFAULT_ACTION_CAP,
  EVENT_GLOBAL_TARGET_CAP,
  EVENT_ROOT_TARGET_CAP,
} from '@proto.ui/module-event';
import { EXPOSE_EVENT_SINK_CAP } from '@proto.ui/module-expose-event';
import {
  FOCUS_BLUR_CAP,
  FOCUS_INSTANCE_TOKEN_CAP,
  FOCUS_IS_NATIVELY_FOCUSABLE_CAP,
  FOCUS_PARENT_CAP,
  FOCUS_REQUEST_FOCUS_CAP,
  FOCUS_ROOT_TARGET_CAP,
  FOCUS_RUN_IN_CALLBACK_CAP,
  FOCUS_SET_FOCUSABLE_CAP,
  FOCUS_TARGET_READY_CAP,
} from '@proto.ui/module-focus';
import { A11Y_PROJECT_CAP } from '@proto.ui/module-a11y';
import { EFFECTS_CAP } from '@proto.ui/module-feedback';
import { EXPOSE_STATE_SET_EXPOSES_CAP } from '@proto.ui/module-expose-state';
import { CONTEXT_INSTANCE_TOKEN_CAP, CONTEXT_PARENT_CAP } from '@proto.ui/module-context';
import {
  ANATOMY_INSTANCE_TOKEN_CAP,
  ANATOMY_PARENT_CAP,
  ANATOMY_GET_PROTO_CAP,
  ANATOMY_ROOT_TARGET_CAP,
  ANATOMY_ORDER_OBSERVER_CAP,
} from '@proto.ui/module-anatomy';
import { RULE_META_GET_CAP } from '@proto.ui/module-rule-meta';
import { asTextareaRoot } from '@proto.ui/prototypes-base/textarea';
import {
  TEXT_CONTROL_HOST_CAP,
  TEXT_CONTROL_RUN_IN_CALLBACK_CAP,
} from '@proto.ui/module-text-control';

import * as shadcn from '@proto.ui/prototypes-shadcn';

type BuildMetadata = {
  proto_ui_version?: unknown;
  proto_ui_commit?: unknown;
};

const BUILD_METADATA = (globalThis as unknown as {
  __sailbreak_proto_ui_metadata?: BuildMetadata;
}).__sailbreak_proto_ui_metadata;
const PROTO_UI_VERSION =
  typeof BUILD_METADATA?.proto_ui_version === 'string'
    ? BUILD_METADATA.proto_ui_version
    : 'main-snapshot';
const PROTO_UI_COMMIT =
  typeof BUILD_METADATA?.proto_ui_commit === 'string' ? BUILD_METADATA.proto_ui_commit : 'unrecorded';
const PROTOCOL_MAJOR = 1;
const PROTOCOL_MINOR = 0;
const HOST_NAME = 'sailbreak';
const GPUI_VERSION = '0.2.2';
const HOST_PLATFORM = 'embedded-quickjs';
const REGISTRY_DIGEST = `proto-ui-main@${PROTO_UI_COMMIT}`;
const MAX_BRIDGE_MESSAGE_BYTES = 256 * 1024;
const DOCUMENT_POSITION_PRECEDING = 2;
const DOCUMENT_POSITION_FOLLOWING = 4;
const nodeGlobal = globalThis as unknown as {
  Node?: { DOCUMENT_POSITION_PRECEDING: number; DOCUMENT_POSITION_FOLLOWING: number };
};
if (!nodeGlobal.Node) {
  nodeGlobal.Node = {
    DOCUMENT_POSITION_PRECEDING,
    DOCUMENT_POSITION_FOLLOWING,
  };
}
let nextSurfaceOrder = 0;
const bridgeMicrotasks: Array<() => void> = [];
const microtaskGlobal = globalThis as unknown as {
  queueMicrotask?: (callback: () => void) => void;
};
if (typeof microtaskGlobal.queueMicrotask !== 'function') {
  microtaskGlobal.queueMicrotask = (callback) => {
    if (bridgeMicrotasks.length >= 256) throw new Error('microtask queue overflow');
    bridgeMicrotasks.push(callback);
  };
}

type JsonPrimitive = null | boolean | number | string;
type JsonValue = JsonPrimitive | JsonValue[] | { [key: string]: JsonValue };
type JsonObject = { [key: string]: JsonValue };
type Listener = (event: unknown) => void;
type RuntimeSessionLike = Pick<RuntimeSession, 'mount' | 'unmount' | 'dispose' | 'controller'> & {
  readonly instancePhase: string;
  readonly mountEpoch: number;
  readonly mountPhase: string;
  invokeInCallbackScope?(callback: () => void): void;
};
type ModuleWiringLike = {
  attach(moduleName: string, entries: CapEntries): boolean;
};
type CommitSignal = { done(): void };
type WireTemplateNode =
  | { kind: 'container'; tag: string; style?: string[]; children?: WireTemplateNode[] }
  | { kind: 'text'; text: string }
  | { kind: 'slot'; slot_id: string }
  | { kind: 'svg'; tag: string; attributes: Record<string, string>; children?: WireTemplateNode[] };
type WireA11y = {
  role: string;
  name: string;
  disabled: boolean;
  focused: boolean;
  focus_visible: boolean;
  hidden: boolean;
  orientation?: string;
  selected?: boolean;
  toggled?: boolean;
  actions?: string[];
};
type TextControlEventType =
  | 'input'
  | 'change'
  | 'compositionstart'
  | 'compositionupdate'
  | 'compositionend';
type TextControlEvent = {
  type: TextControlEventType;
  value: string;
  composing: boolean;
  data: string | null;
  inputType: string | null;
};
type TextControlSelection = {
  start: number;
  end: number;
  direction: 'none' | 'forward' | 'backward';
};
type TextControlPatch = Record<string, unknown>;
type TextControlConnection = {
  patch: TextControlPatch;
  onEvent(event: TextControlEvent): void;
};
type TextControlHostState = {
  patch: TextControlPatch;
  connection: TextControlConnection | null;
  value: string;
  defaultValue: string;
  composing: boolean;
  selection: TextControlSelection;
  deferredValue: string | null;
  initialized: boolean;
  disposed: boolean;
};
type TextControlHostLease = {
  update(patch: TextControlPatch): void;
  snapshot(): { value: string; composing: boolean; selection: TextControlSelection };
  dispose(): void;
};
type WireParent = {
  session_id: string;
  instance_id: string;
  view_epoch: number;
  route_ref: string;
};

type WireEvent =
  | { type: 'registry'; proto_ui: string; keys: string[] }
  | {
      type: 'ready';
      handshake: {
        protocol: { major: number; minor: number };
        proto_ui: string;
        host: { name: string; gpui: string; platform: string };
        registry_digest: string;
      };
    }
  | {
      type: 'projection';
      projection: {
        session_id: string;
        instance_id: string;
        view_epoch: number;
        commit_id: number;
        template: WireTemplateNode[];
        slot: { slot_id: string; accessible_name: string };
        style: { tokens: string[] };
        a11y: WireA11y | null;
      };
    }
  | {
      type: 'style';
      session_id: string;
      instance_id: string;
      view_epoch: number;
      style: { tokens: string[] };
    }
  | {
      type: 'a11y';
      session_id: string;
      instance_id: string;
      view_epoch: number;
      a11y: WireA11y;
    }
  | {
      type: 'state';
      session_id: string;
      instance_id: string;
      view_epoch: number;
      values: JsonObject;
    }
  | {
      type: 'signal';
      session_id: string;
      instance_id: string;
      view_epoch: number;
      sequence: number;
      key: string;
    }
  | {
      type: 'text_control';
      session_id: string;
      instance_id: string;
      view_epoch: number;
      sequence: number;
      control_ref: string;
      event: TextControlEvent;
    }
  | {
      type: 'diagnostic';
      diagnostic: { code: string; detail: string; fatal: boolean };
    };

type Surface = {
  focused: boolean;
  readonly order: number;
  compareDocumentPosition(other: Surface): number;
  focus(): void;
  blur(): void;
};

type PendingCommit = {
  commit_id: number;
  view_epoch: number;
  signal: CommitSignal;
};

type DelayTask = {
  due: number;
  run(): void;
  cancel(): void;
};

class LogicalBus {
  private readonly listeners = new Map<string, Set<Listener>>();

  addEventListener(type: string, listener: Listener): void {
    const listeners = this.listeners.get(type) ?? new Set<Listener>();
    listeners.add(listener);
    this.listeners.set(type, listeners);
  }

  removeEventListener(type: string, listener: Listener): void {
    const listeners = this.listeners.get(type);
    listeners?.delete(listener);
    if (listeners?.size === 0) this.listeners.delete(type);
  }

  dispatch(type: string, event: unknown): void {
    for (const listener of [...(this.listeners.get(type) ?? [])]) listener(event);
  }
}

type SessionRecord = {
  session_id: string;
  instance_id: string;
  prototype: string;
  definition: Prototype;
  props: JsonObject;
  meta: JsonObject;
  slot: { slot_id: string; accessible_name: string };
  route_ref: string | null;
  parent_ref: WireParent | null;
  root_bus: LogicalBus;
  global_bus: LogicalBus;
  surface: Surface;
  ready_listeners: Set<() => void>;
  seen_press_samples: Set<string>;
  events: WireEvent[];
  session: RuntimeSessionLike | undefined;
  pending_commit: PendingCommit | undefined;
  commit_id: number;
  output_sequence: number;
  style_tokens: string[];
  text_control_ref: string;
  text_control: TextControlHostState | null;
  a11y: WireA11y | null;
  state_values: JsonObject;
  exposed_handles: Record<string, unknown>;
  state_unsubs: Array<() => void>;
  disposed: boolean;
  virtual_time: number;
  delay_tasks: Set<DelayTask>;
  scheduled_tasks: Array<() => void>;
};

const TEXTAREA_BASE_TOKENS =
  'flex min-h-16 w-full rounded-md border border-input bg-transparent px-3 py-2 text-base shadow-xs outline-none transition-colors';

const shadcnTextareaRoot = definePrototype({
  name: 'shadcn-textarea-root',
  modules: asTextareaRoot.modules,
  setup(def) {
    const base = asTextareaRoot();
    const state = base.stateHandles;
    if (!state) throw new Error('[shadcn-textarea-root] asTextareaRoot must project state handles.');
    const { disabled, focusVisible } = state;
    def.feedback.style.use(tw(TEXTAREA_BASE_TOKENS));
    def.rule({
      when: (w) => w.state(focusVisible).eq(true),
      intent: (i) => i.feedback.style.use(tw('border-ring ring-3 ring-ring/50')),
    });
    def.rule({
      when: (w) => w.state(disabled).eq(true),
      intent: (i) => i.feedback.style.use(tw('cursor-not-allowed opacity-50')),
    });
    def.rule({
      when: (w) => w.all(w.meta('colorScheme').eq('dark'), w.state(disabled).eq(false)),
      intent: (i) => i.feedback.style.use(tw('bg-input/30')),
    });
  },
});

type Registry = Record<string, Prototype>;

const registry: Registry = {
  'shadcn-button': shadcn.shadcnButton,
  'shadcn-toggle': shadcn.shadcnToggle,
  'shadcn-checkbox-root': shadcn.shadcnCheckboxRoot,
  'shadcn-checkbox-indicator': shadcn.shadcnCheckboxIndicator,
  'shadcn-separator-root': shadcn.shadcnSeparatorRoot,
  'shadcn-switch-root': shadcn.shadcnSwitchRoot,
  'shadcn-switch-thumb': shadcn.shadcnSwitchThumb,
  'shadcn-tabs-root': shadcn.shadcnTabsRoot,
  'shadcn-textarea-root': shadcnTextareaRoot,
  'shadcn-tabs-list': shadcn.shadcnTabsList,
  'shadcn-tabs-trigger': shadcn.shadcnTabsTrigger,
  'shadcn-tabs-content': shadcn.shadcnTabsContent,
  'shadcn-hover-card-root': shadcn.shadcnHoverCardRoot,
  'shadcn-hover-card-trigger': shadcn.shadcnHoverCardTrigger,
  'shadcn-hover-card-content': shadcn.shadcnHoverCardContent,
  'shadcn-dropdown-root': shadcn.shadcnDropdownRoot,
  'shadcn-dropdown-trigger': shadcn.shadcnDropdownTrigger,
  'shadcn-dropdown-content': shadcn.shadcnDropdownContent,
  'shadcn-dropdown-item': shadcn.shadcnDropdownItem,
  'shadcn-select-root': shadcn.shadcnSelectRoot,
  'shadcn-select-trigger': shadcn.shadcnSelectTrigger,
  'shadcn-select-value': shadcn.shadcnSelectValue,
  'shadcn-select-content': shadcn.shadcnSelectContent,
  'shadcn-select-item': shadcn.shadcnSelectItem,
  'shadcn-dialog-root': shadcn.shadcnDialogRoot,
  'shadcn-dialog-trigger': shadcn.shadcnDialogTrigger,
  'shadcn-dialog-mask': shadcn.shadcnDialogMask,
  'shadcn-dialog-content': shadcn.shadcnDialogContent,
  'shadcn-dialog-title': shadcn.shadcnDialogTitle,
  'shadcn-dialog-description': shadcn.shadcnDialogDescription,
  'shadcn-dialog-close': shadcn.shadcnDialogClose,
  'shadcn-dialog-close-icon': shadcn.shadcnDialogCloseIcon,
  'shadcn-dialog-header': shadcn.shadcnDialogHeader,
  'shadcn-dialog-footer': shadcn.shadcnDialogFooter,
};

const sessions = new Map<string, SessionRecord>();
let bridgeFailed = false;

function recordOf(value: unknown): Record<string, unknown> | null {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) return null;
  return value as Record<string, unknown>;
}

function stringValue(value: unknown): string | null {
  return typeof value === 'string' ? value : null;
}

function numberValue(value: unknown): number | null {
  return typeof value === 'number' && Number.isSafeInteger(value) ? value : null;
}

function booleanValue(value: unknown): boolean | null {
  return typeof value === 'boolean' ? value : null;
}

function getRequiredString(object: Record<string, unknown>, key: string): string {
  const value = stringValue(object[key]);
  if (value === null || value.trim().length === 0) throw new Error(`missing string field: ${key}`);
  return value;
}

function getRequiredNumber(object: Record<string, unknown>, key: string): number {
  const value = numberValue(object[key]);
  if (value === null || value <= 0) throw new Error(`invalid positive number field: ${key}`);
  return value;
}

function jsonValue(value: unknown, depth = 0): JsonValue | undefined {
  if (depth > 16) return undefined;
  if (value === null) return null;
  if (typeof value === 'string' || typeof value === 'boolean') return value;
  if (typeof value === 'number') return Number.isFinite(value) ? value : undefined;
  if (Array.isArray(value)) {
    const values: JsonValue[] = [];
    for (const item of value) {
      const next = jsonValue(item, depth + 1);
      if (typeof next === 'undefined') return undefined;
      values.push(next);
    }
    return values;
  }
  const object = recordOf(value);
  if (!object) return undefined;
  const result: { [key: string]: JsonValue } = {};
  for (const [key, item] of Object.entries(object)) {
    const next = jsonValue(item, depth + 1);
    if (typeof next === 'undefined') return undefined;
    result[key] = next;
  }
  return result;
}

function utf8Length(value: string): number {
  let length = 0;
  for (const character of value) {
    const code = character.codePointAt(0) ?? 0;
    if (code <= 0x7f) length += 1;
    else if (code <= 0x7ff) length += 2;
    else if (code <= 0xffff) length += 3;
    else length += 4;
  }
  return length;
}

function jsonObject(value: unknown): JsonObject {
  const parsed = jsonValue(value);
  if (!parsed || Array.isArray(parsed) || typeof parsed !== 'object') {
    throw new Error('expected a JSON object');
  }
  return parsed;
}

function styleTokens(value: unknown): string[] {
  const object = recordOf(value);
  if (!object) return [];
  const tokens = object.tokens;
  if (!Array.isArray(tokens)) return [];
  return tokens.filter((token): token is string => typeof token === 'string');
}

function templateNode(value: unknown, record: SessionRecord): WireTemplateNode | null {
  if (typeof value === 'string' || typeof value === 'number') {
    return { kind: 'text', text: String(value) };
  }
  const object = recordOf(value);
  if (!object) return null;
  if (object.kind === 'svg-node') {
    const tag = stringValue(object.tag);
    if (!tag) throw new Error('SVG template node is missing a tag');
    const children = templateChildren(object.children, record);
    const attributes = svgAttributes(object.props, tag);
    return {
      kind: 'svg',
      tag,
      attributes,
      ...(children.length > 0 ? { children } : {}),
    };
  }
  const type = object.type;
  const reserved = recordOf(type);
  if (reserved && reserved.kind === 'slot') {
    return { kind: 'slot', slot_id: record.slot.slot_id };
  }
  if (typeof type !== 'string') {
    throw new Error('unsupported Proto UI template node');
  }
  const children = templateChildren(object.children, record);
  const style = styleTokens(object.style);
  return {
    kind: 'container',
    tag: type,
    ...(style.length > 0 ? { style } : {}),
    ...(children.length > 0 ? { children } : {}),
  };
}

function svgAttributes(value: unknown, tag: string): Record<string, string> {
  const object = recordOf(value);
  if (!object) throw new Error(`SVG ${tag} props must be an object`);
  const attributes: Record<string, string> = {};
  for (const [name, candidate] of Object.entries(object)) {
    if (
      typeof candidate !== 'string' &&
      typeof candidate !== 'number' &&
      typeof candidate !== 'boolean'
    ) {
      throw new Error(`SVG ${tag} attribute ${name} is not a primitive`);
    }
    attributes[name] = String(candidate);
  }
  return attributes;
}

function templateChildren(value: unknown, record: SessionRecord): WireTemplateNode[] {
  const values = Array.isArray(value) ? value : [value];
  const nodes: WireTemplateNode[] = [];
  for (const item of values) {
    if (item === null || typeof item === 'boolean' || typeof item === 'undefined') continue;
    const node = templateNode(item, record);
    if (node) nodes.push(node);
  }
  return nodes;
}

function a11ySnapshot(value: unknown, record: SessionRecord): WireA11y {
  const object = recordOf(value);
  const states = object ? recordOf(object.states) : null;
  const tree = object ? recordOf(object.tree) : null;
  const nameObject = object ? recordOf(object.name) : null;
  const name = nameObject?.kind === 'content'
    ? record.slot.accessible_name
    : stringValue(nameObject?.value) ?? stringValue(object?.name) ?? '';
  const actionsObject = object ? recordOf(object.actions) : null;
  const actions = actionsObject ? Object.keys(actionsObject) : [];
  const selected = booleanValue(states?.selected);
  const toggled = booleanValue(states?.checked) ?? booleanValue(states?.pressed);
  const orientation = stringValue(states?.orientation);
  return {
    role: stringValue(object?.role) ?? 'generic',
    name,
    disabled: booleanValue(states?.disabled) ?? false,
    focused: booleanValue(states?.focused) ?? false,
    focus_visible: booleanValue(states?.focusVisible) ?? false,
    hidden: booleanValue(tree?.hidden) ?? false,
    ...(orientation !== null && orientation.length > 0 ? { orientation } : {}),
    ...(selected !== null ? { selected } : {}),
    ...(toggled !== null ? { toggled } : {}),
    ...(actions.length > 0 ? { actions } : {}),
  };
}

function exposedStates(value: unknown): JsonObject {
  const object = recordOf(value);
  if (!object) return {};
  const result: JsonObject = {};
  for (const [key, candidate] of Object.entries(object)) {
    const state = recordOf(candidate);
    const getter = state?.get;
    const resolved = typeof getter === 'function' ? getter.call(candidate) : candidate;
    const json = jsonValue(resolved);
    if (typeof json !== 'undefined') result[key] = json;
  }
  return result;
}

function nextOutputSequence(record: SessionRecord): number {
  record.output_sequence += 1;
  return record.output_sequence;
}

function emitDiagnostic(record: SessionRecord, code: string, detail: string, fatal: boolean): void {
  record.events.push({ type: 'diagnostic', diagnostic: { code, detail, fatal } });
}

function cancelDelayTasks(record: SessionRecord): void {
  for (const task of [...record.delay_tasks]) task.cancel();
}

function scheduleTask(record: SessionRecord, task: () => void): void {
  if (record.scheduled_tasks.length >= 256) {
    bridgeFailed = true;
    throw new Error(`scheduled task overflow for session ${record.session_id}`);
  }
  record.scheduled_tasks.push(task);
}

function flushScheduledTasks(): void {
  let executed = 0;
  while (true) {
    let task = bridgeMicrotasks.shift();
    if (!task) {
      const record = [...sessions.values()].find(
        (candidate) => candidate.scheduled_tasks.length > 0,
      );
      task = record?.scheduled_tasks.shift();
    }
    if (!task) return;
    executed += 1;
    if (executed > 1024) {
      bridgeFailed = true;
      throw new Error('scheduled task flush overflow');
    }
    task();
  }
}

function advanceTime(record: SessionRecord, milliseconds: number): void {
  if (!Number.isSafeInteger(milliseconds) || milliseconds < 0) {
    throw new Error('advance_time requires a non-negative safe integer');
  }
  if (record.virtual_time > Number.MAX_SAFE_INTEGER - milliseconds) {
    throw new Error('virtual clock overflow');
  }
  record.virtual_time += milliseconds;
  while (true) {
    const due = [...record.delay_tasks]
      .filter((task) => task.due <= record.virtual_time)
      .sort((left, right) => left.due - right.due);
    if (due.length === 0) return;
    due[0].run();
  }
}

function emitStyle(record: SessionRecord): void {
  if (record.disposed) return;
  record.events.push({
    type: 'style',
    session_id: record.session_id,
    instance_id: record.instance_id,
    view_epoch: record.session?.mountEpoch ?? 1,
    style: { tokens: [...record.style_tokens] },
  });
}

function emitStateValues(record: SessionRecord): void {
  if (record.disposed) return;
  record.state_values = exposedStates(record.exposed_handles);
  record.events.push({
    type: 'state',
    session_id: record.session_id,
    instance_id: record.instance_id,
    view_epoch: record.session?.mountEpoch ?? 1,
    values: { ...record.state_values },
  });
}

function emitA11y(record: SessionRecord, value: unknown): void {
  if (record.disposed) return;
  record.a11y = a11ySnapshot(value, record);
  record.events.push({
    type: 'a11y',
    session_id: record.session_id,
    instance_id: record.instance_id,
    view_epoch: record.session?.mountEpoch ?? 1,
    a11y: record.a11y,
  });
}

function setExposes(record: SessionRecord, value: unknown): void {
  if (record.disposed) return;
  for (const unsubscribe of record.state_unsubs.splice(0)) unsubscribe();
  record.exposed_handles = recordOf(value) ?? {};
  emitStateValues(record);
  for (const candidate of Object.values(record.exposed_handles)) {
    const candidateObject = recordOf(candidate);
    const subscribe = candidateObject?.subscribe;
    if (typeof subscribe !== 'function') continue;
    const unsubscribe = subscribe.call(candidate, () => emitStateValues(record));
    if (typeof unsubscribe === 'function') record.state_unsubs.push(unsubscribe);
  }
}

function parentRecordFor(record: SessionRecord): SessionRecord | null {
  const parent = record.parent_ref;
  if (!parent) return null;
  const candidate = sessions.get(parent.session_id);
  if (!candidate || candidate.disposed) return null;
  if (candidate.instance_id !== parent.instance_id) return null;
  if (candidate.route_ref !== parent.route_ref) return null;
  if (candidate.session?.mountEpoch !== parent.view_epoch) return null;
  if (candidate.session?.mountPhase === 'detached' || candidate.session?.mountPhase === 'unmounting') {
    return null;
  }
  return candidate;
}

function parentTokenFor(instance: unknown): unknown | null {
  for (const record of sessions.values()) {
    if (record.surface === instance) return parentRecordFor(record)?.surface ?? null;
  }
  return null;
}

function prototypeFor(instance: unknown): Prototype | null {
  for (const record of sessions.values()) {
    if (record.surface === instance) return record.definition;
  }
  return null;
}

function rootRecordFor(record: SessionRecord): SessionRecord {
  let current = record;
  const visited = new Set<string>();
  while (!visited.has(current.session_id)) {
    visited.add(current.session_id);
    const parent = parentRecordFor(current);
    if (!parent) break;
    current = parent;
  }
  return current;
}

function rootTargetFor(instance: unknown): Surface | null {
  for (const record of sessions.values()) {
    if (record.surface === instance) return record.surface;
  }
  return null;
}

function familyGlobalBus(record: SessionRecord): LogicalBus {
  return rootRecordFor(record).global_bus;
}

function recordForSurface(surface: Surface): SessionRecord | null {
  for (const record of sessions.values()) {
    if (record.surface === surface) return record;
  }
  return null;
}

function blurSurface(surface: Surface): void {
  if (!surface.focused) return;
  surface.focused = false;
  const record = recordForSurface(surface);
  record?.root_bus.dispatch('host:blur', { target: surface, nativeEvent: { target: surface } });
}

function focusSurface(surface: Surface): void {
  if (surface.focused) return;
  const record = recordForSurface(surface);
  if (!record) return;
  const root = rootRecordFor(record);
  for (const candidate of sessions.values()) {
    if (candidate.surface !== surface && rootRecordFor(candidate) === root) {
      blurSurface(candidate.surface);
    }
  }
  surface.focused = true;
  record.root_bus.dispatch('host:focus', { target: surface, nativeEvent: { target: surface } });
}

function parseParent(value: unknown): WireParent | null {
  if (typeof value === 'undefined' || value === null) return null;
  const object = recordOf(value);
  if (!object) throw new Error('parent must be a JSON object');
  return {
    session_id: getRequiredString(object, 'session_id'),
    instance_id: getRequiredString(object, 'instance_id'),
    view_epoch: getRequiredNumber(object, 'view_epoch'),
    route_ref: getRequiredString(object, 'route_ref'),
  };
}

function validateParent(parent: WireParent): void {
  const candidate = sessions.get(parent.session_id);
  if (!candidate) throw new Error(`unknown parent session: ${parent.session_id}`);
  if (candidate.instance_id !== parent.instance_id) {
    throw new Error(`parent instance mismatch: ${parent.instance_id}`);
  }
  if (candidate.route_ref !== parent.route_ref) {
    throw new Error(`parent route mismatch: expected ${candidate.route_ref ?? ''}/${parent.route_ref}`);
  }
  const epoch = candidate.session?.mountEpoch ?? 0;
  if (epoch !== parent.view_epoch) {
    throw new Error(`stale parent view epoch: expected ${epoch}/${parent.view_epoch}`);
  }
  if (candidate.disposed || candidate.session?.mountPhase === 'detached' || candidate.session?.mountPhase === 'unmounting') {
    throw new Error(`parent session is not mounted: ${parent.session_id}`);
  }
}

function textControlPatchValue(
  state: TextControlHostState,
  patch: TextControlPatch,
): string | null {
  if (typeof patch.value === 'string') return patch.value;
  if (!state.initialized && patch.valueMode === 'uncontrolled' && typeof patch.defaultValue === 'string') {
    return patch.defaultValue;
  }
  return null;
}

function replaceTextControlValue(state: TextControlHostState, value: string): void {
  state.value = value;
  state.selection = {
    ...state.selection,
    start: Math.min(state.selection.start, value.length),
    end: Math.min(state.selection.end, value.length),
  };
}

function applyTextControlPatch(
  state: TextControlHostState,
  patch: TextControlPatch,
  allowValueProjection: boolean,
): void {
  state.patch = { ...state.patch, ...patch };
  if (typeof patch.defaultValue === 'string') state.defaultValue = patch.defaultValue;
  const value = textControlPatchValue(state, patch);
  state.initialized = true;
  if (value === null || value === state.value) return;
  if (allowValueProjection) {
    replaceTextControlValue(state, value);
    state.deferredValue = null;
  } else {
    state.deferredValue = value;
  }
}

function createTextControlHost(record: SessionRecord): { attach(connection: TextControlConnection): TextControlHostLease } {
  return {
    attach(connection) {
      if (!connection || typeof connection !== 'object' || typeof connection.onEvent !== 'function') {
        throw new Error('[TextControl] host connection is invalid.');
      }
      if (record.text_control) {
        record.text_control.disposed = true;
        record.text_control.connection = null;
      }
      const initialPatch = { ...connection.patch };
      const state: TextControlHostState = {
        patch: initialPatch,
        connection,
        value: '',
        defaultValue: typeof initialPatch.defaultValue === 'string' ? initialPatch.defaultValue : '',
        composing: false,
        selection: { start: 0, end: 0, direction: 'none' },
        deferredValue: null,
        initialized: false,
        disposed: false,
      };
      record.text_control = state;
      applyTextControlPatch(state, initialPatch, true);
      return {
        update(next) {
          if (state.disposed) return;
          applyTextControlPatch(state, next, !state.composing);
        },
        snapshot() {
          return Object.freeze({
            value: state.value,
            composing: state.composing,
            selection: { ...state.selection },
          });
        },
        dispose() {
          if (state.disposed) return;
          state.disposed = true;
          state.connection = null;
          if (record.text_control === state) record.text_control = null;
        },
      };
    },
  };
}

function textControlExposeEvent(
  record: SessionRecord,
  key: string,
  payload: unknown,
): TextControlEvent | null {
  const typeByKey: Record<string, TextControlEventType> = {
    valueChange: 'input',
    change: 'change',
    compositionStart: 'compositionstart',
    compositionUpdate: 'compositionupdate',
    compositionEnd: 'compositionend',
  };
  const type = typeByKey[key];
  if (!type || !record.text_control || record.text_control.disposed) return null;
  const object = recordOf(payload);
  if (!object || typeof object.value !== 'string') return null;
  return Object.freeze({
    type,
    value: object.value,
    composing: typeof object.composing === 'boolean'
      ? object.composing
      : type === 'compositionstart' || type === 'compositionupdate',
    data: typeof object.data === 'string' ? object.data : null,
    inputType: typeof object.inputType === 'string' ? object.inputType : null,
  });
}

function attachCapabilities(record: SessionRecord, wiring: ModuleWiringLike): void {
  const parentGetter = (instance: unknown): unknown | null => parentTokenFor(instance);
  wiring.attach('rule-meta', [
    [RULE_META_GET_CAP, (key: string) => record.meta[key]],
  ]);
  wiring.attach('context', [
    [CONTEXT_INSTANCE_TOKEN_CAP, record.surface],
    [CONTEXT_PARENT_CAP, parentGetter],
  ]);
  wiring.attach('anatomy', [
    [ANATOMY_INSTANCE_TOKEN_CAP, record.surface],
    [ANATOMY_PARENT_CAP, parentGetter],
    [ANATOMY_GET_PROTO_CAP, (instance: unknown) => prototypeFor(instance)],
    [ANATOMY_ROOT_TARGET_CAP, (instance: unknown) => rootTargetFor(instance)],
    [ANATOMY_ORDER_OBSERVER_CAP, () => () => undefined],
  ]);
  wiring.attach('as-trigger', [
    [AS_TRIGGER_INSTANCE_CAP, record.surface],
    [AS_TRIGGER_PARENT_CAP, parentGetter],
    [AS_TRIGGER_GET_PROTO_CAP, (instance: unknown) => prototypeFor(instance)],
    [AS_TRIGGER_MERGE_GROUP_CAP, () => undefined],
    [AS_TRIGGER_GET_GROUP_EVENT_TARGET_CAP, (instance: unknown) => {
      const target = rootTargetFor(instance);
      if (!target) return record.root_bus;
      for (const candidate of sessions.values()) {
        if (candidate.surface === target) return candidate.root_bus;
      }
      return record.root_bus;
    }],
  ]);
  wiring.attach('event', [
    [EVENT_ROOT_TARGET_CAP, () => record.root_bus],
    [EVENT_GLOBAL_TARGET_CAP, () => familyGlobalBus(record)],
    [EVENT_CANCEL_DEFAULT_ACTION_CAP, () => {
      emitDiagnostic(
        record,
        'default-action-not-applicable',
        'GPUI owns the native default action; no browser cancellation was claimed',
        false,
      );
    }],
  ]);
  wiring.attach('expose-event', [
    [EXPOSE_EVENT_SINK_CAP, (key: string, payload: unknown) => {
      const textEvent = textControlExposeEvent(record, key, payload);
      if (textEvent) {
        record.events.push({
          type: 'text_control',
          session_id: record.session_id,
          instance_id: record.instance_id,
          view_epoch: record.session?.mountEpoch ?? 1,
          sequence: nextOutputSequence(record),
          control_ref: record.text_control_ref,
          event: textEvent,
        });
        return;
      }
      record.events.push({
        type: 'signal',
        session_id: record.session_id,
        instance_id: record.instance_id,
        view_epoch: record.session?.mountEpoch ?? 1,
        sequence: nextOutputSequence(record),
        key,
      });
    }],
  ]);
  wiring.attach('text-control', [
    [TEXT_CONTROL_HOST_CAP, createTextControlHost(record)],
    [TEXT_CONTROL_RUN_IN_CALLBACK_CAP, (callback: () => void) => {
      const session = record.session;
      if (session?.invokeInCallbackScope) session.invokeInCallbackScope(callback);
      else callback();
    }],
  ]);
  wiring.attach('focus', [
    [FOCUS_ROOT_TARGET_CAP, () => record.surface],
    [FOCUS_TARGET_READY_CAP, (listener: () => void) => {
      record.ready_listeners.add(listener);
      return () => record.ready_listeners.delete(listener);
    }],
    [FOCUS_INSTANCE_TOKEN_CAP, record.surface],
    [FOCUS_PARENT_CAP, parentGetter],
    [FOCUS_IS_NATIVELY_FOCUSABLE_CAP, () => true],
    // GPUI owns tab-stop projection; the QuickJS surface remains programmatically focusable.
    [FOCUS_SET_FOCUSABLE_CAP, (_target: Surface, _enabled: boolean) => undefined],
    [FOCUS_REQUEST_FOCUS_CAP, (target: Surface) => {
      target.focus();
      return target.focused;
    }],
    [FOCUS_BLUR_CAP, (target: Surface) => target.blur()],
    [FOCUS_RUN_IN_CALLBACK_CAP, (callback: () => void) => {
      const session = record.session;
      if (session?.invokeInCallbackScope) session.invokeInCallbackScope(callback);
      else callback();
    }],
  ]);
  wiring.attach('feedback', [
    [EFFECTS_CAP, {
      queueStyle: (handle: { tokens?: unknown }) => {
        record.style_tokens = styleTokens(handle);
      },
      requestFlush: () => emitStyle(record),
    }],
  ]);
  wiring.attach('a11y', [
    [A11Y_PROJECT_CAP, (snapshot: unknown) => emitA11y(record, snapshot)],
  ]);
  wiring.attach('expose-state', [
    [EXPOSE_STATE_SET_EXPOSES_CAP, (exposes: unknown) => setExposes(record, exposes)],
  ]);
}

function projection(record: SessionRecord, children: TemplateChildren): WireEvent {
  const viewEpoch = record.session?.mountEpoch ?? 1;
  record.commit_id += 1;
  const template = templateChildren(children, record);
  record.pending_commit = {
    commit_id: record.commit_id,
    view_epoch: viewEpoch,
    signal: record.pending_commit?.signal ?? { done: () => undefined },
  };
  return {
    type: 'projection',
    projection: {
      session_id: record.session_id,
      instance_id: record.instance_id,
      view_epoch: viewEpoch,
      commit_id: record.commit_id,
      template,
      slot: { ...record.slot },
      style: { tokens: [...record.style_tokens] },
      a11y: record.a11y,
    },
  };
}

function createRecord(
  sessionId: string,
  instanceId: string,
  prototype: string,
  definition: Prototype,
  props: JsonObject,
  meta: JsonObject,
  slot: { slot_id: string; accessible_name: string },
  routeRef: string | null,
  parentRef: WireParent | null,
): SessionRecord {
  const order = nextSurfaceOrder++;
  const surface: Surface = {
    focused: false,
    order,
    compareDocumentPosition(other) {
      if (surface === other) return 0;
      return surface.order < other.order
        ? DOCUMENT_POSITION_FOLLOWING
        : DOCUMENT_POSITION_PRECEDING;
    },
    focus() {
      focusSurface(surface);
    },
    blur() {
      blurSurface(surface);
    },
  };
  return {
    session_id: sessionId,
    instance_id: instanceId,
    prototype,
    definition,
    props,
    meta,
    slot,
    route_ref: routeRef,
    parent_ref: parentRef,
    root_bus: new LogicalBus(),
    global_bus: new LogicalBus(),
    surface,
    ready_listeners: new Set<() => void>(),
    seen_press_samples: new Set<string>(),
    events: [],
    session: undefined,
    pending_commit: undefined,
    commit_id: 0,
    output_sequence: 0,
    style_tokens: [],
    text_control_ref: `${sessionId}:text-control`,
    text_control: null,
    a11y: null,
    state_values: {},
    exposed_handles: {},
    state_unsubs: [],
    disposed: false,
    virtual_time: 0,
    delay_tasks: new Set<DelayTask>(),
    scheduled_tasks: [],
  };
}

function runtimeHost(record: SessionRecord) {
  return {
    prototypeName: record.prototype,
    getRawProps: () => record.props,
    schedule: (task: () => void) => scheduleTask(record, task),
    scheduleDelay: (durationMs: number, task: () => void) => {
      if (!Number.isFinite(durationMs) || durationMs < 0) {
        emitDiagnostic(record, 'delayed-task-dropped', 'delay must be a finite non-negative number', false);
        return { cancel: () => undefined };
      }
      if (durationMs === 0) {
        let active = true;
        scheduleTask(record, () => {
          if (!active) return;
          active = false;
          task();
        });
        return { cancel: () => { active = false; } };
      }
      if (
        record.delay_tasks.size >= 64 ||
        record.virtual_time > Number.MAX_SAFE_INTEGER - durationMs
      ) {
        emitDiagnostic(record, 'delayed-task-dropped', 'delay queue capacity or clock range exceeded', false);
        return { cancel: () => undefined };
      }
      let active = true;
      let timer!: DelayTask;
      timer = {
        due: record.virtual_time + durationMs,
        cancel: () => {
          if (!active) return;
          active = false;
          record.delay_tasks.delete(timer);
        },
        run: () => {
          if (!active) return;
          active = false;
          record.delay_tasks.delete(timer);
          task();
        },
      };
      record.delay_tasks.add(timer);
      return timer;
    },
    commit: (children: TemplateChildren, signal?: CommitSignal) => {
      if (!signal) throw new Error('Proto UI host commit requires a completion signal');
      record.pending_commit = {
        commit_id: record.commit_id + 1,
        view_epoch: record.session?.mountEpoch ?? 1,
        signal,
      };
      record.events.push(projection(record, children));
    },
    onRuntimeReady: (wiring: ModuleWiringLike) => attachCapabilities(record, wiring),
  };
}


function startSession(command: Record<string, unknown>): void {
  const sessionId = getRequiredString(command, 'session_id');
  const instanceId = getRequiredString(command, 'instance_id');
  const prototype = getRequiredString(command, 'prototype');
  const definition = registry[prototype];
  if (!definition) {
    throw new Error(`unknown Proto UI prototype: ${prototype}`);
  }
  const props = jsonObject(command.props ?? {});
  const meta = jsonObject(command.meta ?? {});
  const slotObject = recordOf(command.slot);
  if (!slotObject) throw new Error('slot must be a JSON object');
  const slot = {
    slot_id: getRequiredString(slotObject, 'slot_id'),
    accessible_name: getRequiredString(slotObject, 'accessible_name'),
  };
  const routeRef = command.route_ref === undefined || command.route_ref === null
    ? null
    : getRequiredString({ route_ref: command.route_ref }, 'route_ref');
  const parentRef = parseParent(command.parent);
  if (parentRef) validateParent(parentRef);
  if (sessions.has(sessionId)) throw new Error(`duplicate session: ${sessionId}`);
  const record = createRecord(
    sessionId,
    instanceId,
    prototype,
    definition,
    props,
    meta,
    slot,
    routeRef,
    parentRef,
  );
  sessions.set(sessionId, record);
  try {
    const session = createRuntimeSession(definition, runtimeHost(record)) as RuntimeSessionLike;
    record.session = session;
    record.events.unshift({
      type: 'ready',
      handshake: {
        protocol: { major: PROTOCOL_MAJOR, minor: PROTOCOL_MINOR },
        proto_ui: PROTO_UI_VERSION,
        host: { name: HOST_NAME, gpui: GPUI_VERSION, platform: HOST_PLATFORM },
        registry_digest: REGISTRY_DIGEST,
      },
    });
    void session.mount().catch((error: unknown) => {
      emitDiagnostic(record, 'runtime-mount-failed', String(error), true);
    });
  } catch (error) {
    sessions.delete(sessionId);
    throw error;
  }
}

function sessionFor(command: Record<string, unknown>): SessionRecord {
  const sessionId = getRequiredString(command, 'session_id');
  const record = sessions.get(sessionId);
  if (!record) throw new Error(`unknown session: ${sessionId}`);
  const instanceId = getRequiredString(command, 'instance_id');
  if (record.instance_id !== instanceId) throw new Error(`instance mismatch: ${instanceId}`);
  return record;
}

function acknowledge(command: Record<string, unknown>): void {
  const ack = recordOf(command.ack);
  if (!ack) throw new Error('projection ack must be a JSON object');
  const record = sessionFor(ack);
  const pending = record.pending_commit;
  if (!pending) throw new Error('projection ack has no pending commit');
  const epoch = getRequiredNumber(ack, 'view_epoch');
  const commitId = getRequiredNumber(ack, 'commit_id');
  if (pending.view_epoch !== epoch || pending.commit_id !== commitId) {
    throw new Error(`stale projection ack: expected ${pending.view_epoch}/${pending.commit_id}`);
  }
  const status = getRequiredString(ack, 'status');
  if (status === 'applied') {
    record.pending_commit = undefined;
    pending.signal.done();
    for (const listener of [...record.ready_listeners]) listener();
  } else if (status !== 'superseded') {
    throw new Error(`projection rejected: ${status}`);
  }
}

function input(command: Record<string, unknown>): void {
  const inputObject = recordOf(command.input);
  if (!inputObject) throw new Error('input must be a JSON object');
  const record = sessionFor(inputObject);
  const kind = getRequiredString(inputObject, 'kind');
  const routeRef = getRequiredString(inputObject, 'route_ref');
  if (record.route_ref !== null && record.route_ref !== routeRef) {
    throw new Error(`input route mismatch: expected ${record.route_ref}/${routeRef}`);
  }
  if (record.route_ref === null) record.route_ref = routeRef;
  const sampleId = getRequiredString(inputObject, 'sample_id');
  if (kind === 'press_commit') {
    if (record.seen_press_samples.has(sampleId)) return;
    if (record.seen_press_samples.size >= 1024) {
      const oldest = record.seen_press_samples.values().next().value;
      if (typeof oldest === 'string') record.seen_press_samples.delete(oldest);
    }
    record.seen_press_samples.add(sampleId);
  }
  const eventType = kind.replaceAll('_', '.');
  const detailObject = recordOf(command.detail);
  const detail = detailObject ? { ...detailObject } : {};
  if (kind === 'key_down' || kind === 'key_up') {
    detail.preventDefault = () => {
      emitDiagnostic(
        record,
        'default-action-not-applicable',
        'GPUI owns the native default action; no browser cancellation was claimed',
        false,
      );
    };
  }
  const event = { detail };
  if (kind === 'key_down' || kind === 'key_up') {
    familyGlobalBus(record).dispatch(eventType, event);
  } else if (kind === 'focus') {
    focusSurface(record.surface);
  } else if (kind === 'blur') {
    blurSurface(record.surface);
  } else {
    record.root_bus.dispatch(eventType, event);
  }
}

function textControl(command: Record<string, unknown>): void {
  const operation = recordOf(command.command);
  if (!operation) throw new Error('text-control command must be a JSON object');
  const request = { ...command, ...operation };
  const record = sessionFor(request);
  const epoch = getRequiredNumber(request, 'view_epoch');
  const currentEpoch = record.session?.mountEpoch ?? 0;
  if (currentEpoch !== epoch) {
    throw new Error(`stale text-control view epoch: expected ${currentEpoch}/${epoch}`);
  }
  if (operation.kind !== 'event') {
    throw new Error(`unsupported text-control operation: ${String(operation.kind)}`);
  }
  const controlRef = getRequiredString(request, 'control_ref');
  if (controlRef !== record.text_control_ref) {
    throw new Error(`text-control reference mismatch: expected ${record.text_control_ref}/${controlRef}`);
  }
  const state = record.text_control;
  if (!state || state.disposed || !state.connection) {
    throw new Error('text-control host lease is unavailable');
  }
  const eventObject = recordOf(request.event);
  if (!eventObject) throw new Error('text-control event must be a JSON object');
  const eventType = getRequiredString(eventObject, 'type');
  if (!['input', 'change', 'compositionstart', 'compositionupdate', 'compositionend'].includes(eventType)) {
    throw new Error(`unsupported text-control event: ${eventType}`);
  }
  const value = stringValue(eventObject.value);
  if (value === null) throw new Error('text-control event value must be a string');
  const disabled = state.patch.disabled === true;
  const readOnly = state.patch.readOnly === true;
  if (disabled || (readOnly && eventType !== 'change')) return;
  const composing = typeof eventObject.composing === 'boolean'
    ? eventObject.composing
    : eventType === 'compositionstart' || eventType === 'compositionupdate';
  if (eventType === 'compositionstart' || eventType === 'compositionupdate') state.composing = true;
  if (eventType === 'compositionend') state.composing = false;
  state.value = value.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
  const selectionObject = recordOf(request.selection);
  if (selectionObject) {
    const start = numberValue(selectionObject.start);
    const end = numberValue(selectionObject.end);
    if (start === null || end === null || start < 0 || end < 0) {
      throw new Error('text-control selection must contain non-negative safe integers');
    }
    const direction = selectionObject.direction;
    state.selection = {
      start: Math.min(start, state.value.length),
      end: Math.min(end, state.value.length),
      direction: direction === 'forward' || direction === 'backward' ? direction : 'none',
    };
  } else {
    state.selection = {
      ...state.selection,
      start: Math.min(state.selection.start, state.value.length),
      end: Math.min(state.selection.end, state.value.length),
    };
  }
  state.connection.onEvent(Object.freeze({
    type: eventType as TextControlEventType,
    value: state.value,
    composing,
    data: typeof eventObject.data === 'string' ? eventObject.data : null,
    inputType: typeof eventObject.inputType === 'string' ? eventObject.inputType : null,
  }));
  if (eventType === 'compositionend' && state.deferredValue !== null) {
    replaceTextControlValue(state, state.deferredValue);
    state.deferredValue = null;
  }
}

function setProps(command: Record<string, unknown>): void {
  const record = sessionFor(command);
  record.props = jsonObject(command.props ?? {});
  record.session?.controller.applyRawProps(record.props);
  record.session?.controller.update();
}

function remount(command: Record<string, unknown>): void {
  const record = sessionFor(command);
  const session = record.session;
  if (!session) throw new Error('session is not ready to remount');
  record.pending_commit = undefined;
  cancelDelayTasks(record);
  void session.unmount();
  void session.mount().catch((error: unknown) => {
    emitDiagnostic(record, 'runtime-mount-failed', String(error), true);
  });
}

function advanceSessionTime(command: Record<string, unknown>): void {
  const record = sessionFor(command);
  advanceTime(record, getRequiredNumber(command, 'milliseconds'));
}

function unmount(command: Record<string, unknown>): void {
  const record = sessionFor(command);
  const epoch = getRequiredNumber(command, 'view_epoch');
  if (record.session && record.session.mountEpoch !== epoch) {
    throw new Error(`stale unmount request: expected ${record.session.mountEpoch}/${epoch}`);
  }
  cancelDelayTasks(record);
  void record.session?.unmount().catch((error: unknown) => {
    emitDiagnostic(record, 'runtime-unmount-failed', String(error), true);
  });
}

function dispose(command: Record<string, unknown>): void {
  const record = sessionFor(command);
  record.disposed = true;
  cancelDelayTasks(record);
  record.scheduled_tasks.splice(0);
  for (const unsubscribe of record.state_unsubs.splice(0)) unsubscribe();
  void record.session?.dispose().catch((error: unknown) => {
    emitDiagnostic(record, 'runtime-dispose-failed', String(error), true);
  });
  sessions.delete(record.session_id);

}
function registryEvent(): WireEvent {
  return {
    type: 'registry',
    proto_ui: PROTO_UI_VERSION,
    keys: Object.keys(registry).sort(),
  };
}


function dispatch(serialized: string): string {
  if (bridgeFailed) throw new Error('bridge is terminally failed after message overflow');
  const command = recordOf(JSON.parse(serialized));
  if (!command) throw new Error('bridge command must be a JSON object');
  const directEvents: WireEvent[] = [];
  switch (command.type) {
    case 'registry':
      directEvents.push(registryEvent());
      break;
    case 'start':
      startSession(command);
      break;
    case 'projection_ack':
      acknowledge(command);
      break;
    case 'input':
      input(command);
      break;
    case 'text_control':
      textControl(command);
      break;
    case 'advance_time':
      advanceSessionTime(command);
      break;
    case 'set_props':
      setProps(command);
      break;
    case 'remount':
      remount(command);
      break;
    case 'unmount':
      unmount(command);
      break;
    case 'dispose':
      dispose(command);
      break;
    default:
      throw new Error(`unknown bridge command: ${String(command.type)}`);
  }
  flushScheduledTasks();
  const events: WireEvent[] = [...directEvents];
  for (const record of sessions.values()) {
    if (record.events.length === 0) continue;
    events.push(...record.events);
  }
  const output = JSON.stringify(events);
  if (utf8Length(output) > MAX_BRIDGE_MESSAGE_BYTES) {
    bridgeFailed = true;
    return JSON.stringify([
      {
        type: 'diagnostic',
        diagnostic: {
          code: 'message-overflow',
          detail: `bridge response exceeds ${MAX_BRIDGE_MESSAGE_BYTES} bytes`,
          fatal: true,
        },
      },
    ]);
  }
  for (const record of sessions.values()) record.events.splice(0);
  return output;
}

const bridge = { dispatch };
(globalThis as unknown as { __sailbreak_proto_ui_bridge_v1: typeof bridge }).__sailbreak_proto_ui_bridge_v1 = bridge;
