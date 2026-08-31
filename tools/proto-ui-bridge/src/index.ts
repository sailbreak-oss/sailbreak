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

type JsonPrimitive = null | boolean | number | string;
type JsonValue = JsonPrimitive | JsonValue[] | { [key: string]: JsonValue };
type JsonObject = { [key: string]: JsonValue };
type Listener = (event: unknown) => void;
type RuntimeSessionLike = Pick<RuntimeSession, 'mount' | 'unmount' | 'dispose' | 'controller'> & {
  readonly instancePhase: string;
  readonly mountEpoch: number;
  readonly mountPhase: string;
};
type ModuleWiringLike = {
  attach(moduleName: string, entries: CapEntries): boolean;
};
type CommitSignal = { done(): void };
type WireTemplateNode =
  | { kind: 'container'; tag: string; style?: string[]; children?: WireTemplateNode[] }
  | { kind: 'text'; text: string }
  | { kind: 'slot'; slot_id: string };
type WireA11y = {
  role: string;
  name: string;
  disabled: boolean;
  focused: boolean;
  focus_visible: boolean;
  actions?: string[];
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
      type: 'diagnostic';
      diagnostic: { code: string; detail: string; fatal: boolean };
    };

type Surface = {
  focusable: boolean;
  focused: boolean;
  focus(): void;
  blur(): void;
};

type PendingCommit = {
  commit_id: number;
  view_epoch: number;
  signal: CommitSignal;
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
  props: JsonObject;
  slot: { slot_id: string; accessible_name: string };
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
  a11y: WireA11y | null;
  state_values: JsonObject;
  exposed_handles: Record<string, unknown>;
  state_unsubs: Array<() => void>;
  disposed: boolean;
};

type Registry = Record<string, Prototype>;

const registry: Registry = {
  'shadcn-button': shadcn.shadcnButton,
  'shadcn-toggle': shadcn.shadcnToggle,
  'shadcn-switch-root': shadcn.shadcnSwitchRoot,
  'shadcn-switch-thumb': shadcn.shadcnSwitchThumb,
  'shadcn-tabs-root': shadcn.shadcnTabsRoot,
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
  const nameObject = object ? recordOf(object.name) : null;
  const name = nameObject?.kind === 'content'
    ? record.slot.accessible_name
    : stringValue(nameObject?.value) ?? stringValue(object?.name) ?? record.slot.accessible_name;
  const actionsObject = object ? recordOf(object.actions) : null;
  const actions = actionsObject ? Object.keys(actionsObject) : [];
  return {
    role: stringValue(object?.role) ?? 'generic',
    name,
    disabled: booleanValue(states?.disabled) ?? false,
    focused: booleanValue(states?.focused) ?? false,
    focus_visible: booleanValue(states?.focusVisible) ?? false,
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

function emitState(record: SessionRecord, value: unknown): void {
  if (record.disposed) return;
  for (const unsubscribe of record.state_unsubs.splice(0)) unsubscribe();
  record.exposed_handles = recordOf(value) ?? {};
  record.state_values = exposedStates(record.exposed_handles);
  record.events.push({
    type: 'state',
    session_id: record.session_id,
    instance_id: record.instance_id,
    view_epoch: record.session?.mountEpoch ?? 1,
    values: { ...record.state_values },
  });
  for (const candidate of Object.values(record.exposed_handles)) {
    const candidateObject = recordOf(candidate);
    const subscribe = candidateObject?.subscribe;
    if (typeof subscribe !== 'function') continue;
    const unsubscribe = subscribe.call(candidate, () => emitState(record, record.exposed_handles));
    if (typeof unsubscribe === 'function') record.state_unsubs.push(unsubscribe);
  }
}

function attachCapabilities(record: SessionRecord, wiring: ModuleWiringLike): void {
  wiring.attach('as-trigger', [
    [AS_TRIGGER_INSTANCE_CAP, record.surface],
    [AS_TRIGGER_PARENT_CAP, () => null],
    [AS_TRIGGER_GET_PROTO_CAP, () => null],
    [AS_TRIGGER_MERGE_GROUP_CAP, () => undefined],
    [AS_TRIGGER_GET_GROUP_EVENT_TARGET_CAP, () => record.root_bus],
  ]);
  wiring.attach('event', [
    [EVENT_ROOT_TARGET_CAP, () => record.root_bus],
    [EVENT_GLOBAL_TARGET_CAP, () => record.global_bus],
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
    [EXPOSE_EVENT_SINK_CAP, (key: string) => {
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
  wiring.attach('focus', [
    [FOCUS_ROOT_TARGET_CAP, () => record.surface],
    [FOCUS_TARGET_READY_CAP, (listener: () => void) => {
      record.ready_listeners.add(listener);
      return () => record.ready_listeners.delete(listener);
    }],
    [FOCUS_INSTANCE_TOKEN_CAP, record.surface],
    [FOCUS_PARENT_CAP, () => null],
    [FOCUS_IS_NATIVELY_FOCUSABLE_CAP, () => record.surface.focusable],
    [FOCUS_SET_FOCUSABLE_CAP, (target: Surface, enabled: boolean) => {
      target.focusable = enabled;
    }],
    [FOCUS_REQUEST_FOCUS_CAP, (target: Surface) => {
      if (!target.focusable) return false;
      target.focus();
      return true;
    }],
    [FOCUS_BLUR_CAP, (target: Surface) => target.blur()],
    [FOCUS_RUN_IN_CALLBACK_CAP, (callback: () => void) => callback()],
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
    [EXPOSE_STATE_SET_EXPOSES_CAP, (exposes: unknown) => emitState(record, exposes)],
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
  props: JsonObject,
  slot: { slot_id: string; accessible_name: string },
): SessionRecord {
  const surface: Surface = {
    focusable: false,
    focused: false,
    focus() {
      surface.focused = true;
    },
    blur() {
      surface.focused = false;
    },
  };
  return {
    session_id: sessionId,
    instance_id: instanceId,
    prototype,
    props,
    slot,
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
    a11y: null,
    state_values: {},
    exposed_handles: {},
    state_unsubs: [],
    disposed: false,
  };
}

function runtimeHost(record: SessionRecord) {
  return {
    prototypeName: record.prototype,
    getRawProps: () => record.props,
    schedule: (task: () => void) => task(),
    scheduleDelay: (durationMs: number, task: () => void) => {
      let active = true;
      const timer = { cancel: () => { active = false; } };
      if (durationMs <= 0 && active) task();
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
  const slotObject = recordOf(command.slot);
  if (!slotObject) throw new Error('slot must be a JSON object');
  const slot = {
    slot_id: getRequiredString(slotObject, 'slot_id'),
    accessible_name: getRequiredString(slotObject, 'accessible_name'),
  };
  if (sessions.has(sessionId)) throw new Error(`duplicate session: ${sessionId}`);
  const record = createRecord(sessionId, instanceId, prototype, props, slot);
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
  const sampleId = getRequiredString(inputObject, 'sample_id');
  if (kind === 'press_commit') {
    if (record.seen_press_samples.has(sampleId)) return;
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
    record.global_bus.dispatch(eventType, event);
  } else if (kind === 'focus') {
    record.root_bus.dispatch('host:focus', event);
  } else if (kind === 'blur') {
    record.root_bus.dispatch('host:blur', event);
  } else {
    record.root_bus.dispatch(eventType, event);
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
  void session.unmount();
  void session.mount().catch((error: unknown) => {
    emitDiagnostic(record, 'runtime-mount-failed', String(error), true);
  });
}

function unmount(command: Record<string, unknown>): void {
  const record = sessionFor(command);
  void record.session?.unmount().catch((error: unknown) => {
    emitDiagnostic(record, 'runtime-unmount-failed', String(error), true);
  });
}

function dispose(command: Record<string, unknown>): void {
  const record = sessionFor(command);
  record.disposed = true;
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
  const events: WireEvent[] = directEvents;
  for (const record of sessions.values()) {
    if (record.events.length === 0) continue;
    events.push(...record.events.splice(0));
  }
  return JSON.stringify(events);
}

const bridge = { dispatch };
(globalThis as unknown as { __sailbreak_proto_ui_bridge_v1: typeof bridge }).__sailbreak_proto_ui_bridge_v1 = bridge;
