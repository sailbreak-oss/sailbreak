(() => {
  var __create = Object.create;
  var __getProtoOf = Object.getPrototypeOf;
  var __defProp = Object.defineProperty;
  var __getOwnPropNames = Object.getOwnPropertyNames;
  var __hasOwnProp = Object.prototype.hasOwnProperty;
  function __accessProp(key) {
    return this[key];
  }
  var __toESMCache_node;
  var __toESMCache_esm;
  var __toESM = (mod, isNodeMode, target) => {
    var canCache = mod != null && typeof mod === "object";
    if (canCache) {
      var cache = isNodeMode ? __toESMCache_node ??= new WeakMap : __toESMCache_esm ??= new WeakMap;
      var cached = cache.get(mod);
      if (cached)
        return cached;
    }
    target = mod != null ? __create(__getProtoOf(mod)) : {};
    const to = isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target;
    if (mod && typeof mod === "object" || typeof mod === "function") {
      for (let key of __getOwnPropNames(mod))
        if (!__hasOwnProp.call(to, key))
          __defProp(to, key, {
            get: __accessProp.bind(mod, key),
            enumerable: true
          });
    }
    if (canCache)
      cache.set(mod, to);
    return to;
  };
  var __require = /* @__PURE__ */ ((x) => typeof require !== "undefined" ? require : typeof Proxy !== "undefined" ? new Proxy(x, {
    get: (a, b) => (typeof require !== "undefined" ? require : a)[b]
  }) : x)(function(x) {
    if (typeof require !== "undefined")
      return require.apply(this, arguments);
    throw Error('Dynamic require of "' + x + '" is not supported');
  });
  // ../packages/core/src/internal.ts
  var activeAsHookContexts = [];
  var activeRuntimeDelayContexts = [];
  var asHookRuntimeByDef = new WeakMap;
  function enterActiveAsHookContext(ctx) {
    activeAsHookContexts.push(ctx);
  }
  function exitActiveAsHookContext() {
    activeAsHookContexts.pop();
  }
  function getActiveAsHookContext(name) {
    const ctx = activeAsHookContexts.at(-1);
    if (!ctx) {
      throw new Error(`[AsHook] no active setup context for ${name}.`);
    }
    return ctx;
  }
  function enterActiveRuntimeDelayContext(ctx) {
    activeRuntimeDelayContexts.push(ctx);
  }
  function exitActiveRuntimeDelayContext() {
    activeRuntimeDelayContexts.pop();
  }
  function getActiveRuntimeDelayContext() {
    return activeRuntimeDelayContexts.at(-1);
  }
  function bindAsHookRuntime(def, runtime) {
    asHookRuntimeByDef.set(def, runtime);
  }
  function getAsHookRuntime(def) {
    return asHookRuntimeByDef.get(def);
  }

  // ../packages/core/src/prototype.ts
  var MODULE_DECLARATION_TOKEN_BRAND = Symbol("@proto.ui/module-declaration-token");
  function moduleDeclaration(id) {
    if (typeof id !== "string" || id.trim().length === 0) {
      throw new Error(`[Prototype] module declaration token id must be a non-empty string.`);
    }
    return Object.freeze({
      id,
      [MODULE_DECLARATION_TOKEN_BRAND]: true
    });
  }
  function declareModule(token, config) {
    if (token?.[MODULE_DECLARATION_TOKEN_BRAND] !== true) {
      throw new Error(`[Prototype] declareModule() expects a ModuleDeclarationToken.`);
    }
    const frozenConfig = config !== null && typeof config === "object" ? Object.freeze(config) : config;
    return Object.freeze({ id: token.id, token, config: frozenConfig });
  }
  function normalizeAsHookRender(value) {
    if (typeof value !== "undefined" && typeof value !== "function") {
      throw new Error(`[AsHook] setup() must return render function or void, got: ${typeof value}.`);
    }
    return typeof value === "function" ? value : undefined;
  }
  function getModuleDeclaration(proto, token) {
    return proto.modules?.find((declaration) => declaration.id === token.id);
  }
  function definePrototype(proto) {
    if (!proto || typeof proto !== "object") {
      throw new Error(`[Prototype] definePrototype() expects an object.`);
    }
    if (!proto.name || typeof proto.name !== "string") {
      throw new Error(`[Prototype] illegal name.`);
    }
    if (typeof proto.setup !== "function") {
      throw new Error(`[Prototype] setup must be a function.`);
    }
    proto.modules = freezeModuleDeclarations(proto.modules, "Prototype");
    return proto;
  }
  function freezeModuleDeclarations(declarations, owner) {
    const values = declarations ?? [];
    const ids = new Set;
    for (const declaration of values) {
      if (ids.has(declaration.id)) {
        throw new Error(`[${owner}] duplicate module declaration id: ${declaration.id}`);
      }
      ids.add(declaration.id);
    }
    return Object.freeze(values.slice());
  }
  function defineAsHook(proto) {
    return createHookCaller(proto, "asHook");
  }
  function createHookCaller(proto, kind) {
    if (!proto || typeof proto !== "object") {
      throw new Error(`[${kind === "hook" ? "Hook" : "AsHook"}] define expects an object.`);
    }
    if (!proto.name || typeof proto.name !== "string") {
      throw new Error(`[${kind === "hook" ? "Hook" : "AsHook"}] illegal name.`);
    }
    if (typeof proto.setup !== "function") {
      throw new Error(`[${kind === "hook" ? "Hook" : "AsHook"}] setup must be a function.`);
    }
    const staticModules = kind === "asHook" ? freezeModuleDeclarations(proto.modules, "AsHook") : Object.freeze([]);
    if (kind === "asHook") {
      Object.defineProperty(proto, "modules", {
        value: staticModules,
        enumerable: true,
        configurable: false,
        writable: false
      });
    }
    const caller = (options) => {
      const { def: activeDef, rt } = getActiveAsHookContext(proto.name);
      const def = activeDef;
      rt.ensureSetup(`asHook(${proto.name})`);
      const mode = kind === "hook" ? proto.mode : "once";
      const reg = rt.register(proto.name, {
        privileged: false,
        mode: mode ?? "once"
      });
      const api = {
        name: proto.name,
        order: reg.order,
        store: reg.state.store
      };
      const tools = {
        warn(message) {
          if (typeof console !== "undefined" && typeof console.warn === "function") {
            console.warn(`[AsHook:${proto.name}] ${message}`);
          }
        },
        conflict(message) {
          throw new Error(`[AsHook:${proto.name}] ${message}`);
        }
      };
      if (reg.action === "skip") {
        const result2 = reg.state.result ?? {};
        const handle = kind === "asHook" && Object.hasOwn(reg.state, "callerResult") ? reg.state.callerResult : result2;
        rt.recordAsHookResult({
          name: proto.name,
          order: reg.order,
          privileged: false,
          mode: mode ?? "once",
          result: result2,
          handle
        });
        return handle;
      }
      if (reg.action === "setup") {
        rt.beginCapture(proto.name, {
          order: reg.order,
          privileged: false,
          mode: mode ?? "once"
        });
        let captureOpen = true;
        try {
          const render = normalizeAsHookRender(kind === "hook" ? proto.setup(def, options, api) : proto.setup(def));
          const result2 = rt.endCapture(render);
          captureOpen = false;
          let finalResult = result2;
          if (result2 && typeof result2 === "object" && "state" in result2) {
            const nextState = rt.projectState(result2.state);
            if (result2.state !== nextState) {
              finalResult = { ...result2, state: nextState };
            }
          }
          reg.state.result = finalResult;
          if (kind === "asHook") {
            const projectHandle = proto.projectHandle;
            reg.state.callerResult = projectHandle ? projectHandle(finalResult) : finalResult;
          }
          rt.recordAsHookResult({
            name: proto.name,
            order: reg.order,
            privileged: false,
            mode: mode ?? "once",
            result: finalResult,
            handle: kind === "asHook" ? reg.state.callerResult : finalResult
          });
          const hookProto2 = proto;
          if (kind === "hook" && (hookProto2.mode ?? "once") === "configurable" && typeof hookProto2.configure === "function") {
            hookProto2.configure(api, options, tools);
          }
          return kind === "asHook" ? reg.state.callerResult : reg.state.result ?? {};
        } catch (e) {
          if (captureOpen) {
            rt.abortCapture();
          }
          throw e;
        }
      }
      const hookProto = proto;
      if (kind === "hook" && typeof hookProto.configure === "function") {
        hookProto.configure(api, options, tools);
      }
      const result = reg.state.result ?? {};
      rt.recordAsHookResult({
        name: proto.name,
        order: reg.order,
        privileged: false,
        mode: mode ?? "once",
        result,
        handle: result
      });
      return result;
    };
    Object.defineProperty(caller, "kind", {
      value: kind,
      enumerable: false,
      configurable: false,
      writable: false
    });
    Object.defineProperty(caller, "definition", {
      value: proto,
      enumerable: false,
      configurable: false,
      writable: false
    });
    if (kind === "asHook") {
      Object.defineProperty(caller, "modules", {
        value: staticModules,
        enumerable: false,
        configurable: false,
        writable: false
      });
    }
    return caller;
  }
  function defineHook(proto) {
    return createHookCaller(proto, "hook");
  }
  // ../packages/core/src/anatomy.ts
  function freezeFamilyDecl(decl) {
    if (!decl || typeof decl !== "object") {
      throw new Error(`[Anatomy] family declaration is required.`);
    }
    if (!decl.roles || typeof decl.roles !== "object") {
      throw new Error(`[Anatomy] family declaration must define roles.`);
    }
    if (!decl.roles.root) {
      throw new Error(`[Anatomy] family declaration must define a root role.`);
    }
    const roles = Object.freeze(Object.fromEntries(Object.entries(decl.roles ?? {}).map(([role, roleDecl]) => [
      role,
      Object.freeze({
        ...roleDecl,
        cardinality: Object.freeze({ ...roleDecl.cardinality }),
        requires: roleDecl.requires ? Object.freeze([...roleDecl.requires]) : undefined
      })
    ])));
    const profiles = decl.profiles ? Object.freeze(Object.fromEntries(Object.entries(decl.profiles).map(([name, profileDecl]) => [
      name,
      Object.freeze({
        ...profileDecl,
        roles: profileDecl.roles ? Object.freeze(Object.fromEntries(Object.entries(profileDecl.roles).map(([role, roleDecl]) => [
          role,
          Object.freeze({
            ...roleDecl,
            cardinality: roleDecl.cardinality ? Object.freeze({ ...roleDecl.cardinality }) : undefined,
            requires: roleDecl.requires ? Object.freeze([...roleDecl.requires]) : undefined
          })
        ]))) : undefined,
        relations: profileDecl.relations ? Object.freeze([...profileDecl.relations]) : undefined
      })
    ]))) : undefined;
    return Object.freeze({
      roles,
      relations: decl.relations ? Object.freeze([...decl.relations]) : undefined,
      profiles
    });
  }
  function createAnatomyFamily(debugName, decl) {
    if (typeof debugName !== "string" || debugName.length === 0) {
      throw new Error(`[Anatomy] debugName must be a non-empty string.`);
    }
    return Object.freeze({
      __brand: "AnatomyFamily",
      debugName,
      decl: freezeFamilyDecl(decl)
    });
  }
  // ../packages/core/src/spec/feedback/semantic-merge.ts
  var PREFIXES_V0 = [
    "bg-",
    "p-",
    "px-",
    "py-",
    "pt-",
    "pr-",
    "pb-",
    "pl-",
    "m-",
    "mx-",
    "my-",
    "mt-",
    "mr-",
    "mb-",
    "ml-",
    "w-",
    "h-",
    "min-w-",
    "min-h-",
    "max-w-",
    "max-h-",
    "justify-",
    "items-",
    "content-",
    "opacity-",
    "shadow-",
    "rounded"
  ];
  var TEXT_SIZE_TOKENS = new Set([
    "xs",
    "sm",
    "base",
    "lg",
    "xl",
    "2xl",
    "3xl",
    "4xl",
    "5xl",
    "6xl",
    "7xl",
    "8xl",
    "9xl"
  ]);
  function getSemanticGroupKeyV0(token) {
    if (token === "flex" || token === "inline-flex" || token === "block" || token === "inline-block") {
      return "display";
    }
    if (token === "flex-row" || token === "flex-row-reverse" || token === "flex-col" || token === "flex-col-reverse") {
      return "flex-direction";
    }
    if (token === "border" || token === "border-0" || token === "border-2" || token === "border-4" || token === "border-8") {
      return "border-width";
    }
    const directionalBorderWidth = token.match(/^border-([trblxy])(?:-(0|2|4|8))?$/);
    if (directionalBorderWidth) {
      return `border-${directionalBorderWidth[1]}-width`;
    }
    if (token.startsWith("border-")) {
      return "border-color";
    }
    if (token.startsWith("bg-clip-"))
      return "bg-clip-";
    if (token.startsWith("text-")) {
      const suffix = token.slice("text-".length);
      if (TEXT_SIZE_TOKENS.has(suffix) || suffix.startsWith("["))
        return "text-size";
      return "text-color";
    }
    for (const p of PREFIXES_V0) {
      if (token.startsWith(p))
        return p;
    }
    return token;
  }
  function mergeTwTokensV0(tokens) {
    const groupOrder = [];
    const seen = new Set;
    const lastByGroup = new Map;
    for (const t of tokens) {
      const g = getSemanticGroupKeyV0(t);
      if (!seen.has(g)) {
        seen.add(g);
        groupOrder.push(g);
      }
      lastByGroup.set(g, t);
    }
    const out = [];
    for (const g of groupOrder) {
      const v = lastByGroup.get(g);
      if (v)
        out.push(v);
    }
    return { tokens: out };
  }

  // ../packages/core/src/spec/feedback/tokens.ts
  function assertTwTokenV0(token, ctx) {
    const where = ctx ? ` (${ctx})` : "";
    if (typeof token !== "string" || !token.trim()) {
      throw new Error(`[feedback] invalid tw token${where}: empty`);
    }
    if (/\s/.test(token)) {
      throw new Error(`[feedback] invalid tw token${where}: contains whitespace: "${token}"`);
    }
    if (token.startsWith(".") || token.startsWith("#")) {
      throw new Error(`[feedback] invalid tw token${where}: selector-like token is forbidden in "${token}"`);
    }
    if (token.includes(":")) {
      throw new Error(`[feedback] invalid tw token${where}: forbidden character ":" in "${token}"`);
    }
    const left = token.indexOf("[");
    const right = token.lastIndexOf("]");
    if (left !== -1 || right !== -1) {
      if (!(left !== -1 && right !== -1 && right > left)) {
        throw new Error(`[feedback] invalid tw token${where}: malformed bracket in "${token}"`);
      }
      const inside = token.slice(left + 1, right);
      if (!inside.length) {
        throw new Error(`[feedback] invalid tw token${where}: empty bracket value in "${token}"`);
      }
      if (/[\s]/.test(inside)) {
        throw new Error(`[feedback] invalid tw token${where}: bracket value contains whitespace in "${token}"`);
      }
      if (inside.includes(":")) {
        throw new Error(`[feedback] invalid tw token${where}: bracket value contains ":" in "${token}"`);
      }
    }
  }

  // ../packages/core/src/spec/feedback/recorder.ts
  class FeedbackStyleRecorder {
    nextId = 1;
    chunks = [];
    runtimePatch = new Map;
    use(...handles) {
      const flattened = [];
      for (const h of handles) {
        if (!h || h.kind !== "tw" || !Array.isArray(h.tokens)) {
          throw new Error(`[feedback] unsupported style handle in v0`);
        }
        for (const t of h.tokens) {
          assertTwTokenV0(t, "feedback.style.use");
          flattened.push(t);
        }
      }
      const chunk = {
        id: this.nextId++,
        tokens: flattened,
        removed: false
      };
      this.chunks.push(chunk);
      const unUse = () => {
        chunk.removed = true;
      };
      return unUse;
    }
    useUnsafe(...handles) {
      const flattened = [];
      for (const h of handles) {
        if (!h || h.kind !== "tw" || !Array.isArray(h.tokens)) {
          throw new Error(`[feedback] unsupported style handle in v0`);
        }
        for (const t of h.tokens) {
          if (typeof t !== "string" || !t) {
            throw new Error(`[feedback] invalid tw token (unsafe): empty`);
          }
          flattened.push(t);
        }
      }
      const chunk = {
        id: this.nextId++,
        tokens: flattened,
        removed: false
      };
      this.chunks.push(chunk);
      const unUse = () => {
        chunk.removed = true;
      };
      return unUse;
    }
    patch(...handles) {
      for (const token of this.flattenRuntimePatchHandles(handles, "run.feedback.style.patch")) {
        this.runtimePatch.set(getSemanticGroupKeyV0(token), { kind: "patch", token });
      }
    }
    suppress(...handles) {
      for (const token of this.flattenRuntimePatchHandles(handles, "run.feedback.style.suppress")) {
        this.runtimePatch.set(getSemanticGroupKeyV0(token), { kind: "suppress" });
      }
    }
    clearPatch() {
      this.runtimePatch.clear();
    }
    export() {
      return this.exportWithAdditional();
    }
    exportWithAdditional(...handles) {
      return this.applyPatchLayer(this.exportBaseTokens(handles));
    }
    exportBase() {
      return { tokens: this.exportBaseTokens() };
    }
    exportBaseTokens(additionalHandles = []) {
      const inputs = [];
      for (const c of this.chunks) {
        if (c.removed)
          continue;
        inputs.push(...c.tokens);
      }
      for (const h of additionalHandles) {
        if (!h || h.kind !== "tw" || !Array.isArray(h.tokens)) {
          throw new Error(`[feedback] unsupported style handle in v0`);
        }
        inputs.push(...h.tokens);
      }
      return mergeTwTokensV0(inputs).tokens;
    }
    applyPatchLayer(baseTokens) {
      if (this.runtimePatch.size === 0)
        return { tokens: baseTokens };
      const patchTokens = [];
      const baseAfterSuppress = [];
      for (const token of baseTokens) {
        const entry = this.runtimePatch.get(getSemanticGroupKeyV0(token));
        if (entry)
          continue;
        baseAfterSuppress.push(token);
      }
      for (const entry of this.runtimePatch.values()) {
        if (entry.kind === "patch")
          patchTokens.push(entry.token);
      }
      return mergeTwTokensV0([...baseAfterSuppress, ...patchTokens]);
    }
    flattenRuntimePatchHandles(handles, op) {
      const flattened = [];
      for (const h of handles) {
        if (!h || h.kind !== "tw" || !Array.isArray(h.tokens)) {
          throw new Error(`[feedback] unsupported style handle in v0`);
        }
        for (const t of h.tokens) {
          assertTwTokenV0(t, op);
          if (t === "data-pui-style") {
            throw new Error(`[feedback] invalid tw token (${op}): host style artifact is forbidden`);
          }
          flattened.push(t);
        }
      }
      return flattened;
    }
  }
  // ../packages/core/src/spec/feedback/style.ts
  function tw(tokens, ...more) {
    const all = [tokens, ...more].join(" ").trim();
    const list = all ? all.split(/\s+/g) : [];
    return { kind: "tw", tokens: list };
  }
  function isTemplateStyleHandle(v) {
    return v && typeof v === "object" && v.kind === "tw" && Array.isArray(v.tokens) && v.tokens.every((x) => typeof x === "string");
  }
  // ../packages/core/src/spec/template.ts
  var DEFAULT_NORMALIZE = {
    flatten: "deep",
    keepNull: false
  };
  function normalizeChildren(input, opt = DEFAULT_NORMALIZE) {
    const cfg = { ...DEFAULT_NORMALIZE, ...opt };
    if (input === undefined)
      return null;
    const out = [];
    const push = (v) => {
      if (typeof v === "boolean") {
        throw new Error(`[Template] boolean child is illegal. Use null for empty, or omit the child.`);
      }
      if (v === undefined) {
        throw new Error(`[Template] undefined child is illegal. Use null for empty, or omit the child.`);
      }
      if (v === null) {
        if (cfg.keepNull)
          out.push(null);
        return;
      }
      if (typeof v === "string" || typeof v === "number") {
        out.push(v);
        return;
      }
      out.push(v);
    };
    const walk = (v, depth) => {
      if (!Array.isArray(v)) {
        push(v);
        return;
      }
      if (cfg.flatten === "none") {
        throw new Error(`[Template] array children is not allowed when flatten=none.`);
      }
      if (cfg.flatten === "shallow" && depth >= 1) {
        throw new Error(`[Template] nested array children is not allowed when flatten=shallow.`);
      }
      for (const x of v)
        walk(x, depth + 1);
    };
    walk(input, 0);
    if (out.length === 0)
      return null;
    if (out.length === 1)
      return out[0];
    return out;
  }
  function isTemplateProps(v) {
    if (!v || typeof v !== "object")
      return false;
    const keys = Object.keys(v);
    if (keys.length === 0)
      return true;
    if (keys.length === 1 && keys[0] === "style")
      return true;
    return false;
  }
  function assertTemplateProps(v) {
    if (!isTemplateProps(v)) {
      throw new Error(`[Template] illegal template-props: only { style?: TemplateStyleHandle } is allowed.
 illegal template-props value: ${JSON.stringify(v)}`);
    }
    if (v?.style && !isTemplateStyleHandle(v.style)) {
      throw new Error(`[Template] style must be a TemplateStyleHandle.`);
    }
  }
  var SVG_ALLOWED_KEYS = {
    svg: [
      "viewBox",
      "width",
      "height",
      "aria-hidden",
      "fill",
      "stroke",
      "strokeWidth",
      "strokeLinecap",
      "strokeLinejoin",
      "fillRule",
      "clipRule",
      "opacity"
    ],
    g: [
      "fill",
      "stroke",
      "strokeWidth",
      "strokeLinecap",
      "strokeLinejoin",
      "fillRule",
      "clipRule",
      "opacity"
    ],
    path: [
      "d",
      "fill",
      "stroke",
      "strokeWidth",
      "strokeLinecap",
      "strokeLinejoin",
      "fillRule",
      "clipRule",
      "opacity"
    ],
    circle: [
      "cx",
      "cy",
      "r",
      "fill",
      "stroke",
      "strokeWidth",
      "strokeLinecap",
      "strokeLinejoin",
      "fillRule",
      "clipRule",
      "opacity"
    ],
    rect: [
      "x",
      "y",
      "width",
      "height",
      "rx",
      "ry",
      "fill",
      "stroke",
      "strokeWidth",
      "strokeLinecap",
      "strokeLinejoin",
      "fillRule",
      "clipRule",
      "opacity"
    ],
    line: [
      "x1",
      "y1",
      "x2",
      "y2",
      "fill",
      "stroke",
      "strokeWidth",
      "strokeLinecap",
      "strokeLinejoin",
      "fillRule",
      "clipRule",
      "opacity"
    ],
    polyline: [
      "points",
      "fill",
      "stroke",
      "strokeWidth",
      "strokeLinecap",
      "strokeLinejoin",
      "fillRule",
      "clipRule",
      "opacity"
    ],
    ellipse: [
      "cx",
      "cy",
      "rx",
      "ry",
      "fill",
      "stroke",
      "strokeWidth",
      "strokeLinecap",
      "strokeLinejoin",
      "fillRule",
      "clipRule",
      "opacity"
    ],
    polygon: [
      "points",
      "fill",
      "stroke",
      "strokeWidth",
      "strokeLinecap",
      "strokeLinejoin",
      "fillRule",
      "clipRule",
      "opacity"
    ]
  };
  var SVG_REQUIRED_KEYS = {
    svg: ["viewBox"],
    path: ["d"],
    circle: ["cx", "cy", "r"],
    rect: ["width", "height"],
    line: ["x1", "y1", "x2", "y2"],
    polyline: ["points"],
    ellipse: ["cx", "cy", "rx", "ry"],
    polygon: ["points"]
  };
  function assertSvgProps(tag, props) {
    if (!props || typeof props !== "object" || Array.isArray(props)) {
      throw new Error(`[Template][SVG] ${tag} props must be an object.`);
    }
    const allowed = new Set(SVG_ALLOWED_KEYS[tag]);
    const keys = Object.keys(props);
    for (const key of keys) {
      if (!allowed.has(key)) {
        throw new Error(`[Template][SVG] ${tag} does not support prop '${key}'.`);
      }
    }
    const required = SVG_REQUIRED_KEYS[tag] ?? [];
    for (const key of required) {
      const value = props[key];
      if (value === undefined || value === null || value === "") {
        throw new Error(`[Template][SVG] ${tag} requires non-empty prop '${key}'.`);
      }
    }
  }
  function createSvgNode(tag, props, children) {
    assertSvgProps(tag, props);
    return {
      kind: "svg-node",
      tag,
      props: Object.freeze({ ...props }),
      children
    };
  }
  function createRendererPrimitives(opt = {}) {
    const normOpt = opt.normalize ?? DEFAULT_NORMALIZE;
    const el = function(type, a, b) {
      let props;
      let childrenInput = null;
      if (arguments.length === 1) {
        childrenInput = null;
      } else if (arguments.length === 2) {
        if (isTemplateProps(a)) {
          assertTemplateProps(a);
          props = a;
          childrenInput = null;
        } else {
          childrenInput = a;
        }
      } else {
        assertTemplateProps(a);
        props = a;
        childrenInput = b;
      }
      return {
        type,
        style: props?.style,
        children: normalizeChildren(childrenInput, normOpt)
      };
    };
    const r = {
      slot() {
        if (arguments.length > 0) {
          const args = Array.from(arguments);
          throw new Error(`[Template] slot() takes no arguments.
 illegal slot arguments: ${JSON.stringify(args)}`);
        }
        return el({ kind: "slot" });
      }
    };
    const slot = r.slot;
    const svg = {
      root(props, children = null) {
        return createSvgNode("svg", props, normalizeChildren(children, normOpt));
      },
      g(props = {}, children = null) {
        return createSvgNode("g", props, normalizeChildren(children, normOpt));
      },
      path(props) {
        return createSvgNode("path", props, null);
      },
      circle(props) {
        return createSvgNode("circle", props, null);
      },
      rect(props) {
        return createSvgNode("rect", props, null);
      },
      line(props) {
        return createSvgNode("line", props, null);
      },
      polyline(props) {
        return createSvgNode("polyline", props, null);
      },
      ellipse(props) {
        return createSvgNode("ellipse", props, null);
      },
      polygon(props) {
        return createSvgNode("polygon", props, null);
      }
    };
    return { el, slot, r, svg };
  }
  // ../packages/core/src/errors/codes.ts
  class ProtoUiError extends Error {
    code;
    details;
    constructor(code, message, details) {
      super(message);
      this.code = code;
      this.details = details;
    }
  }
  function capUnavailable(cap, details) {
    return new ProtoUiError("E_CAP_UNAVAILABLE", `[Caps] capability unavailable: ${cap}`, details);
  }
  function illegalPhase(op, phase, details) {
    return new ProtoUiError("E_ILLEGAL_PHASE", `[Phase] illegal phase for ${op}: ${phase}`, details);
  }
  // ../packages/core/src/context.ts
  function createContextKey(debugName) {
    return Object.freeze({
      __brand: "ContextKey",
      debugName
    });
  }
  // ../packages/core/src/delay.ts
  function delay(durationMs, callback) {
    if (!Number.isFinite(durationMs) || durationMs < 0) {
      throw new Error("[Delay] durationMs must be a non-negative finite number.");
    }
    if (typeof callback !== "function") {
      throw new Error("[Delay] callback must be a function.");
    }
    const ctx = getActiveRuntimeDelayContext();
    if (!ctx) {
      throw new Error("[Delay] delay() is runtime-only and requires an active runtime callback context.");
    }
    return ctx.scheduleDelay(durationMs, callback);
  }
  // ../packages/core/src/text-control.ts
  function canonicalizeLineEndings(value) {
    return value.replace(/\r\n/g, `
`).replace(/\r/g, `
`);
  }
  function canonicalizeTextControlValue(value, lineMode) {
    const normalized = canonicalizeLineEndings(value);
    return lineMode === "single" ? normalized.replace(/\n/g, "") : normalized;
  }
  // ../packages/core/src/caps/token.ts
  function cap(id) {
    return { id };
  }
  // ../packages/core/src/caps/host.ts
  var HOST_ELEMENT_CAP = cap("@proto.ui/web/hostElement");
  // ../packages/runtime/src/kernel/guard.ts
  function illegalPhase2(op, prototypeName, phase, hint) {
    const msg = `[ProtoUI] illegal call: ${op}
` + `prototype: ${prototypeName}
` + `phase: ${phase}
` + (hint ? `hint: ${hint}
` : "");
    throw new Error(msg);
  }
  // ../packages/modules/base/src/caps-vault/vault.ts
  class CapsVault {
    base = new Map;
    attached = new Map;
    listeners = new Set;
    epoch = 0;
    has(token) {
      const id = token.id;
      return this.attached.has(id) || this.base.has(id);
    }
    get(token) {
      const id = token.id;
      if (this.attached.has(id))
        return this.attached.get(id);
      if (this.base.has(id))
        return this.base.get(id);
      throw capUnavailable(id, { epoch: this.epoch });
    }
    onChange(cb) {
      this.listeners.add(cb);
      return () => this.listeners.delete(cb);
    }
    attachBase(entries) {
      if (!entries || entries.length === 0)
        return;
      let changed = false;
      for (const [token, value] of entries) {
        const id = token.id;
        const prev = this.base.get(id);
        if (!this.base.has(id) || prev !== value) {
          this.base.set(id, value);
          changed = true;
        }
      }
      if (changed)
        this.bump();
    }
    attach(entries) {
      if (!entries || entries.length === 0)
        return;
      let changed = false;
      for (const [token, value] of entries) {
        const id = token.id;
        const prev = this.attached.get(id);
        if (!this.attached.has(id) || prev !== value) {
          this.attached.set(id, value);
          changed = true;
        }
      }
      if (changed)
        this.bump();
    }
    resetAttached() {
      if (this.attached.size === 0)
        return;
      this.attached.clear();
      this.bump();
    }
    resetAll() {
      if (this.attached.size === 0 && this.base.size === 0)
        return;
      this.attached.clear();
      this.base.clear();
      this.bump();
    }
    bump() {
      this.epoch++;
      for (const cb of this.listeners)
        cb(this.epoch);
    }
  }
  // ../packages/modules/base/src/system-caps.ts
  var SYS_CAP = cap("@proto.ui/__sys");

  // ../packages/modules/base/src/module-base.ts
  class ModuleBase {
    protoPhase = "setup";
    instancePhase = "setup";
    mountPhase = "detached";
    caps;
    pending = [];
    constructor(caps2) {
      this.caps = caps2;
      this.caps.onChange((epoch) => {
        this.onCapsEpoch(epoch);
        this.flushPending();
      });
    }
    get sys() {
      return this.caps.get(SYS_CAP);
    }
    onProtoPhase(phase) {
      this.protoPhase = phase;
    }
    onInstancePhase(phase) {
      this.instancePhase = phase;
    }
    onMountPhase(phase, _epoch) {
      this.mountPhase = phase;
    }
    onCapsEpoch(_epoch) {}
    defer(fn) {
      this.pending.push(fn);
    }
    flushPending() {
      if (this.pending.length === 0)
        return;
      const tasks = this.pending;
      this.pending = [];
      for (const t of tasks)
        t();
    }
  }
  // ../packages/modules/base/src/create-module.ts
  function defineModule(def) {
    return def;
  }
  function createModule(args) {
    const { facade, hooks, port } = args.build({
      init: args.init,
      caps: args.caps,
      deps: args.deps
    });
    return {
      name: args.name,
      scope: args.scope,
      facade,
      hooks: hooks ?? {},
      port
    };
  }
  // ../packages/modules/expose/src/kernel.ts
  class ExposeKernel {
    map = new Map;
    has(key) {
      return this.map.has(key);
    }
    get(key) {
      return this.map.get(key);
    }
    set(key, value) {
      this.map.set(key, value);
    }
    keys() {
      return Array.from(this.map.keys());
    }
    entries() {
      return Array.from(this.map.entries()).map(([key, value]) => ({
        key,
        value
      }));
    }
    toRecord() {
      const out = {};
      for (const [k, v] of this.map) {
        Object.defineProperty(out, k, {
          value: v,
          enumerable: true,
          configurable: true,
          writable: true
        });
      }
      return out;
    }
    clear() {
      this.map.clear();
    }
  }

  // ../packages/modules/expose/src/error.ts
  class ExposeError extends Error {
    code;
    detail;
    constructor(code, message, detail) {
      super(message);
      this.name = "ExposeError";
      this.code = code;
      this.detail = detail;
    }
  }
  function exposeInvalidKey(message, detail) {
    return new ExposeError("EXPOSE_INVALID_KEY", message, detail);
  }
  function exposeDuplicateKey(message, detail) {
    return new ExposeError("EXPOSE_DUPLICATE_KEY", message, detail);
  }
  function exposeDisposed(message, detail) {
    return new ExposeError("EXPOSE_DISPOSED", message, detail);
  }
  function exposePhaseViolation(message, detail) {
    return new ExposeError("EXPOSE_PHASE_VIOLATION", message, detail);
  }

  // ../packages/modules/expose/src/impl.ts
  function isValidKey(key) {
    return typeof key === "string" && key.length > 0;
  }
  function toDiag(key, value) {
    const t = typeof value;
    return {
      key,
      valueType: t,
      isFunction: t === "function",
      isObject: value !== null && t === "object"
    };
  }

  class ExposeModuleImpl extends ModuleBase {
    kernel = new ExposeKernel;
    prototypeName;
    disposed = false;
    constructor(caps2, prototypeName) {
      super(caps2);
      this.prototypeName = prototypeName;
    }
    expose(key, value) {
      this.ensureSetup("def.expose");
      this.ensureAlive("def.expose");
      if (!isValidKey(key)) {
        throw exposeInvalidKey(`[Expose] key must be a non-empty string.`, {
          prototypeName: this.prototypeName,
          key
        });
      }
      if (this.kernel.has(key)) {
        throw exposeDuplicateKey(`[Expose] duplicate key: ${key}`, {
          prototypeName: this.prototypeName,
          key
        });
      }
      this.kernel.set(key, value);
    }
    port = {
      get: (key) => {
        this.ensureAlive("rt.expose.get");
        return this.kernel.get(key);
      },
      getAll: () => {
        this.ensureAlive("rt.expose.getAll");
        return this.kernel.toRecord();
      },
      has: (key) => {
        this.ensureAlive("rt.expose.has");
        return this.kernel.has(key);
      },
      keys: () => {
        this.ensureAlive("rt.expose.keys");
        return this.kernel.keys();
      },
      getDiagnostics: () => {
        this.ensureAlive("rt.expose.getDiagnostics");
        const entries = this.kernel.entries();
        return entries.map((e) => toDiag(e.key, e.value));
      }
    };
    dispose() {
      if (this.disposed)
        return;
      this.disposed = true;
      this.kernel.clear();
    }
    ensureSetup(op) {
      if (this.sys) {
        try {
          this.sys.ensureSetup(op);
          return;
        } catch (e) {
          throw exposePhaseViolation(`[Expose] setup-only: ${op}`, {
            prototypeName: this.prototypeName,
            error: e
          });
        }
      }
      if (this.protoPhase !== "setup") {
        throw illegalPhase(op, this.protoPhase, {
          prototypeName: this.prototypeName
        });
      }
    }
    ensureAlive(op) {
      this.sys?.ensureNotDisposed(op);
      if (this.disposed) {
        throw exposeDisposed(`[Expose] disposed. op=${op}`, {
          prototypeName: this.prototypeName
        });
      }
    }
    facade = {
      expose: (key, value) => this.expose(key, value)
    };
  }

  // ../packages/modules/expose/src/create.ts
  function createExposeModule(ctx) {
    const { init, caps: caps2, deps } = ctx;
    return createModule({
      name: "expose",
      scope: "instance",
      init,
      caps: caps2,
      deps,
      build: ({ init: init2, caps: caps3 }) => {
        const impl = new ExposeModuleImpl(caps3, init2.prototypeName);
        return {
          facade: impl.facade,
          port: impl.port,
          hooks: {
            dispose: () => impl.dispose()
          }
        };
      }
    });
  }
  var ExposeModuleDef = defineModule({
    name: "expose",
    resourceOwnership: "instance",
    deps: [],
    create: createExposeModule
  });
  // ../packages/modules/expose/src/types.ts
  var EXPOSE_ENTRY_CLASSIFICATION = Symbol.for("@proto.ui/expose/entry-classification");
  function createExposeEventDeclaration(spec2) {
    return Object.freeze({
      [EXPOSE_ENTRY_CLASSIFICATION]: "event",
      __pui_expose: "event",
      spec: spec2
    });
  }
  function isExposeEventDeclaration(value) {
    return !!value && typeof value === "object" && value[EXPOSE_ENTRY_CLASSIFICATION] === "event";
  }
  // ../packages/runtime/src/kernel/event/runtime-event-callbacks.ts
  class RuntimeEventCallbacks {
    map = new Map;
    register(id, cb) {
      this.map.set(id, cb);
    }
    remove(id) {
      this.map.delete(id);
    }
    dispatch(run, id, ev) {
      const cb = this.map.get(id);
      if (!cb)
        return;
      cb(run, ev);
    }
    clear() {
      this.map.clear();
    }
  }
  var __RT_EVENT_CALLBACKS = Symbol.for("__rt_event_callbacks");
  // ../packages/runtime/src/kernel/handles/def.ts
  function createLifecycleRegistry() {
    return { created: [], mounted: [], updated: [], unmounted: [], beforeDispose: [] };
  }
  var createDefHandle = (st, life, rules, modules, eventSink) => {
    const recordCaptured = (def2, kind, entry) => {
      const rt = getAsHookRuntime(def2);
      rt?.recordCaptured(kind, entry);
    };
    const registerStateHandle = (def2, handle) => {
      const rt = getAsHookRuntime(def2);
      const name = handle?.__stateName;
      rt?.registerStateName(name, handle?.__stateId);
    };
    const facades = modules.getFacades();
    const feedback2 = facades["feedback"];
    const props = facades["props"];
    const state2 = facades["state"];
    const stateInteraction = facades["state-interaction"];
    const stateAccessibility = facades["state-accessibility"];
    const a11y2 = facades["a11y"];
    const context2 = facades["context"];
    const expose = facades["expose"];
    const anatomy2 = facades["anatomy"];
    const eventFacade = facades["event"];
    const exposeEventFacade = facades["expose-event"];
    const eventCallbacks = new RuntimeEventCallbacks;
    eventSink?.setEventCallbacks(eventCallbacks);
    const ensureSetup = (op) => {
      const phase = st.getPhase();
      if (phase !== "setup") {
        illegalPhase2(op, st.prototypeName, phase, `Use 'run' inside runtime callbacks, not 'def'.`);
      }
    };
    const def = {
      lifecycle: {
        onCreated(cb) {
          ensureSetup(`def.lifecycle.onCreated`);
          life.created.push(cb);
        },
        onMounted(cb) {
          ensureSetup(`def.lifecycle.onMounted`);
          life.mounted.push(cb);
        },
        onUpdated(cb) {
          ensureSetup(`def.lifecycle.onUpdated`);
          life.updated.push(cb);
        },
        onUnmounted(cb) {
          ensureSetup(`def.lifecycle.onUnmounted`);
          life.unmounted.push(cb);
        },
        onBeforeDispose(cb) {
          ensureSetup(`def.lifecycle.onBeforeDispose`);
          life.beforeDispose.push(cb);
        }
      },
      props: {
        define(specMap) {
          ensureSetup(`def.props.define`);
          props.define(specMap);
          recordCaptured(def, "props", { op: "define", specMap });
        },
        setDefaults(partial) {
          ensureSetup(`def.props.setDefaults`);
          props.setDefaults(partial);
          recordCaptured(def, "props", { op: "setDefaults", partial });
        },
        watch(keys, cb) {
          ensureSetup(`def.props.watch`);
          const off = props.watch(keys, (ctx, next, prev, info) => cb(ctx, next, prev, info));
          recordCaptured(def, "props", { op: "watch", keys: [...keys], off });
          return off;
        },
        watchAll(cb) {
          ensureSetup(`def.props.watchAll`);
          const off = props.watchAll((ctx, next, prev, info) => cb(ctx, next, prev, info));
          recordCaptured(def, "props", { op: "watchAll", off });
          return off;
        },
        watchRaw(keys, cb) {
          ensureSetup(`def.props.watchRaw`);
          const off = props.watchRaw(keys, (ctx, next, prev, info) => cb(ctx, next, prev, info));
          recordCaptured(def, "props", { op: "watchRaw", keys: [...keys], off });
          return off;
        },
        watchRawAll(cb) {
          ensureSetup(`def.props.watchRawAll`);
          const off = props.watchRawAll((ctx, next, prev, info) => cb(ctx, next, prev, info));
          recordCaptured(def, "props", { op: "watchRawAll", off });
          return off;
        }
      },
      feedback: {
        style: {
          use: (...handles2) => {
            ensureSetup(`def.feedback.style.use`);
            const unUse = feedback2.style.use(...handles2);
            recordCaptured(def, "feedback", unUse);
            return () => {
              ensureSetup(`def.feedback.style.use:unUse`);
              unUse();
            };
          }
        }
      },
      expose: (() => {
        const fn = (key, value) => {
          ensureSetup("def.expose");
          expose.expose(key, value);
          recordCaptured(def, "context", { op: "expose", key, value });
        };
        fn.event = (key, spec2) => {
          ensureSetup("def.expose.event");
          expose.expose(key, createExposeEventDeclaration(spec2));
          exposeEventFacade.registerExposeEvent(key, spec2);
          recordCaptured(def, "event", { op: "expose.event", key, spec: spec2 });
        };
        fn.state = (key, handle) => {
          ensureSetup("def.expose.state");
          expose.expose(key, handle);
          recordCaptured(def, "state", { op: "expose.state", key, handle });
        };
        fn.value = (key, value) => {
          ensureSetup("def.expose.value");
          expose.expose(key, value);
          recordCaptured(def, "context", { op: "expose.value", key, value });
        };
        fn.method = (key, fnValue) => {
          ensureSetup("def.expose.method");
          expose.expose(key, fnValue);
          recordCaptured(def, "context", { op: "expose.method", key, fn: fnValue });
        };
        return fn;
      })(),
      rule: (spec2) => {
        ensureSetup("def.rule");
        const handle = rules.rule(spec2);
        recordCaptured(def, "context", { op: "rule", handle, off: () => handle.dispose() });
        return handle;
      },
      event: {
        on: (type, cb, options) => {
          ensureSetup(`def.event.on`);
          const token = eventFacade.on(type, options);
          eventCallbacks.register(token.id, cb);
          const off = () => {
            const id = token?.id;
            if (typeof id === "string" && id) {
              eventCallbacks.remove(id);
            }
          };
          recordCaptured(def, "event", { token, off });
          return token;
        },
        onGlobal: (type, cb, options) => {
          ensureSetup(`def.event.onGlobal`);
          const token = eventFacade.onGlobal(type, options);
          eventCallbacks.register(token.id, cb);
          const off = () => {
            const id = token?.id;
            if (typeof id === "string" && id) {
              eventCallbacks.remove(id);
            }
          };
          recordCaptured(def, "event", { token, off });
          return token;
        },
        off: (token) => {
          ensureSetup(`def.event.off`);
          const id = token?.id;
          if (typeof id === "string" && id) {
            eventCallbacks.remove(id);
          }
          eventFacade.off(token);
          recordCaptured(def, "event", { op: "off", token });
        }
      },
      state: {
        bool(semantic, defaultValue) {
          ensureSetup("def.state.bool");
          const handle = state2.bool(semantic, defaultValue);
          registerStateHandle(def, handle);
          recordCaptured(def, "state", handle);
          return handle;
        },
        fromInteraction(name) {
          ensureSetup("def.state.fromInteraction");
          if (!stateInteraction) {
            throw new Error(`[StateInteraction] module unavailable for state: ${String(name)}`);
          }
          const handle = stateInteraction.get(name);
          registerStateHandle(def, handle);
          recordCaptured(def, "state", handle);
          return handle;
        },
        fromAccessibility(name) {
          ensureSetup("def.state.fromAccessibility");
          if (!stateAccessibility) {
            throw new Error(`[StateAccessibility] module unavailable for state: ${String(name)}`);
          }
          const handle = stateAccessibility.get(name);
          registerStateHandle(def, handle);
          recordCaptured(def, "state", handle);
          return handle;
        },
        enum(semantic, defaultValue, spec2) {
          ensureSetup("def.state.enum");
          const handle = state2.enum(semantic, defaultValue, spec2);
          registerStateHandle(def, handle);
          recordCaptured(def, "state", handle);
          return handle;
        },
        string(semantic, defaultValue, spec2) {
          ensureSetup("def.state.string");
          const handle = state2.string(semantic, defaultValue, spec2);
          registerStateHandle(def, handle);
          recordCaptured(def, "state", handle);
          return handle;
        },
        numberRange(semantic, defaultValue, spec2) {
          ensureSetup("def.state.numberRange");
          const handle = state2.numberRange(semantic, defaultValue, spec2);
          registerStateHandle(def, handle);
          recordCaptured(def, "state", handle);
          return handle;
        },
        numberDiscrete(semantic, defaultValue, spec2) {
          ensureSetup("def.state.numberDiscrete");
          const handle = state2.numberDiscrete(semantic, defaultValue, spec2);
          registerStateHandle(def, handle);
          recordCaptured(def, "state", handle);
          return handle;
        }
      },
      context: {
        provide(key, defaultValue) {
          ensureSetup("def.context.provide");
          context2.provide(key, defaultValue);
          recordCaptured(def, "context", { op: "provide", key });
        },
        subscribe(key, cb) {
          ensureSetup("def.context.subscribe");
          if (!cb) {
            const off2 = context2.subscribe(key);
            recordCaptured(def, "context", { op: "subscribe", key, off: off2 });
            return off2;
          }
          const off = context2.subscribe(key, (ctx, next, prev) => cb(ctx, next, prev));
          recordCaptured(def, "context", { op: "subscribe", key, hasCallback: true, off });
          return off;
        },
        trySubscribe(key, cb) {
          ensureSetup("def.context.trySubscribe");
          if (!cb) {
            const off2 = context2.trySubscribe(key);
            recordCaptured(def, "context", { op: "trySubscribe", key, off: off2 });
            return off2;
          }
          const off = context2.trySubscribe(key, (ctx, next, prev) => cb(ctx, next, prev));
          recordCaptured(def, "context", { op: "trySubscribe", key, hasCallback: true, off });
          return off;
        }
      },
      anatomy: {
        claim(family, decl) {
          ensureSetup("def.anatomy.claim");
          if (!anatomy2)
            throw new Error(`[Anatomy] module unavailable`);
          anatomy2.claim(family, decl);
        },
        subscribeParts(family, role, onChange) {
          ensureSetup("def.anatomy.subscribeParts");
          if (!anatomy2)
            throw new Error(`[Anatomy] module unavailable`);
          return anatomy2.subscribeParts(family, role, (ctx, parts) => onChange(ctx, parts));
        }
      },
      a11y: {
        id(target) {
          ensureSetup("def.a11y.id");
          if (!a11y2)
            throw new Error(`[A11y] module unavailable.`);
          a11y2.id(target);
          recordCaptured(def, "context", { op: "a11y.id", target });
        },
        role(role) {
          ensureSetup("def.a11y.role");
          if (!a11y2)
            throw new Error(`[A11y] module unavailable.`);
          a11y2.role(role);
          recordCaptured(def, "context", { op: "a11y.role", role });
        },
        name(value) {
          ensureSetup("def.a11y.name");
          if (!a11y2)
            throw new Error(`[A11y] module unavailable.`);
          a11y2.name(value);
          recordCaptured(def, "context", { op: "a11y.name", value });
        },
        nameFromContent() {
          ensureSetup("def.a11y.nameFromContent");
          if (!a11y2)
            throw new Error(`[A11y] module unavailable.`);
          a11y2.nameFromContent();
          recordCaptured(def, "context", { op: "a11y.nameFromContent" });
        },
        description(value) {
          ensureSetup("def.a11y.description");
          if (!a11y2)
            throw new Error(`[A11y] module unavailable.`);
          a11y2.description(value);
          recordCaptured(def, "context", { op: "a11y.description", value });
        },
        state(key, handle) {
          ensureSetup("def.a11y.state");
          if (!a11y2)
            throw new Error(`[A11y] module unavailable.`);
          a11y2.state(key, handle);
          recordCaptured(def, "state", { op: "a11y.state", key, handle });
        },
        action(key, spec2) {
          ensureSetup("def.a11y.action");
          if (!a11y2)
            throw new Error(`[A11y] module unavailable.`);
          a11y2.action(key, spec2);
          recordCaptured(def, "event", { op: "a11y.action", key, spec: spec2 });
        },
        relation(key, spec2) {
          ensureSetup("def.a11y.relation");
          if (!a11y2)
            throw new Error(`[A11y] module unavailable.`);
          a11y2.relation(key, spec2);
          recordCaptured(def, "context", { op: "a11y.relation", key, spec: spec2 });
        },
        tree(patch) {
          ensureSetup("def.a11y.tree");
          if (!a11y2)
            throw new Error(`[A11y] module unavailable.`);
          a11y2.tree(patch);
          recordCaptured(def, "context", { op: "a11y.tree", patch });
        }
      }
    };
    return def;
  };
  // ../packages/runtime/src/kernel/handles/run.ts
  var createRunHandle = (update, moduleHub, setPresent) => {
    const facades = moduleHub.getFacades();
    const props = facades["props"];
    const context2 = facades["context"];
    const exposeEvent = facades["expose-event"];
    const anatomy2 = facades["anatomy"];
    const feedback2 = facades["feedback"];
    const meta = facades["rule-meta"];
    return {
      update,
      lifecycle: {
        setPresent
      },
      meta: meta ? {
        get: (key) => meta.get(key)
      } : undefined,
      props: {
        get: () => props.get(),
        getRaw: () => props.getRaw(),
        isProvided: (k) => props.isProvided(k)
      },
      context: {
        read: (key) => context2.read(key),
        tryRead: (key) => context2.tryRead(key),
        update: (key, next) => context2.update(key, next),
        tryUpdate: (key, next) => context2.tryUpdate(key, next)
      },
      expose: {
        emit: (key, payload, options) => exposeEvent.emit(key, payload, options)
      },
      feedback: {
        style: {
          patch: (...handles2) => feedback2.style.patch(...handles2),
          suppress: (...handles2) => feedback2.style.suppress(...handles2),
          clearPatch: () => feedback2.style.clearPatch()
        }
      },
      anatomy: {
        has: (family, role) => {
          if (!anatomy2)
            throw new Error(`[Anatomy] module unavailable`);
          return anatomy2.has(family, role);
        },
        parts: (family, options) => {
          if (!anatomy2)
            throw new Error(`[Anatomy] module unavailable`);
          return anatomy2.parts(family, options);
        },
        partsOf: (family, role, options) => {
          if (!anatomy2)
            throw new Error(`[Anatomy] module unavailable`);
          return anatomy2.partsOf(family, role, options);
        },
        order: {
          version: (family, options) => {
            if (!anatomy2)
              throw new Error(`[Anatomy] module unavailable`);
            return anatomy2.order.version(family, options);
          },
          parts: (family, options) => {
            if (!anatomy2)
              throw new Error(`[Anatomy] module unavailable`);
            return anatomy2.order.parts(family, options);
          },
          partsOf: (family, role, options) => {
            if (!anatomy2)
              throw new Error(`[Anatomy] module unavailable`);
            return anatomy2.order.partsOf(family, role, options);
          },
          indexOfSelf: (family, role, options) => {
            if (!anatomy2)
              throw new Error(`[Anatomy] module unavailable`);
            return anatomy2.order.indexOfSelf(family, role, options);
          },
          prevOfSelf: (family, role, options) => {
            if (!anatomy2)
              throw new Error(`[Anatomy] module unavailable`);
            return anatomy2.order.prevOfSelf(family, role, options);
          },
          nextOfSelf: (family, role, options) => {
            if (!anatomy2)
              throw new Error(`[Anatomy] module unavailable`);
            return anatomy2.order.nextOfSelf(family, role, options);
          }
        }
      }
    };
  };
  // ../packages/runtime/src/kernel/as-hook.ts
  var TRACE_INTERNAL = Symbol.for("@proto.ui/asHook/trace-internal");
  function isStateHandleLike(x) {
    return !!x && typeof x === "object" && typeof x.get === "function" && !!x.__stateId;
  }
  function collectNamedStateHandles(entries) {
    const named = new Map;
    const seenIds = new Set;
    for (const entry of entries) {
      const handle = entry?.op === "expose.state" || entry?.op === "a11y.state" ? entry.handle : entry;
      if (!isStateHandleLike(handle))
        continue;
      const name = handle.__stateName;
      if (typeof name !== "string" || !name)
        continue;
      const id = handle.__stateId ?? name;
      if (seenIds.has(id))
        continue;
      seenIds.add(id);
      named.set(name, handle);
    }
    if (named.size === 0)
      return;
    return Object.fromEntries(named);
  }
  function collectDisposers(entries, predicate) {
    const out = [];
    for (const entry of entries) {
      if (predicate && !predicate(entry))
        continue;
      if (typeof entry === "function") {
        out.push(entry);
        continue;
      }
      const off = entry?.off;
      if (typeof off === "function")
        out.push(off);
    }
    return out;
  }
  function wrapSetupOnlyDisposer(disposer, ensureSetup) {
    return () => {
      ensureSetup("asHook.disposer");
      disposer();
    };
  }
  function collectEventKeys(entries) {
    const out = {};
    for (const entry of entries) {
      if (entry?.op !== "expose.event")
        continue;
      const key = entry?.key;
      if (typeof key !== "string" || !key)
        continue;
      out[key] = key;
    }
    return Object.keys(out).length > 0 ? out : undefined;
  }
  function collectExposeMethods(entries) {
    const out = {};
    for (const entry of entries) {
      if (entry?.op !== "expose.method")
        continue;
      const key = entry?.key;
      if (typeof key !== "string" || !key)
        continue;
      out[key] = entry?.fn;
    }
    return Object.keys(out).length > 0 ? Object.freeze(out) : undefined;
  }
  function getOrCreateTrace(proto) {
    const anyProto = proto;
    if (!anyProto[TRACE_INTERNAL]) {
      const store = { entries: [], nameSet: new Set };
      Object.defineProperty(anyProto, TRACE_INTERNAL, {
        value: store,
        enumerable: false,
        configurable: false,
        writable: false
      });
      Object.defineProperty(anyProto, "__asHooks", {
        get: () => Object.freeze(store.entries.slice()),
        enumerable: false,
        configurable: false
      });
    }
    return anyProto[TRACE_INTERNAL] ?? {
      entries: [],
      nameSet: new Set
    };
  }
  function createBorrowedHandle(port, handle) {
    const raw = port.createBorrowedHandle(handle);
    const borrowed = {
      get: () => raw.get(),
      setDefault: (v) => raw.setDefault(v),
      set: (v, reason) => raw.set(v, reason),
      watch: (cb) => raw.watch((ctx, e) => cb(ctx, e))
    };
    borrowed.__stateId = handle.__stateId;
    borrowed.__stateName = handle.__stateName;
    borrowed.__stateSemantic = handle.__stateSemantic;
    borrowed.__stateKind = handle.__stateKind;
    borrowed.__stateSpec = handle.__stateSpec;
    return borrowed;
  }
  function projectStateValue(port, value) {
    if (isStateHandleLike(value)) {
      return createBorrowedHandle(port, value);
    }
    if (Array.isArray(value)) {
      let changed = false;
      const mapped = value.map((v) => {
        const next = projectStateValue(port, v);
        if (next !== v)
          changed = true;
        return next;
      });
      return changed ? mapped : value;
    }
    if (value && typeof value === "object") {
      let changed = false;
      const out = {};
      for (const [k, v] of Object.entries(value)) {
        const next = projectStateValue(port, v);
        if (next !== v)
          changed = true;
        out[k] = next;
      }
      return changed ? out : value;
    }
    return value;
  }
  function createAsHookStateProjector(port) {
    if (!port)
      return (state2) => state2;
    return (state2) => projectStateValue(port, state2);
  }
  function attachAsHookRuntime(def2, st, proto, opt) {
    const trace = getOrCreateTrace(proto);
    const instances = new Map;
    const frameStack = [];
    const rootStateNames = new Map;
    let instanceOrder = 0;
    const projectState = opt?.projectState ?? ((state2) => state2);
    const createFrame = (name, meta) => ({
      name,
      order: meta.order,
      privileged: meta.privileged,
      mode: meta.mode,
      asHooks: [],
      stateNames: new Map,
      effects: {
        props: [],
        state: [],
        context: [],
        event: [],
        feedback: []
      }
    });
    const compact = (values) => {
      if (values.length === 0)
        return;
      if (values.length === 1)
        return values[0];
      return values.slice();
    };
    const ensureSetup = (op) => {
      const phase = st.getPhase();
      if (phase !== "setup") {
        illegalPhase2(op, st.prototypeName, phase, `Use 'asHook' in setup only.`);
      }
    };
    const validateStateName = (name) => {
      if (typeof name !== "string" || name.length === 0) {
        throw new Error(`[State] state name must be a non-empty string.`);
      }
    };
    const registerStateNameIn = (names, name, stateId) => {
      validateStateName(name);
      const existing = names.get(name);
      if (!existing) {
        names.set(name, { stateId });
        return;
      }
      if (typeof existing.stateId !== "undefined" && typeof stateId !== "undefined" && Object.is(existing.stateId, stateId)) {
        return;
      }
      throw new Error(`[State] duplicate state name in setup frame: ${name}`);
    };
    const runtime = {
      ensureSetup,
      register: (name, meta) => {
        const mode = meta.mode ?? "once";
        const existing = instances.get(name);
        if (mode === "multiple") {
          const order = instanceOrder++;
          const state2 = { store: {} };
          if (!trace.nameSet.has(name)) {
            const entry = {
              name,
              order,
              privileged: !!meta.privileged,
              mode
            };
            trace.nameSet.add(name);
            trace.entries.push(entry);
          }
          return { action: "setup", order, state: state2 };
        }
        if (!existing) {
          const order = instanceOrder++;
          const state2 = { store: {} };
          instances.set(name, { order, state: state2, mode });
          if (!trace.nameSet.has(name)) {
            const entry = {
              name,
              order,
              privileged: !!meta.privileged,
              mode
            };
            trace.nameSet.add(name);
            trace.entries.push(entry);
          }
          return { action: "setup", order, state: state2 };
        }
        if (existing.mode === "once" || mode === "once") {
          return { action: "skip", order: existing.order, state: existing.state };
        }
        return { action: "configure", order: existing.order, state: existing.state };
      },
      beginCapture: (name, meta) => {
        frameStack.push(createFrame(name, meta));
      },
      recordCaptured: (kind, entry) => {
        if (frameStack.length === 0)
          return;
        const frame = frameStack[frameStack.length - 1];
        frame.effects[kind].push(entry);
      },
      recordAsHookResult: (entry) => {
        ensureSetup("asHook.result");
        const frame = frameStack[frameStack.length - 1];
        if (!frame)
          return;
        frame.asHooks.push(Object.freeze({ ...entry }));
      },
      registerStateName: (name, stateId) => {
        ensureSetup("def.state");
        const frame = frameStack[frameStack.length - 1];
        registerStateNameIn(frame ? frame.stateNames : rootStateNames, name, stateId);
      },
      endCapture: (render) => {
        const frame = frameStack.pop();
        if (!frame)
          return render ? { render } : {};
        const result = {};
        const props = compact(frame.effects.props);
        const state2 = compact(frame.effects.state);
        const context2 = compact(frame.effects.context);
        const event = compact(frame.effects.event);
        const feedback2 = compact(frame.effects.feedback);
        const stateHandles = collectNamedStateHandles(frame.effects.state);
        const methods = collectExposeMethods(frame.effects.context);
        const propsDisposers = collectDisposers(frame.effects.props);
        const contextDisposers = collectDisposers(frame.effects.context, (entry) => entry?.op === "subscribe" || entry?.op === "trySubscribe");
        const ruleDisposers = collectDisposers(frame.effects.context, (entry) => entry?.op === "rule");
        const eventDisposers = collectDisposers(frame.effects.event);
        const feedbackDisposers = collectDisposers(frame.effects.feedback);
        const wrappedPropsDisposers = propsDisposers.map((disposer) => wrapSetupOnlyDisposer(disposer, ensureSetup));
        const wrappedContextDisposers = contextDisposers.map((disposer) => wrapSetupOnlyDisposer(disposer, ensureSetup));
        const wrappedRuleDisposers = ruleDisposers.map((disposer) => wrapSetupOnlyDisposer(disposer, ensureSetup));
        const wrappedEventDisposers = eventDisposers.map((disposer) => wrapSetupOnlyDisposer(disposer, ensureSetup));
        const wrappedFeedbackDisposers = feedbackDisposers.map((disposer) => wrapSetupOnlyDisposer(disposer, ensureSetup));
        const eventKeys = collectEventKeys(frame.effects.event);
        const allDisposers = [
          ...wrappedPropsDisposers,
          ...wrappedContextDisposers,
          ...wrappedRuleDisposers,
          ...wrappedEventDisposers,
          ...wrappedFeedbackDisposers
        ];
        if (typeof props !== "undefined")
          result.props = props;
        if (typeof state2 !== "undefined")
          result.state = state2;
        let projectedStateHandles;
        if (typeof stateHandles !== "undefined") {
          projectedStateHandles = projectState(stateHandles);
          result.stateHandles = Object.freeze(projectedStateHandles);
          result.getState = (key) => projectedStateHandles[key];
        }
        if (methods) {
          result.methods = methods;
          result.getMethod = (key) => methods[key];
        }
        if (frame.asHooks.length > 0) {
          const asHooks = Object.freeze(frame.asHooks.slice());
          result.asHooks = asHooks;
          result.getAsHook = (name) => asHooks.find((entry) => entry.name === name);
          result.getAsHookHandle = (name) => asHooks.find((entry) => entry.name === name)?.handle;
        }
        if (projectedStateHandles || eventKeys || methods || frame.asHooks.length > 0) {
          const artifacts = {};
          if (projectedStateHandles)
            artifacts.stateHandles = result.stateHandles;
          if (eventKeys)
            artifacts.eventKeys = Object.freeze(eventKeys);
          if (methods)
            artifacts.methods = methods;
          if (frame.asHooks.length > 0)
            artifacts.asHooks = result.asHooks;
          result.artifacts = Object.freeze(artifacts);
        }
        if (allDisposers.length > 0) {
          const disposers = {
            all: Object.freeze(allDisposers.slice())
          };
          if (propsDisposers.length > 0) {
            disposers.props = Object.freeze(wrappedPropsDisposers.slice());
          }
          if (contextDisposers.length > 0) {
            disposers.context = Object.freeze(wrappedContextDisposers.slice());
          }
          if (ruleDisposers.length > 0) {
            disposers.rule = Object.freeze(wrappedRuleDisposers.slice());
          }
          if (eventDisposers.length > 0) {
            disposers.event = Object.freeze(wrappedEventDisposers.slice());
          }
          if (feedbackDisposers.length > 0) {
            disposers.feedback = Object.freeze(wrappedFeedbackDisposers.slice());
          }
          result.disposers = Object.freeze(disposers);
        }
        if (typeof context2 !== "undefined")
          result.context = context2;
        if (typeof event !== "undefined")
          result.event = event;
        if (typeof feedback2 !== "undefined")
          result.feedback = feedback2;
        if (typeof render === "function")
          result.render = render;
        return result;
      },
      abortCapture: () => {
        if (frameStack.length === 0)
          return;
        frameStack.pop();
      },
      projectState: (state2) => {
        return projectState(state2);
      },
      getTrace: () => trace.entries.slice()
    };
    bindAsHookRuntime(def2, runtime);
    return runtime;
  }

  // ../packages/runtime/src/kernel/view-intent.ts
  function createViewIntent(args) {
    let snapshot = Object.freeze({ present: true, version: 0 });
    let terminal = false;
    const listeners = new Set;
    return {
      getSnapshot: () => snapshot,
      subscribe(listener) {
        if (terminal)
          return () => {};
        listeners.add(listener);
        return () => listeners.delete(listener);
      },
      setPresent(present) {
        if (terminal) {
          throw new Error(`[Lifecycle] cannot update view presence after terminal disposal begins: ${args.prototypeName}`);
        }
        const phase = args.getPhase();
        if (phase !== "callback") {
          illegalPhase2("run.lifecycle.setPresent", args.prototypeName, phase, `Call it from a runtime callback that receives 'run'.`);
        }
        if (snapshot.present === present)
          return;
        snapshot = Object.freeze({ present, version: snapshot.version + 1 });
        for (const listener of [...listeners])
          listener(snapshot);
      },
      lockTerminal() {
        terminal = true;
        listeners.clear();
      }
    };
  }

  // ../packages/runtime/src/kernel/kernel.ts
  function createKernel(proto, modules, opt) {
    let phase = "unknown";
    const setPhase = (p) => {
      phase = p;
      opt?.onPhaseChange?.(p);
    };
    const st = {
      prototypeName: proto.name,
      getPhase: () => phase
    };
    const lifecycle = createLifecycleRegistry();
    const viewIntent = createViewIntent(st);
    const rules = modules.getFacades()["rule"];
    const def2 = createDefHandle(st, lifecycle, rules, modules, opt?.eventSink);
    const asHookRuntime = attachAsHookRuntime(def2, st, proto, opt?.asHook);
    setPhase("setup");
    opt?.asHook?.enterSetup?.({ def: def2, rt: asHookRuntime });
    let maybeRender;
    try {
      maybeRender = proto.setup(def2);
    } finally {
      opt?.asHook?.exitSetup?.();
    }
    const defaultRender = (renderer2) => [renderer2.slot()];
    let renderFn = defaultRender;
    if (typeof maybeRender === "function") {
      renderFn = maybeRender;
    } else if (typeof maybeRender !== "undefined") {
      throw new Error(`[Prototype] setup() must return render function or void, got: ${typeof maybeRender}.`);
    }
    setPhase("unknown");
    let runUpdateImpl = undefined;
    if (opt?.allowRunUpdate) {
      runUpdateImpl = () => {
        throw new Error(`[Runtime] run.update() is not wired yet.`);
      };
    }
    const run2 = createRunHandle(() => {
      if (!runUpdateImpl) {
        throw new Error(`[Runtime] run.update() is not supported in host-free execution.`);
      }
      runUpdateImpl();
    }, modules, (present) => viewIntent.setPresent(present));
    const facades = modules.getFacades();
    const propsFacade = facades["props"];
    const contextFacade = facades["context"];
    const anatomyFacade = facades["anatomy"];
    const read = {
      props: propsFacade,
      context: {
        read: (key) => contextFacade.read(key),
        tryRead: (key) => contextFacade.tryRead(key)
      },
      anatomy: {
        has: (family, role) => {
          if (!anatomyFacade)
            throw new Error(`[Anatomy] module unavailable`);
          return anatomyFacade.has(family, role);
        },
        parts: (family, options) => {
          if (!anatomyFacade)
            throw new Error(`[Anatomy] module unavailable`);
          return anatomyFacade.parts(family, options);
        },
        partsOf: (family, role, options) => {
          if (!anatomyFacade)
            throw new Error(`[Anatomy] module unavailable`);
          return anatomyFacade.partsOf(family, role, options);
        },
        order: {
          version: (family, options) => {
            if (!anatomyFacade)
              throw new Error(`[Anatomy] module unavailable`);
            return anatomyFacade.order.version(family, options);
          },
          parts: (family, options) => {
            if (!anatomyFacade)
              throw new Error(`[Anatomy] module unavailable`);
            return anatomyFacade.order.parts(family, options);
          },
          partsOf: (family, role, options) => {
            if (!anatomyFacade)
              throw new Error(`[Anatomy] module unavailable`);
            return anatomyFacade.order.partsOf(family, role, options);
          },
          indexOfSelf: (family, role, options) => {
            if (!anatomyFacade)
              throw new Error(`[Anatomy] module unavailable`);
            return anatomyFacade.order.indexOfSelf(family, role, options);
          },
          prevOfSelf: (family, role, options) => {
            if (!anatomyFacade)
              throw new Error(`[Anatomy] module unavailable`);
            return anatomyFacade.order.prevOfSelf(family, role, options);
          },
          nextOfSelf: (family, role, options) => {
            if (!anatomyFacade)
              throw new Error(`[Anatomy] module unavailable`);
            return anatomyFacade.order.nextOfSelf(family, role, options);
          }
        }
      }
    };
    const { el, slot, r, svg } = createRendererPrimitives();
    const renderer = { el, slot, r, svg, read };
    const renderOnce = () => {
      setPhase("render");
      const children = renderFn(renderer);
      setPhase("unknown");
      return children;
    };
    return {
      getPhase: () => phase,
      setPhase: (p) => setPhase(p),
      lifecycle,
      viewIntent,
      rules,
      run: run2,
      read,
      renderer,
      renderFn,
      renderOnce
    };
  }
  // ../packages/runtime/src/kernel/lifecycle-events.ts
  function projectLegacyCheckpoint(event) {
    switch (event.type) {
      case "instance.setup.exit":
        return "CP0_SETUP_EXIT";
      case "instance.created":
        return "CP1_CREATED_CALLBACKS";
      case "mount.render":
        return "CP2_LOGICAL_TREE_READY";
      case "mount.commit.start":
        return "CP3_COMMIT_START";
      case "mount.commit.done":
        return "CP4_COMMIT_DONE";
      case "mount.mounted":
        return "CP5_MOUNTED_CALLBACKS";
      case "update.render":
        return "CP6_UPDATE_RENDER";
      case "update.commit.done":
        return "CP7_UPDATE_COMMIT_DONE";
      case "update.updated":
        return "CP8_UPDATED_CALLBACKS";
      case "unmount.begin":
        return "CP9_UNMOUNT_BEGIN";
      case "instance.dispose.done":
        return "CP10_DISPOSE_COMPLETE";
      default:
        return;
    }
  }
  // ../packages/modules/feedback/src/caps.ts
  var EFFECTS_CAP = cap("@proto.ui/feedback/effects");

  // ../packages/modules/feedback/src/create.ts
  function createFeedbackModule(ctx) {
    const { init, caps: caps2, deps } = ctx;
    return createModule({
      name: "feedback",
      scope: "instance",
      init,
      caps: caps2,
      deps,
      build: ({ init: init2, caps: caps3 }) => {

        class Impl extends ModuleBase {
          recorder = new FeedbackStyleRecorder;
          dirty = false;
          flushRequested = false;
          useStyle(handles3) {
            const op = "def.feedback.style.use";
            this.sys?.ensureSetup(op);
            if (!this.sys && this.protoPhase !== "setup") {
              throw illegalPhase(op, this.protoPhase, {
                prototypeName: init2.prototypeName,
                hint: `Use 'run' inside runtime callbacks, not 'def'.`
              });
            }
            const unUse = this.recorder.use(...handles3);
            this.dirty = true;
            return () => {
              unUse();
              this.dirty = true;
            };
          }
          useStyleRuntime(handles3) {
            const op = "rule.feedback.style.use";
            if (this.protoPhase === "setup") {
              throw illegalPhase(op, this.protoPhase, {
                prototypeName: init2.prototypeName,
                hint: `Use 'def' only during setup.`
              });
            }
            const unUse = this.recorder.use(...handles3);
            this.dirty = true;
            this.flushIfPossible();
            return this.createRuntimeStyleDisposer(unUse);
          }
          replaceStyleRuntime(previous, handles3) {
            const op = "rule.feedback.style.replace";
            if (this.protoPhase === "setup") {
              throw illegalPhase(op, this.protoPhase, {
                prototypeName: init2.prototypeName,
                hint: `Use 'def' only during setup.`
              });
            }
            previous?.({ flush: false });
            const next = handles3.length > 0 ? this.recorder.use(...handles3) : null;
            this.dirty = true;
            this.flushIfPossible();
            return next ? this.createRuntimeStyleDisposer(next) : null;
          }
          patchStyle(handles3) {
            const op = "run.feedback.style.patch";
            this.ensureRuntime(op);
            this.recorder.patch(...handles3);
            this.dirty = true;
            this.flushIfPossible();
          }
          suppressStyle(handles3) {
            const op = "run.feedback.style.suppress";
            this.ensureRuntime(op);
            this.recorder.suppress(...handles3);
            this.dirty = true;
            this.flushIfPossible();
          }
          clearStylePatch() {
            const op = "run.feedback.style.clearPatch";
            this.ensureRuntime(op);
            this.recorder.clearPatch();
            this.dirty = true;
            this.flushIfPossible();
          }
          useStyleUnsafe(handles3) {
            const unUse = this.recorder.useUnsafe(...handles3);
            this.dirty = true;
            this.flushIfPossible();
            return () => {
              unUse();
              this.dirty = true;
              this.flushIfPossible();
            };
          }
          exportMerged() {
            const { tokens } = this.recorder.export();
            return { kind: "tw", tokens };
          }
          onProtoPhase(phase) {
            super.onProtoPhase(phase);
            if (phase === "mounted")
              this.flushIfPossible();
          }
          onMountPhase(phase, epoch) {
            super.onMountPhase(phase, epoch);
            if (phase === "mounting") {
              this.replayStyleForViewEpoch();
            }
          }
          onCapsEpoch(_epoch) {
            this.flushIfPossible();
          }
          flushIfPossible() {
            if (this.protoPhase === "setup")
              return;
            if (this.mountPhase === "detached" || this.mountPhase === "unmounting")
              return;
            if (!this.dirty)
              return;
            if (!this.caps.has(EFFECTS_CAP)) {
              this.defer(() => this.flushIfPossible());
              return;
            }
            const effects2 = this.caps.get(EFFECTS_CAP);
            const merged = this.exportMerged();
            this.dirty = false;
            effects2.queueStyle(merged);
            this.flushRequested = true;
            effects2.requestFlush();
          }
          applyMergedStyle(handle) {
            if (this.protoPhase === "setup")
              return;
            if (!this.caps.has(EFFECTS_CAP)) {
              this.defer(() => this.applyMergedStyle(handle));
              return;
            }
            const effects2 = this.caps.get(EFFECTS_CAP);
            const merged = this.recorder.exportWithAdditional(handle);
            effects2.queueStyle({ kind: "tw", tokens: merged.tokens });
            effects2.requestFlush();
            this.flushRequested = true;
          }
          afterRenderCommit() {
            if (!this.caps.has(EFFECTS_CAP))
              return;
            const effects2 = this.caps.get(EFFECTS_CAP);
            const merged = this.exportMerged();
            effects2.queueStyle(merged);
            effects2.requestFlush();
            this.flushRequested = true;
          }
          replayStyleForViewEpoch() {
            if (!this.caps.has(EFFECTS_CAP))
              return;
            const effects2 = this.caps.get(EFFECTS_CAP);
            effects2.queueStyle(this.exportMerged());
            effects2.requestFlush();
            this.flushRequested = true;
          }
          onEffectsFlushed() {
            this.flushRequested = false;
            if (this.dirty && this.caps.has(EFFECTS_CAP)) {
              this.caps.get(EFFECTS_CAP).requestFlush();
              this.flushRequested = true;
            }
          }
          ensureRuntime(op) {
            this.sys?.ensureRuntime(op);
            if (!this.sys && this.protoPhase === "setup") {
              throw illegalPhase(op, this.protoPhase, {
                prototypeName: init2.prototypeName,
                hint: `Use 'run' only after setup.`
              });
            }
          }
          createRuntimeStyleDisposer(unUse) {
            return (options = {}) => {
              unUse();
              this.dirty = true;
              if (options.flush !== false)
                this.flushIfPossible();
            };
          }
        }
        const impl = new Impl(caps3);
        const facade = {
          style: {
            use: (...handles3) => impl.useStyle(handles3),
            patch: (...handles3) => impl.patchStyle(handles3),
            suppress: (...handles3) => impl.suppressStyle(handles3),
            clearPatch: () => impl.clearStylePatch(),
            exportMerged: () => impl.exportMerged()
          }
        };
        return {
          facade,
          port: {
            applyMergedStyle: (h) => impl.applyMergedStyle(h),
            useStyleRuntime: (...handles3) => impl.useStyleRuntime(handles3),
            replaceStyleRuntime: (previous, ...handles3) => impl.replaceStyleRuntime(previous, handles3),
            patchStyle: (...handles3) => impl.patchStyle(handles3),
            suppressStyle: (...handles3) => impl.suppressStyle(handles3),
            clearStylePatch: () => impl.clearStylePatch(),
            useStyleUnsafe: (...handles3) => impl.useStyleUnsafe(handles3)
          },
          hooks: {
            onMountPhase: (p, epoch) => impl.onMountPhase(p, epoch),
            onProtoPhase: (p) => impl.onProtoPhase(p),
            afterRenderCommit: () => impl.afterRenderCommit(),
            flushIfPossible: () => impl.flushIfPossible(),
            onEffectsFlushed: () => impl.onEffectsFlushed()
          }
        };
      }
    });
  }
  var FeedbackModuleDef = defineModule({
    name: "feedback",
    resourceOwnership: "mixed",
    deps: [],
    create: createFeedbackModule
  });
  // ../packages/modules/props/src/caps.ts
  var RAW_PROPS_SOURCE_CAP = cap("@proto.ui/props/rawPropsSource");

  // ../packages/modules/props/src/kernel/json-value.ts
  function isJsonPropsValue(value, seen = new Set) {
    if (value === null)
      return true;
    switch (typeof value) {
      case "string":
      case "boolean":
        return true;
      case "number":
        return Number.isFinite(value);
      case "object":
        break;
      default:
        return false;
    }
    if (seen.has(value))
      return false;
    seen.add(value);
    if (Array.isArray(value)) {
      for (let index = 0;index < value.length; index += 1) {
        if (!(index in value))
          return false;
        if (!isJsonPropsValue(value[index], seen))
          return false;
      }
      seen.delete(value);
      return true;
    }
    const prototype2 = Object.getPrototypeOf(value);
    if (prototype2 !== Object.prototype && prototype2 !== null)
      return false;
    if (Object.getOwnPropertySymbols(value).length > 0)
      return false;
    for (const key of Object.keys(value)) {
      if (!isJsonPropsValue(value[key], seen))
        return false;
    }
    seen.delete(value);
    return true;
  }

  // ../packages/modules/props/src/kernel/merge.ts
  function hasOwn(obj, key) {
    return Object.prototype.hasOwnProperty.call(obj, key);
  }
  function isNumberOrUndef(x) {
    return x === undefined || typeof x === "number" && !Number.isNaN(x);
  }
  function isValidEmptyBehavior(x) {
    return x === "accept" || x === "fallback" || x === "error";
  }
  function isValidPropType(x) {
    return x === "any" || x === "boolean" || x === "string" || x === "number" || x === "object" || x === "enum";
  }
  function isValidOptions(x) {
    return Array.isArray(x) && x.length > 0 && x.every((option) => typeof option === "string");
  }
  function isSupersetOptions(next, prev) {
    if (!prev || prev.length === 0)
      return true;
    if (!next)
      return false;
    const set = new Set(next);
    return prev.every((x) => set.has(x));
  }
  function rangeWider(next, prev) {
    if (!prev)
      return true;
    if (!next)
      return false;
    const prevMin = prev.min ?? -Infinity;
    const prevMax = prev.max ?? Infinity;
    const nextMin = next.min ?? -Infinity;
    const nextMax = next.max ?? Infinity;
    return nextMin <= prevMin && nextMax >= prevMax;
  }
  function rangeNarrower(next, prev) {
    if (!next)
      return false;
    if (!prev)
      return false;
    const prevMin = prev.min ?? -Infinity;
    const prevMax = prev.max ?? Infinity;
    const nextMin = next.min ?? -Infinity;
    const nextMax = next.max ?? Infinity;
    return nextMin > prevMin || nextMax < prevMax;
  }
  function mergeSpecs(base, incoming) {
    const out = { ...base };
    const diags = [];
    for (const key of Object.keys(incoming)) {
      const next = incoming[key];
      const prev = out[key];
      if (!next)
        continue;
      if (!hasOwn(next, "type") || !isValidPropType(next.type)) {
        diags.push({
          level: "error",
          key,
          message: `type must be one of "any" | "boolean" | "string" | "number" | "object" | "enum"`
        });
        continue;
      }
      if (next.type === "enum" && !isValidOptions(next.options)) {
        diags.push({
          level: "error",
          key,
          message: `enum props require non-empty string options`
        });
        continue;
      }
      if (next.type !== "enum" && hasOwn(next, "options")) {
        diags.push({
          level: "error",
          key,
          message: `options are only allowed for type "enum"`
        });
        continue;
      }
      if (hasOwn(next, "enum")) {
        diags.push({
          level: "error",
          key,
          message: `enum descriptor field is deprecated; use type "enum" with options`
        });
        continue;
      }
      if (hasOwn(next, "empty")) {
        const e = next.empty;
        if (e === undefined || !isValidEmptyBehavior(e)) {
          diags.push({
            level: "error",
            key,
            message: `empty must be one of "fallback" | "accept" | "error"`
          });
          continue;
        }
      }
      if (next.range) {
        const r = next.range;
        if (!isNumberOrUndef(r.min)) {
          diags.push({
            level: "error",
            key,
            message: `range.min must be a number`
          });
          continue;
        }
        if (!isNumberOrUndef(r.max)) {
          diags.push({
            level: "error",
            key,
            message: `range.max must be a number`
          });
          continue;
        }
      }
      if (hasOwn(next, "default") && !isJsonPropsValue(next.default)) {
        diags.push({
          level: "error",
          key,
          message: `default must be a JSON props value`
        });
        continue;
      }
      if (!prev) {
        out[key] = { ...next };
        continue;
      }
      const prevType = prev.type ?? "any";
      const nextType = next.type ?? "any";
      if (prevType !== nextType) {
        diags.push({
          level: "error",
          key,
          message: `type conflict: ${prevType} vs ${nextType}`
        });
        continue;
      }
      const prevEmpty = prev.empty ?? "fallback";
      const nextEmpty = next.empty ?? "fallback";
      const rank = (e) => e === "accept" ? 0 : e === "fallback" ? 1 : 2;
      let mergedEmpty = prevEmpty;
      if (hasOwn(next, "empty")) {
        const pr = rank(prevEmpty);
        const nr = rank(nextEmpty);
        if (nr > pr) {
          diags.push({
            level: "error",
            key,
            message: `empty behavior becomes stricter (${prevEmpty} -> ${nextEmpty})`
          });
          continue;
        }
        if (nr < pr) {
          diags.push({
            level: "warning",
            key,
            message: `empty behavior becomes looser (${prevEmpty} -> ${nextEmpty})`
          });
          mergedEmpty = prevEmpty;
        } else {
          mergedEmpty = nextEmpty;
        }
      }
      if (prev.options || next.options) {
        const prevOptions = prev.options;
        const nextOptions = next.options;
        if (!isSupersetOptions(nextOptions, prevOptions)) {
          diags.push({
            level: "error",
            key,
            message: `enum options become stricter (subset)`
          });
          continue;
        }
        if (!isSupersetOptions(prevOptions, nextOptions)) {
          diags.push({
            level: "warning",
            key,
            message: `enum options widened (superset)`
          });
        }
      }
      if (prev.range || next.range) {
        const prevRange = prev.range;
        const nextRange = next.range;
        if (rangeNarrower(nextRange, prevRange)) {
          diags.push({
            level: "error",
            key,
            message: `range becomes stricter (narrower)`
          });
          continue;
        }
        if (prevRange && nextRange && !rangeWider(prevRange, nextRange)) {
          diags.push({
            level: "warning",
            key,
            message: `range widened`
          });
        }
      }
      const prevHasValidator = !!prev.validator;
      const nextHasValidator = !!next.validator;
      if (!prevHasValidator && nextHasValidator) {} else if (prevHasValidator && !nextHasValidator) {
        diags.push({
          level: "error",
          key,
          message: `validator removal is disallowed in merge`
        });
        continue;
      } else if (prevHasValidator && nextHasValidator && prev.validator !== next.validator) {
        diags.push({
          level: "error",
          key,
          message: `validator replacement is disallowed in merge`
        });
        continue;
      }
      const hasPrevDefault = "default" in prev;
      const hasNextDefault = "default" in next;
      let mergedDefault = prev.default;
      if (!hasPrevDefault && hasNextDefault) {
        mergedDefault = next.default;
      } else if (hasPrevDefault && hasNextDefault && prev.default !== next.default) {
        diags.push({
          level: "warning",
          key,
          message: `default overridden in define(); prefer setDefaults()`
        });
        mergedDefault = prev.default;
      }
      const merged = {
        ...prev,
        ...next,
        type: prev.type,
        empty: mergedEmpty,
        options: next.options ?? prev.options,
        range: next.range ?? prev.range,
        validator: prev.validator ?? next.validator
      };
      if (hasPrevDefault || hasNextDefault) {
        merged.default = mergedDefault;
      }
      out[key] = merged;
    }
    return { specs: out, diags };
  }

  // ../packages/modules/props/src/kernel/kernel.ts
  var hasOwn2 = (obj, key) => Object.prototype.hasOwnProperty.call(obj, key);
  function shallowFreeze(o) {
    return Object.freeze({ ...o });
  }
  function objectIs(a, b) {
    return Object.is(a, b);
  }
  function diffKeys(prev, next, keys) {
    const changed = [];
    for (const k of keys) {
      if (!objectIs(prev[k], next[k]))
        changed.push(k);
    }
    return changed;
  }

  class PropsKernel {
    specs = {};
    defaultStack = [];
    prevValid = {};
    raw = Object.freeze({});
    resolved = Object.freeze({});
    diags = [];
    hydrated = false;
    getDiagnostics() {
      return this.diags;
    }
    define(input) {
      const { specs, diags } = mergeSpecs(this.specs, input);
      const hasError = diags.some((d) => d.level === "error");
      if (hasError) {
        const msg = diags.filter((d) => d.level === "error").map((d) => d.key ? `${d.key}: ${d.message}` : d.message).join("; ");
        throw new Error(`[Props] define merge error: ${msg}`);
      }
      for (const d of diags) {
        if (d.level === "warning") {
          this.diags.push({ level: "warning", key: d.key, message: d.message });
        }
      }
      this.specs = specs;
      this.recomputeResolved();
    }
    setDefaults(partial) {
      const declKeys = Object.keys(this.specs);
      if (declKeys.length === 0) {
        const keys = Object.keys(partial);
        if (keys.length > 0) {
          throw new Error(`[Props] setDefaults() rejects keys not in specs: ${keys.join(", ")}`);
        }
      } else {
        for (const k of Object.keys(partial)) {
          if (!hasOwn2(this.specs, k)) {
            throw new Error(`[Props] setDefaults() rejects key not in specs: ${k}`);
          }
          if (!isJsonPropsValue(partial[k])) {
            throw new Error(`[Props] setDefaults() rejects non-JSON props value for key: ${k}`);
          }
        }
      }
      this.defaultStack.unshift({ ...partial });
      this.recomputeResolved();
    }
    get() {
      return this.resolved;
    }
    getRaw() {
      return this.raw;
    }
    isProvided(key) {
      return hasOwn2(this.raw, key);
    }
    applyRaw(nextRawInput) {
      const prevRaw = this.raw;
      const prevResolved = this.resolved;
      const normalized = this.normalizeRawInput(nextRawInput ?? {});
      const nextRaw = shallowFreeze(normalized);
      this.raw = nextRaw;
      const { snapshot: nextResolved, meta } = this.resolve(nextRaw, true);
      this.resolved = nextResolved;
      if (!this.hydrated) {
        this.hydrated = true;
        return { meta };
      }
      const declKeys = Object.keys(this.specs);
      const changedAllResolved = diffKeys(prevResolved, nextResolved, declKeys);
      const unionKeys = Array.from(new Set([...Object.keys(prevRaw), ...Object.keys(nextRaw)]));
      const changedAllRaw = diffKeys(prevRaw, nextRaw, unionKeys);
      if (changedAllResolved.length === 0 && changedAllRaw.length === 0) {
        return { meta };
      }
      return {
        meta,
        report: {
          prevRaw,
          nextRaw,
          prevResolved,
          nextResolved,
          changedAllResolved,
          changedAllRaw,
          meta
        }
      };
    }
    recomputeResolved() {
      const { snapshot } = this.resolve(this.raw, false);
      this.resolved = snapshot;
    }
    resolve(raw, strict) {
      const out = {};
      const providedKeys = [];
      const emptyKeys = [];
      const invalidKeys = [];
      const usedFallbackKeys = [];
      const acceptedEmptyKeys = [];
      const declKeys = Object.keys(this.specs);
      for (const k of declKeys) {
        const decl = this.specs[k];
        const provided = Object.prototype.hasOwnProperty.call(raw, k);
        if (provided)
          providedKeys.push(k);
        const eb = decl.empty ?? "fallback";
        const rawVal = provided ? raw[k] : undefined;
        const isProvidedEmpty = provided && (rawVal === null || rawVal === undefined);
        const isMissing = !provided;
        if (isMissing) {
          const requireNonEmpty = strict && eb === "error";
          const fb2 = this.pickFallbackMissingOnly(k, decl, requireNonEmpty);
          if (!fb2.ok) {
            throw new Error(`[Props] prop "${k}" is missing and empty="error" has no non-empty fallback.`);
          }
          out[k] = fb2.value;
          if (fb2.usedDefault)
            usedFallbackKeys.push(k);
          continue;
        }
        if (isProvidedEmpty) {
          emptyKeys.push(k);
          if (eb === "accept") {
            out[k] = null;
            acceptedEmptyKeys.push(k);
            continue;
          }
          const mode2 = strict && eb === "error" ? "non-empty" : "any";
          const fb2 = this.pickFallbackProvidedUnusable(k, decl, mode2);
          if (!fb2.ok) {
            throw new Error(`[Props] prop "${k}" is empty (null/undefined) and empty="error" has no non-empty fallback.`);
          }
          out[k] = fb2.value;
          if (fb2.usedDefault)
            usedFallbackKeys.push(k);
          if (fb2.isNonEmpty)
            this.prevValid[k] = fb2.value;
          continue;
        }
        const valid = this.validateNonEmptyValue(rawVal, decl);
        if (valid.ok) {
          out[k] = valid.value;
          this.prevValid[k] = valid.value;
          continue;
        }
        invalidKeys.push(k);
        const mode = strict && eb === "error" ? "non-empty" : "any";
        const fb = this.pickFallbackProvidedUnusable(k, decl, mode);
        if (!fb.ok) {
          throw new Error(`[Props] prop "${k}" is invalid and empty="error" has no non-empty fallback.`);
        }
        out[k] = fb.value;
        if (fb.usedDefault)
          usedFallbackKeys.push(k);
        if (fb.isNonEmpty)
          this.prevValid[k] = fb.value;
      }
      return {
        snapshot: shallowFreeze(out),
        meta: {
          providedKeys,
          emptyKeys,
          invalidKeys,
          usedFallbackKeys,
          acceptedEmptyKeys
        }
      };
    }
    validateNonEmptyValue(v, decl) {
      if (!isJsonPropsValue(v))
        return { ok: false };
      switch (decl.type) {
        case "boolean":
          if (typeof v !== "boolean")
            return { ok: false };
          break;
        case "string":
          if (typeof v !== "string")
            return { ok: false };
          break;
        case "enum":
          if (typeof v !== "string")
            return { ok: false };
          if (!Array.isArray(decl.options) || !decl.options.includes(v)) {
            return { ok: false };
          }
          break;
        case "number":
          if (typeof v !== "number" || Number.isNaN(v))
            return { ok: false };
          break;
        case "object":
          if (typeof v !== "object")
            return { ok: false };
          break;
        case "any":
        default:
          break;
      }
      if (decl.range) {
        if (typeof v !== "number")
          return { ok: false };
        const min = decl.range.min ?? -Infinity;
        const max = decl.range.max ?? Infinity;
        if (v < min || v > max)
          return { ok: false };
      }
      if (decl.validator) {
        try {
          if (!decl.validator(v))
            return { ok: false };
        } catch {
          return { ok: false };
        }
      }
      return { ok: true, value: v };
    }
    pickFallbackMissingOnly(key, decl, requireNonEmpty) {
      const acceptAny = (v) => v !== undefined;
      const acceptNonEmpty = (v) => v !== null && v !== undefined;
      const accept = requireNonEmpty ? acceptNonEmpty : acceptAny;
      const tryTake = (v, usedDefault) => {
        if (!accept(v))
          return { ok: false, usedDefault, isNonEmpty: false };
        if (v === null || v === undefined) {
          return { ok: true, value: null, usedDefault, isNonEmpty: false };
        }
        const valid = this.validateNonEmptyValue(v, decl);
        if (!valid.ok)
          return { ok: false, usedDefault, isNonEmpty: false };
        return { ok: true, value: valid.value, usedDefault, isNonEmpty: true };
      };
      for (const layer of this.defaultStack) {
        if (hasOwn2(layer, key)) {
          const r = tryTake(layer[key], true);
          if (r.ok)
            return r;
        }
      }
      if (hasOwn2(decl, "default")) {
        const r = tryTake(decl.default, true);
        if (r.ok)
          return r;
      }
      if (!requireNonEmpty) {
        return { ok: true, value: null, usedDefault: true, isNonEmpty: false };
      }
      return { ok: false, usedDefault: false, isNonEmpty: false };
    }
    pickFallbackProvidedUnusable(key, decl, mode) {
      const acceptAny = (v) => v !== undefined;
      const acceptNonEmpty = (v) => v !== null && v !== undefined;
      const accept = mode === "non-empty" ? acceptNonEmpty : acceptAny;
      const tryTake = (v, usedDefault) => {
        if (!accept(v))
          return { ok: false, usedDefault, isNonEmpty: false };
        if (v === null || v === undefined) {
          return { ok: true, value: null, usedDefault, isNonEmpty: false };
        }
        const valid = this.validateNonEmptyValue(v, decl);
        if (!valid.ok)
          return { ok: false, usedDefault, isNonEmpty: false };
        return { ok: true, value: valid.value, usedDefault, isNonEmpty: true };
      };
      if (hasOwn2(this.prevValid, key)) {
        const v = this.prevValid[key];
        const r = tryTake(v, false);
        if (r.ok)
          return r;
      }
      for (const layer of this.defaultStack) {
        if (hasOwn2(layer, key)) {
          const r = tryTake(layer[key], true);
          if (r.ok)
            return r;
        }
      }
      if (hasOwn2(decl, "default")) {
        const r = tryTake(decl.default, true);
        if (r.ok)
          return r;
      }
      if (mode === "any") {
        return { ok: true, value: null, usedDefault: true, isNonEmpty: false };
      }
      return { ok: false, usedDefault: false, isNonEmpty: false };
    }
    dispose() {
      this.hydrated = false;
      this.defaultStack = [];
      this.prevValid = {};
      this.raw = Object.freeze({});
      this.resolved = Object.freeze({});
    }
    normalizeRawInput(input) {
      const out = {};
      for (const k of Object.keys(input)) {
        const v = input[k];
        out[k] = v === undefined ? null : v;
      }
      return out;
    }
  }

  // ../packages/modules/props/src/impl.ts
  function objectIs2(a, b) {
    return Object.is(a, b);
  }
  function diffKeys2(prev, next, keys) {
    const changed = [];
    for (const k of keys) {
      if (!objectIs2(prev[k], next[k]))
        changed.push(k);
    }
    return changed;
  }

  class PropsModuleImpl extends ModuleBase {
    kernel = new PropsKernel;
    rawDirty = true;
    subscribed = false;
    unsubRaw;
    lastSource;
    prototypeName;
    declaredKeys = new Set;
    resolvedWatchSeq = 0;
    watch = [];
    watchAll = [];
    watchRaw = [];
    watchRawAll = [];
    implDiags = [];
    pendingReport = null;
    constructor(caps3, prototypeName) {
      super(caps3);
      this.prototypeName = prototypeName;
    }
    define(decl) {
      this.guardSetupOnly("def.props.define");
      this.kernel.define(decl);
      for (const k of Object.keys(decl))
        this.declaredKeys.add(k);
    }
    setDefaults(partialDefaults) {
      this.guardSetupOnly("def.props.setDefaults");
      this.kernel.setDefaults(partialDefaults);
    }
    watchKeys(keys, cb) {
      this.guardSetupOnly("def.props.watch");
      if (!Array.isArray(keys) || keys.length === 0) {
        throw new Error(`[Props] watch(keys) requires non-empty declared keys. Use watchAll() instead.`);
      }
      if (this.declaredKeys.size === 0) {
        throw new Error(`[Props] watch(keys) requires props to be declared first (define()).`);
      }
      for (const k of keys) {
        if (!this.declaredKeys.has(k)) {
          throw new Error(`[Props] watch(keys) only allows declared keys. Undeclared: ${k}`);
        }
      }
      const entry = { order: this.resolvedWatchSeq++, keys: [...keys], cb, active: true };
      this.watch.push(entry);
      return () => {
        entry.active = false;
      };
    }
    watchAllKeys(cb) {
      this.guardSetupOnly("def.props.watchAll");
      const entry = { order: this.resolvedWatchSeq++, cb, active: true };
      this.watchAll.push(entry);
      return () => {
        entry.active = false;
      };
    }
    watchRawKeys(keys, cb, devWarn = true) {
      this.guardSetupOnly("def.props.watchRaw");
      if (!Array.isArray(keys) || keys.length === 0) {
        throw new Error(`[Props] watchRaw(keys) requires non-empty keys. Use watchRawAll() instead.`);
      }
      const entry = { keys: [...keys], cb, devWarn, active: true };
      this.watchRaw.push(entry);
      return () => {
        entry.active = false;
      };
    }
    watchRawAllKeys(cb, devWarn = true) {
      this.guardSetupOnly("def.props.watchRawAll");
      const entry = { cb, devWarn, active: true };
      this.watchRawAll.push(entry);
      return () => {
        entry.active = false;
      };
    }
    get() {
      return this.kernel.get();
    }
    getRaw() {
      return this.kernel.getRaw();
    }
    isProvided(key) {
      return this.kernel.isProvided(key);
    }
    syncFromHost() {
      this.ensureRawPropsSubscription();
      if (!this.caps.has(RAW_PROPS_SOURCE_CAP))
        return;
      const src = this.caps.get(RAW_PROPS_SOURCE_CAP);
      if (!src)
        return;
      if (this.lastSource !== src)
        this.rawDirty = true;
      if (!this.rawDirty)
        return;
      const raw = src.get();
      this.applyRaw(raw);
      this.rawDirty = false;
    }
    applyRaw(nextRaw) {
      const { report } = this.kernel.applyRaw({ ...nextRaw ?? {} });
      if (report)
        this.mergePending(report);
      this.rawDirty = false;
    }
    consumeTasks() {
      const p = this.pendingReport;
      if (!p)
        return [];
      this.pendingReport = null;
      const tasks = [];
      const prevResolved = p.prevResolved;
      const nextResolved = p.nextResolved;
      const prevRaw = p.prevRaw;
      const nextRaw = p.nextRaw;
      const declKeys = Object.keys(nextResolved);
      const changedAllResolved = diffKeys2(prevResolved, nextResolved, declKeys);
      const unionKeys = Array.from(new Set([...Object.keys(prevRaw), ...Object.keys(nextRaw)]));
      const changedAllRaw = diffKeys2(prevRaw, nextRaw, unionKeys);
      for (const w of this.watchRawAll) {
        if (!w.active)
          continue;
        if (changedAllRaw.length === 0)
          continue;
        if (w.devWarn) {
          this.implDiags.push({
            level: "warning",
            message: `[Props] watchRawAll() is an escape hatch; avoid in official prototypes.`
          });
        }
        const info = {
          changedKeysAll: changedAllRaw,
          changedKeysMatched: changedAllRaw
        };
        tasks.push({
          kind: "raw",
          cb: w.cb,
          next: nextRaw,
          prev: prevRaw,
          info
        });
      }
      for (const w of this.watchRaw) {
        if (!w.active)
          continue;
        if (changedAllRaw.length === 0)
          continue;
        if (w.devWarn) {
          this.implDiags.push({
            level: "warning",
            message: `[Props] watchRaw() is an escape hatch; avoid in official prototypes.`
          });
        }
        const matched = diffKeys2(prevRaw, nextRaw, w.keys);
        if (matched.length === 0)
          continue;
        const info = {
          changedKeysAll: changedAllRaw,
          changedKeysMatched: matched
        };
        tasks.push({
          kind: "raw",
          cb: w.cb,
          next: nextRaw,
          prev: prevRaw,
          info
        });
      }
      const resolvedTasks = [];
      for (const w of this.watchAll) {
        if (!w.active)
          continue;
        if (changedAllResolved.length === 0)
          continue;
        const info = {
          changedKeysAll: changedAllResolved,
          changedKeysMatched: changedAllResolved
        };
        resolvedTasks.push({
          order: w.order,
          task: {
            kind: "resolved",
            cb: w.cb,
            next: nextResolved,
            prev: prevResolved,
            info
          }
        });
      }
      for (const w of this.watch) {
        if (!w.active)
          continue;
        if (changedAllResolved.length === 0)
          continue;
        const matched = diffKeys2(prevResolved, nextResolved, w.keys);
        if (matched.length === 0)
          continue;
        const info = {
          changedKeysAll: changedAllResolved,
          changedKeysMatched: matched
        };
        resolvedTasks.push({
          order: w.order,
          task: {
            kind: "resolved",
            cb: w.cb,
            next: nextResolved,
            prev: prevResolved,
            info
          }
        });
      }
      resolvedTasks.sort((a, b) => a.order - b.order);
      for (const { task } of resolvedTasks)
        tasks.push(task);
      return tasks;
    }
    getDiagnostics() {
      const base = this.kernel.getDiagnostics?.() ?? [];
      if (this.implDiags.length === 0)
        return base;
      return [...base, ...this.implDiags];
    }
    onProtoPhase(phase) {
      super.onProtoPhase(phase);
      if (phase === "unmounted") {
        this.dispose();
      }
    }
    onCapsEpoch(_epoch) {
      this.ensureRawPropsSubscription();
      this.rawDirty = true;
    }
    dispose() {
      this.unsubRaw?.();
      this.unsubRaw = undefined;
      this.subscribed = false;
      this.lastSource = undefined;
      this.rawDirty = true;
      this.pendingReport = null;
      this.kernel.dispose?.();
    }
    guardSetupOnly(op) {
      if (this.protoPhase !== "setup") {
        throw illegalPhase(op, this.protoPhase, {
          prototypeName: this.prototypeName,
          hint: `Use 'run' callbacks (created/mounted/updated) for runtime behavior; do not call def.* after setup.`
        });
      }
    }
    mergePending(r) {
      if (!this.pendingReport) {
        this.pendingReport = {
          prevRaw: r.prevRaw,
          prevResolved: r.prevResolved,
          nextRaw: r.nextRaw,
          nextResolved: r.nextResolved
        };
        return;
      }
      this.pendingReport.nextRaw = r.nextRaw;
      this.pendingReport.nextResolved = r.nextResolved;
    }
    ensureRawPropsSubscription() {
      const has = this.caps.has(RAW_PROPS_SOURCE_CAP);
      if (!has) {
        if (this.unsubRaw) {
          this.unsubRaw();
          this.unsubRaw = undefined;
        }
        this.subscribed = false;
        this.lastSource = undefined;
        return;
      }
      const src = this.caps.get(RAW_PROPS_SOURCE_CAP);
      if (!src) {
        if (this.unsubRaw) {
          this.unsubRaw();
          this.unsubRaw = undefined;
        }
        this.subscribed = false;
        this.lastSource = undefined;
        return;
      }
      if (this.subscribed && this.lastSource === src)
        return;
      if (this.unsubRaw) {
        this.unsubRaw();
        this.unsubRaw = undefined;
      }
      this.lastSource = src;
      this.subscribed = true;
      this.unsubRaw = src.subscribe(() => {
        this.rawDirty = true;
      });
    }
  }

  // ../packages/modules/props/src/create.ts
  function createPropsModule(ctx) {
    const { init, caps: caps3, deps } = ctx;
    return createModule({
      name: "props",
      scope: "instance",
      init,
      caps: caps3,
      deps,
      build: ({ init: init2, caps: caps4 }) => {
        const impl = new PropsModuleImpl(caps4, init2.prototypeName);
        return {
          facade: {
            define: (decl) => impl.define(decl),
            setDefaults: (partial) => impl.setDefaults(partial),
            watch: (keys, cb) => impl.watchKeys(keys, cb),
            watchAll: (cb) => impl.watchAllKeys(cb),
            watchRaw: (keys, cb) => impl.watchRawKeys(keys, cb, true),
            watchRawAll: (cb) => impl.watchRawAllKeys(cb, true),
            get: () => impl.get(),
            getRaw: () => impl.getRaw(),
            isProvided: (key) => impl.isProvided(key)
          },
          hooks: {
            onProtoPhase: (p) => impl.onProtoPhase(p),
            dispose: () => impl.dispose()
          },
          port: {
            syncFromHost: () => impl.syncFromHost(),
            applyRaw: (nextRaw) => impl.applyRaw(nextRaw),
            consumeTasks: () => impl.consumeTasks(),
            getDiagnostics: () => impl.getDiagnostics()
          }
        };
      }
    });
  }
  var PropsModuleDef = defineModule({
    name: "props",
    resourceOwnership: "instance",
    deps: [],
    create: createPropsModule
  });
  // ../packages/modules/event/src/kernel.ts
  function isPlainObject(x) {
    return !!x && typeof x === "object" && (x.constructor === Object || x.constructor == null);
  }
  function sameOptions(a, b) {
    if (Object.is(a, b))
      return true;
    if (a == null || b == null)
      return false;
    if (typeof a !== "object" || typeof b !== "object")
      return false;
    if (isPlainObject(a) && isPlainObject(b)) {
      const ak = Object.keys(a);
      const bk = Object.keys(b);
      if (ak.length !== bk.length)
        return false;
      for (const k of ak) {
        if (!Object.prototype.hasOwnProperty.call(b, k))
          return false;
        if (!Object.is(a[k], b[k]))
          return false;
      }
      return true;
    }
    return false;
  }
  function matchReg(r, kind, type, options) {
    return r.kind === kind && r.type === type && sameOptions(r.options, options);
  }

  class EventKernel {
    regs = [];
    seq = 0;
    nextId() {
      this.seq++;
      return `ev_${this.seq}`;
    }
    on(kind, type, options) {
      const id = this.nextId();
      this.regs.push({ id, kind, type, options });
      return id;
    }
    offById(id) {
      for (let i = this.regs.length - 1;i >= 0; i--) {
        const r = this.regs[i];
        if (r.id !== id)
          continue;
        if (r.wrapper && r.boundTarget) {
          r.deactivate?.();
          r.boundTarget.removeEventListener(r.type, r.wrapper, r.options);
        }
        this.regs.splice(i, 1);
        return true;
      }
      return false;
    }
    offLatest(kind, type, options) {
      for (let i = this.regs.length - 1;i >= 0; i--) {
        const r = this.regs[i];
        if (!matchReg(r, kind, type, options))
          continue;
        if (r.wrapper && r.boundTarget) {
          r.deactivate?.();
          r.boundTarget.removeEventListener(r.type, r.wrapper, r.options);
        }
        this.regs.splice(i, 1);
        return true;
      }
      return false;
    }
    setLabel(id, label) {
      for (let i = this.regs.length - 1;i >= 0; i--) {
        const r = this.regs[i];
        if (r.id !== id)
          continue;
        r.debugLabel = label;
        return true;
      }
      return false;
    }
    bindAll(dispatch, getTarget) {
      const pending = [];
      for (const r of this.regs) {
        if (r.wrapper && r.boundTarget)
          continue;
        const target = getTarget(r.kind, r.type);
        let active = true;
        const wrapper = (event2) => {
          if (!active)
            return;
          dispatch(r.id, event2, r.type);
        };
        const deactivate = () => {
          active = false;
        };
        pending.push({ registration: r, target, wrapper, deactivate });
      }
      const attached = [];
      try {
        for (const { registration, target, wrapper, deactivate } of pending) {
          target.addEventListener(registration.type, wrapper, registration.options);
          attached.push({ registration, target, wrapper, deactivate });
          registration.wrapper = wrapper;
          registration.deactivate = deactivate;
          registration.boundTarget = target;
        }
      } catch (error2) {
        for (const { registration, target, wrapper, deactivate } of attached) {
          deactivate();
          try {
            target.removeEventListener(registration.type, wrapper, registration.options);
          } catch {}
          registration.wrapper = undefined;
          registration.deactivate = undefined;
          registration.boundTarget = undefined;
        }
        throw error2;
      }
    }
    unbindAll() {
      for (const r of this.regs) {
        if (!r.wrapper || !r.boundTarget)
          continue;
        const target = r.boundTarget;
        const wrapper = r.wrapper;
        r.deactivate?.();
        r.wrapper = undefined;
        r.deactivate = undefined;
        r.boundTarget = undefined;
        target.removeEventListener(r.type, wrapper, r.options);
      }
    }
    cleanupAll() {
      this.unbindAll();
      this.regs.length = 0;
    }
    snapshot() {
      return this.regs.map((r) => ({
        id: r.id,
        kind: r.kind,
        type: String(r.type),
        bound: !!r.boundTarget && !!r.wrapper,
        label: r.debugLabel
      }));
    }
    hasAny(kind) {
      return this.regs.some((r) => r.kind === kind);
    }
    hasAnyAtAll() {
      return this.regs.length > 0;
    }
  }

  // ../packages/modules/expose-event/src/caps.ts
  var EXPOSE_EVENT_SINK_CAP = cap("@proto.ui/event/emit");
  // ../packages/modules/expose-event/src/error.ts
  class ExposeEventError extends Error {
    code;
    detail;
    constructor(code, message, detail) {
      super(message);
      this.name = "ExposeEventError";
      this.code = code;
      this.detail = detail;
    }
  }
  function exposeEventInvalidArgument(message, detail) {
    return new ExposeEventError("EXPOSE_EVENT_INVALID_ARGUMENT", message, detail);
  }

  // ../packages/modules/expose-event/src/impl.ts
  class ExposeEventModuleImpl extends ModuleBase {
    expose;
    prototypeName;
    constructor(caps4, deps, prototypeName) {
      super(caps4);
      this.expose = deps.requirePort("expose");
      this.prototypeName = prototypeName;
    }
    registerExposeEvent(key, _spec) {
      this.sys?.ensureSetup("def.expose.event");
      this.ensureValidKey(key, "def.expose.event");
      const declaration = this.expose.get(key);
      if (!isExposeEventDeclaration(declaration)) {
        throw exposeEventInvalidArgument(`[ExposeEvent] key is not registered as an expose.event declaration: ${key}`, { prototypeName: this.prototypeName, key });
      }
    }
    emit(key, payload, options) {
      this.sys?.ensureRuntime("rt.expose.emit");
      this.ensureValidKey(key, "rt.expose.emit");
      const declaration = this.expose.get(key);
      if (!isExposeEventDeclaration(declaration)) {
        throw exposeEventInvalidArgument(`[ExposeEvent] emit for unregistered expose.event key: ${key}`, { prototypeName: this.prototypeName, key });
      }
      if (!this.caps.has(EXPOSE_EVENT_SINK_CAP))
        return;
      const sink = this.caps.get(EXPOSE_EVENT_SINK_CAP);
      if (!sink)
        return;
      try {
        sink(key, payload, options);
      } catch {}
    }
    facade = {
      registerExposeEvent: (key, spec2) => this.registerExposeEvent(key, spec2),
      emit: (key, payload, options) => this.emit(key, payload, options)
    };
    ensureValidKey(key, op) {
      if (typeof key === "string" && key.length > 0)
        return;
      throw exposeEventInvalidArgument(`[ExposeEvent] ${op} requires a non-empty string key.`, {
        prototypeName: this.prototypeName,
        key
      });
    }
  }

  // ../packages/modules/expose-event/src/create.ts
  function createExposeEventModule(ctx) {
    return createModule({
      name: "expose-event",
      scope: "instance",
      init: ctx.init,
      caps: ctx.caps,
      deps: ctx.deps,
      build: ({ init, caps: caps4, deps }) => {
        const impl = new ExposeEventModuleImpl(caps4, deps, init.prototypeName);
        return { facade: impl.facade };
      }
    });
  }
  var ExposeEventModuleDef = defineModule({
    name: "expose-event",
    resourceOwnership: "instance",
    deps: ["expose"],
    create: createExposeEventModule
  });
  // ../packages/modules/event/src/caps.ts
  var EVENT_ROOT_TARGET_CAP = cap("@proto.ui/event/getRootTarget");
  var EVENT_GLOBAL_TARGET_CAP = cap("@proto.ui/event/getGlobalTarget");
  var EVENT_CANCEL_DEFAULT_ACTION_CAP = cap("@proto.ui/event/cancelDefaultAction");

  // ../packages/modules/event/src/error.ts
  class EventError extends Error {
    code;
    detail;
    constructor(code, message, detail) {
      super(message);
      this.name = "EventError";
      this.code = code;
      this.detail = detail;
    }
  }
  function eventInvalidArg(message, detail) {
    return new EventError("EVENT_INVALID_ARGUMENT", message, detail);
  }
  function eventTargetUnavailable(message, detail) {
    return new EventError("EVENT_TARGET_UNAVAILABLE", message, detail);
  }

  // ../packages/modules/event/src/impl.ts
  var CORE_EVENT_TYPES = [
    "press.start",
    "press.end",
    "press.cancel",
    "press.commit",
    "key.down",
    "key.up"
  ];
  var OPTIONAL_EVENT_TYPES = [
    "pointer.down",
    "pointer.move",
    "pointer.up",
    "pointer.cancel",
    "pointer.enter",
    "pointer.leave",
    "nav.focus",
    "nav.blur",
    "text.focus",
    "text.blur",
    "input",
    "change",
    "context.menu"
  ];
  var PORTABLE_EVENT_FIELDS = [
    "key",
    "ctrlKey",
    "metaKey",
    "altKey",
    "shiftKey",
    "repeat"
  ];
  function eventDataSource(raw) {
    if (!raw || typeof raw !== "object")
      return {};
    const detail = raw.detail;
    if (detail && typeof detail === "object")
      return detail;
    return raw;
  }
  function createPortableEventPayload(type, raw, control) {
    const source = eventDataSource(raw);
    const payload = { type, control };
    for (const field of PORTABLE_EVENT_FIELDS) {
      const value = source[field];
      if (typeof value === "string" || typeof value === "boolean")
        payload[field] = value;
    }
    return Object.freeze(payload);
  }
  function isValidEventType(type) {
    if (typeof type !== "string" || !type)
      return false;
    if (CORE_EVENT_TYPES.includes(type))
      return true;
    if (OPTIONAL_EVENT_TYPES.includes(type))
      return true;
    if (type.startsWith("host:"))
      return type.length > "host:".length;
    return false;
  }
  function isEventTargetLike(x) {
    return !!x && (typeof x === "object" || typeof x === "function") && typeof x.addEventListener === "function" && typeof x.removeEventListener === "function";
  }

  class EventModuleImpl extends ModuleBase {
    kernel = new EventKernel;
    prototypeName;
    overriddenRootTarget = null;
    overriddenSemanticRootTarget = null;
    lastDispatch = null;
    isBound = false;
    internalCallbacks = new Map;
    constructor(caps5, prototypeName) {
      super(caps5);
      this.prototypeName = prototypeName;
    }
    ensureSetup(op) {
      this.sys?.ensureSetup(op);
      if (!this.sys && this.protoPhase !== "setup") {
        throw illegalPhase(op, this.protoPhase, {
          prototypeName: this.prototypeName
        });
      }
    }
    ensureRuntime(op) {
      this.sys?.ensureRuntime(op);
    }
    makeToken(id, kind, type, options) {
      const meta = {
        kind,
        type: String(type),
        options,
        label: undefined
      };
      const token = {
        id,
        meta,
        [Symbol.for("__eventTokenBrand")]: "EventListenerToken",
        desc: (text) => {
          this.ensureSetup("def.event.token.desc");
          const __DEV__ = true;
          if (__DEV__) {
            const t = typeof text === "string" ? text.trim() : "";
            if (t) {
              meta.label = t;
              this.kernel.setLabel(id, t);
            }
          }
          return token;
        }
      };
      return token;
    }
    redirectRoot(target) {
      this.ensureSetup("event.port.redirectRoot");
      if (!isEventTargetLike(target)) {
        throw eventInvalidArg(`[Event] redirectRoot() requires an EventTarget-like object.`, {
          prototypeName: this.prototypeName,
          target
        });
      }
      this.overriddenRootTarget = target;
    }
    redirectSemanticRoot(target) {
      this.ensureSetup("event.port.redirectSemanticRoot");
      if (!isEventTargetLike(target)) {
        throw eventInvalidArg(`[Event] redirectSemanticRoot() requires an EventTarget-like object.`, {
          prototypeName: this.prototypeName,
          target
        });
      }
      this.overriddenSemanticRootTarget = target;
    }
    on(type, options) {
      this.ensureSetup("def.event.on");
      this.guardArgs(type);
      this.guardListenerOptions(type, options);
      const id = this.kernel.on("root", type, options);
      return this.makeToken(id, "root", type, options);
    }
    onGlobal(type, options) {
      this.ensureSetup("def.event.onGlobal");
      this.guardArgs(type);
      this.guardListenerOptions(type, options);
      const id = this.kernel.on("global", type, options);
      return this.makeToken(id, "global", type, options);
    }
    onInternal(type, cb, options) {
      this.ensureSetup("event.port.on");
      this.guardArgs(type);
      this.guardListenerOptions(type, options);
      if (typeof cb !== "function") {
        throw eventInvalidArg(`[Event] internal listener requires a callback.`, {
          prototypeName: this.prototypeName,
          type
        });
      }
      const id = this.kernel.on("root", type, options);
      this.internalCallbacks.set(id, cb);
      return this.makeToken(id, "root", type, options);
    }
    onGlobalInternal(type, cb, options) {
      this.ensureSetup("event.port.onGlobal");
      this.guardArgs(type);
      this.guardListenerOptions(type, options);
      if (typeof cb !== "function") {
        throw eventInvalidArg(`[Event] internal global listener requires a callback.`, {
          prototypeName: this.prototypeName,
          type
        });
      }
      const id = this.kernel.on("global", type, options);
      this.internalCallbacks.set(id, cb);
      return this.makeToken(id, "global", type, options);
    }
    off(token) {
      this.ensureSetup("def.event.off");
      const id = token?.id;
      if (typeof id !== "string" || !id) {
        throw eventInvalidArg(`[Event] invalid token.`, {
          prototypeName: this.prototypeName,
          token
        });
      }
      this.internalCallbacks.delete(id);
      this.kernel.offById(id);
    }
    bind(dispatch) {
      this.ensureRuntime("rt.event.bind");
      const needsRoot = this.kernel.hasAny("root");
      const needsGlobal = this.kernel.hasAny("global");
      if (!needsRoot && !needsGlobal)
        return;
      const rootGetter = this.caps.has(EVENT_ROOT_TARGET_CAP) ? this.caps.get(EVENT_ROOT_TARGET_CAP) : undefined;
      const globalGetter = this.caps.has(EVENT_GLOBAL_TARGET_CAP) ? this.caps.get(EVENT_GLOBAL_TARGET_CAP) : undefined;
      const root = needsRoot ? rootGetter?.() ?? null : null;
      const global = needsGlobal ? globalGetter?.() ?? null : null;
      if (needsGlobal && !global) {
        throw eventTargetUnavailable(`[Event] global target unavailable during bind().`, {
          prototypeName: this.prototypeName
        });
      }
      this.lastDispatch = dispatch;
      const preventedSamples = new WeakSet;
      const dispatchWithControl = (id, raw, type) => {
        if (!type)
          throw eventInvalidArg("[Event] dispatch requires a registered event type.");
        if (String(type).startsWith("host:")) {
          dispatch(id, raw);
          return;
        }
        let active = true;
        const control = Object.freeze({
          requestDefaultActionPrevention: (options) => {
            if (!active) {
              throw eventInvalidArg("[Event] default-action control is outside its callback window.");
            }
            if (raw && typeof raw === "object") {
              if (preventedSamples.has(raw))
                return;
              preventedSamples.add(raw);
            }
            this.requestDefaultActionPrevented(raw, options);
          }
        });
        const payload = createPortableEventPayload(type, raw, control);
        try {
          dispatch(id, payload);
        } finally {
          active = false;
        }
      };
      this.kernel.bindAll(dispatchWithControl, (kind, type) => {
        if (kind === "global")
          return global;
        const target = this.overriddenRootTarget ?? (String(type).startsWith("host:") ? null : this.overriddenSemanticRootTarget) ?? root;
        if (!target) {
          throw eventTargetUnavailable(`[Event] root target unavailable during bind().`, {
            prototypeName: this.prototypeName,
            type
          });
        }
        return target;
      });
      this.isBound = true;
    }
    unbind() {
      this.ensureRuntime("rt.event.unbind");
      this.kernel.unbindAll();
      this.isBound = false;
    }
    getDiagnostics() {
      return this.kernel.snapshot();
    }
    requestDefaultActionPrevented(ev, options) {
      const detail = ev?.detail ?? ev;
      if (typeof detail?.requestDefaultPrevented === "function") {
        detail.requestDefaultPrevented(options);
        return;
      }
      const nativeEvent = detail?.nativeEvent ?? ev?.nativeEvent ?? ev;
      if (this.caps.has(EVENT_CANCEL_DEFAULT_ACTION_CAP)) {
        const cancel = this.caps.get(EVENT_CANCEL_DEFAULT_ACTION_CAP);
        cancel?.({
          event: nativeEvent,
          reason: options?.reason,
          source: options?.source
        });
        return;
      }
      if (typeof detail?.preventDefault === "function") {
        detail.preventDefault();
        return;
      }
      if (typeof nativeEvent?.preventDefault === "function") {
        nativeEvent.preventDefault();
      }
    }
    dispatchInternal(id, ev) {
      const cb = this.internalCallbacks.get(id);
      if (!cb)
        return;
      cb(ev);
    }
    onProtoPhase(phase) {
      super.onProtoPhase(phase);
      if (phase === "unmounted") {
        this.kernel.cleanupAll();
        this.lastDispatch = null;
        this.isBound = false;
        this.overriddenRootTarget = null;
        this.overriddenSemanticRootTarget = null;
        this.internalCallbacks.clear();
      }
    }
    onCapsEpoch(_epoch) {
      if (this.caps.has(EXPOSE_EVENT_SINK_CAP)) {
        throw eventInvalidArg(`[Event] EXPOSE_EVENT_SINK_CAP must be wired to the expose-event module, not event.`, { prototypeName: this.prototypeName, targetModule: "expose-event" });
      }
      if (!this.isBound)
        return;
      if (!this.lastDispatch)
        return;
      this.kernel.unbindAll();
      this.isBound = false;
      this.bind(this.lastDispatch);
    }
    guardListenerOptions(type, options) {
      if (!String(type).startsWith("host:") && typeof options !== "undefined") {
        throw eventInvalidArg("[Event] listener options are only valid for host:* extensions.", {
          prototypeName: this.prototypeName,
          type
        });
      }
    }
    guardArgs(type) {
      if (!isValidEventType(type)) {
        throw eventInvalidArg(`[Event] invalid event type: ${String(type)}`, {
          prototypeName: this.prototypeName,
          type
        });
      }
    }
  }

  // ../packages/modules/event/src/create.ts
  function createEventModule(ctx) {
    const { init, caps: caps5, deps } = ctx;
    return createModule({
      name: "event",
      scope: "instance",
      init,
      caps: caps5,
      deps,
      build: ({ init: init2, caps: caps6 }) => {
        const impl = new EventModuleImpl(caps6, init2.prototypeName);
        return {
          facade: {
            on: (type, options) => impl.on(type, options),
            onGlobal: (type, options) => impl.onGlobal(type, options),
            off: (token) => impl.off(token)
          },
          hooks: {
            onProtoPhase: (p) => impl.onProtoPhase(p)
          },
          port: {
            on: (type, cb, options) => impl.onInternal(type, cb, options),
            onGlobal: (type, cb, options) => impl.onGlobalInternal(type, cb, options),
            bind: (dispatch) => impl.bind(dispatch),
            unbind: () => impl.unbind(),
            getDiagnostics: () => impl.getDiagnostics(),
            requestDefaultActionPrevented: (ev, options) => impl.requestDefaultActionPrevented(ev, options),
            redirectRoot: (target) => impl.redirectRoot(target),
            redirectSemanticRoot: (target) => impl.redirectSemanticRoot(target),
            dispatchInternal: (id, ev) => impl.dispatchInternal(id, ev)
          }
        };
      }
    });
  }
  var EventModuleDef = defineModule({
    name: "event",
    resourceOwnership: "mixed",
    deps: [],
    create: createEventModule
  });
  // ../packages/modules/anatomy/src/caps.ts
  var ANATOMY_INSTANCE_TOKEN_CAP = cap("@proto.ui/anatomy/instanceToken");
  var ANATOMY_PARENT_CAP = cap("@proto.ui/anatomy/getParent");
  var ANATOMY_GET_PROTO_CAP = cap("@proto.ui/anatomy/getPrototype");
  var ANATOMY_ROOT_TARGET_CAP = cap("@proto.ui/anatomy/getRootTarget");
  var ANATOMY_ORDER_OBSERVER_CAP = cap("@proto.ui/anatomy/orderObserver");

  // ../packages/modules/anatomy/src/error.ts
  var ERR = {
    PHASE: "ANATOMY_PHASE_VIOLATION",
    CAP: "ANATOMY_CAP_UNAVAILABLE",
    FAMILY_INVALID: "ANATOMY_FAMILY_INVALID",
    CLAIM_INVALID: "ANATOMY_CLAIM_INVALID"
  };
  function anatomyError(code, message) {
    const err = new Error(message);
    err.code = code;
    return err;
  }

  // ../packages/modules/anatomy/src/impl.ts
  var CLAIM_BY_PART_VIEW = new WeakMap;
  var ORDER_FOLLOWING = typeof Node !== "undefined" ? Node.DOCUMENT_POSITION_FOLLOWING : 4;
  var ORDER_PRECEDING = typeof Node !== "undefined" ? Node.DOCUMENT_POSITION_PRECEDING : 2;
  var CENTER = (() => {
    const families = new Map;
    const claimsByInstance = new Map;
    const getClaim = (instance, family) => claimsByInstance.get(instance)?.get(family) ?? null;
    return {
      setFamily(family, def2) {
        families.set(family, def2);
      },
      getFamily(family) {
        return families.get(family) ?? null;
      },
      setClaim(record) {
        let byFamily = claimsByInstance.get(record.instance);
        if (!byFamily) {
          byFamily = new Map;
          claimsByInstance.set(record.instance, byFamily);
        }
        byFamily.set(record.family, record);
      },
      getClaim,
      deleteClaim(instance, family) {
        const byFamily = claimsByInstance.get(instance);
        if (!byFamily)
          return;
        byFamily.delete(family);
        if (byFamily.size === 0)
          claimsByInstance.delete(instance);
      },
      listClaims(family) {
        const out = [];
        for (const byFamily of claimsByInstance.values()) {
          const claim = byFamily.get(family);
          if (claim)
            out.push(claim);
        }
        return out;
      }
    };
  })();
  function cloneCardinality(cardinality) {
    return { min: cardinality.min, max: cardinality.max };
  }
  function normalizeCardinality(cardinality) {
    const min = cardinality.min;
    const max = cardinality.max;
    if (typeof min !== "number" || Number.isNaN(min) || min < 0) {
      throw anatomyError(ERR.FAMILY_INVALID, `[Anatomy] invalid cardinality.min`);
    }
    if (max !== "*" && (typeof max !== "number" || Number.isNaN(max) || max < min)) {
      throw anatomyError(ERR.FAMILY_INVALID, `[Anatomy] invalid cardinality.max`);
    }
    return { min, max };
  }
  function compareCardinalityMax(a, b) {
    if (a === b)
      return 0;
    if (a === "*")
      return 1;
    if (b === "*")
      return -1;
    return a - b;
  }
  function exceedsCardinalityMax(count, max) {
    return max !== "*" && count > max;
  }
  function normalizeRequires(requires) {
    if (!requires)
      return [];
    for (const req of requires) {
      if (req?.kind !== "hook" || typeof req.name !== "string" || req.name.length === 0) {
        throw anatomyError(ERR.FAMILY_INVALID, `[Anatomy] invalid requirement`);
      }
    }
    return requires.slice();
  }
  function normalizeFamily(decl) {
    const roles = {};
    for (const [role, roleDecl] of Object.entries(decl.roles ?? {})) {
      roles[role] = {
        cardinality: normalizeCardinality(roleDecl.cardinality),
        requires: normalizeRequires(roleDecl.requires)
      };
    }
    const relations = (decl.relations ?? []).map((it) => ({
      kind: "contains",
      parent: it.parent,
      child: it.child
    }));
    const profiles = {};
    for (const [name, profileDecl] of Object.entries(decl.profiles ?? {})) {
      const nextRoles = {};
      for (const [role, baseRole] of Object.entries(roles)) {
        const patch = profileDecl.roles?.[role];
        const nextCardinality = cloneCardinality(baseRole.cardinality);
        if (patch?.cardinality) {
          if (typeof patch.cardinality.min === "number")
            nextCardinality.min = patch.cardinality.min;
          if (typeof patch.cardinality.max === "number" || patch.cardinality.max === "*") {
            nextCardinality.max = patch.cardinality.max;
          }
        }
        if (nextCardinality.min < baseRole.cardinality.min || compareCardinalityMax(nextCardinality.max, baseRole.cardinality.max) > 0) {
          throw anatomyError(ERR.FAMILY_INVALID, `[Anatomy] profile '${name}' cannot relax family cardinality for role '${role}'`);
        }
        nextRoles[role] = {
          cardinality: normalizeCardinality(nextCardinality),
          requires: [...baseRole.requires, ...normalizeRequires(patch?.requires)]
        };
      }
      profiles[name] = {
        name,
        roles: nextRoles,
        relations: [...relations, ...profileDecl.relations ?? []]
      };
    }
    return { roles, relations, profiles };
  }
  function getHookNames(proto) {
    const trace = proto?.__asHooks;
    const names = new Set;
    if (!Array.isArray(trace))
      return names;
    for (const entry of trace) {
      if (typeof entry?.name === "string" && entry.name)
        names.add(entry.name);
    }
    return names;
  }

  class AnatomyModuleImpl extends ModuleBase {
    static liveInstances = new Set;
    static sharedOrderObservers = new Map;
    prototypeName;
    exposePort;
    disposed = false;
    claimFamilies = new Set;
    orderDispatch = (fn) => fn(undefined);
    orderListeners = new Map;
    targetListeners = new Map;
    observedOrderRoots = new Map;
    orderVersionByFamily = new Map;
    orderSignatureByFamily = new Map;
    objectIds = new WeakMap;
    primitiveIds = new Map;
    nextIdentityId = 1;
    constructor(caps6, prototypeName, exposePort) {
      super(caps6);
      this.prototypeName = prototypeName;
      this.exposePort = exposePort;
      AnatomyModuleImpl.liveInstances.add(this);
    }
    ensureFamilyRegistered(family) {
      const existing = CENTER.getFamily(family);
      if (existing)
        return existing;
      const normalized = normalizeFamily(family.decl);
      CENTER.setFamily(family, normalized);
      return normalized;
    }
    claim(family, decl) {
      this.ensureSetup("def.anatomy.claim");
      const familyDef = this.ensureFamilyRegistered(family);
      if (!familyDef.roles[decl.role]) {
        throw anatomyError(ERR.CLAIM_INVALID, `[Anatomy] unknown role '${decl.role}' in family '${family.debugName}'`);
      }
      if (decl.profile && decl.role !== "root") {
        throw anatomyError(ERR.CLAIM_INVALID, `[Anatomy] only root claim may specify profile`);
      }
      if (decl.profile && !familyDef.profiles[decl.profile]) {
        throw anatomyError(ERR.CLAIM_INVALID, `[Anatomy] unknown profile '${decl.profile}' in family '${family.debugName}'`);
      }
      const instance = this.getSelfToken();
      if (CENTER.getClaim(instance, family)) {
        throw anatomyError(ERR.CLAIM_INVALID, `[Anatomy] duplicate claim for family '${family.debugName}'`);
      }
      CENTER.setClaim({
        instance,
        family,
        role: decl.role,
        profile: decl.profile,
        prototype: this.getPrototypeGetter()(instance),
        exposePort: this.exposePort,
        getRootTarget: this.getRootTargetGetter(),
        invoke: (fn) => this.orderDispatch(fn)
      });
      this.claimFamilies.add(family);
    }
    has(family, role) {
      this.ensureRuntime("run.anatomy.has");
      return (this.partsOf(family, role) ?? []).length > 0;
    }
    subscribeParts(family, role, cb) {
      this.ensureSetup("def.anatomy.subscribeParts");
      let previousSignature = this.computeRoleOrderSignature(family, role);
      return this.subscribeOrder(family, (ctx) => {
        const nextSignature = this.computeRoleOrderSignature(family, role);
        if (nextSignature === previousSignature)
          return;
        previousSignature = nextSignature;
        const parts = this.tryOrderedPartsOf(family, role) ?? [];
        cb(ctx, parts);
      });
    }
    parts(family, options) {
      this.ensureRuntime("run.anatomy.parts");
      if (options?.missing === "null")
        return this.tryParts(family);
      if (options?.missing === "empty")
        return this.tryParts(family) ?? [];
      const domain = this.resolveCurrentDomain(family);
      return domain.claims.map((claim) => this.toPartView(claim));
    }
    tryParts(family) {
      const domain = this.resolveCurrentDomain(family, false);
      if (!domain.rootInstance)
        return null;
      return domain.claims.map((claim) => this.toPartView(claim));
    }
    partsOf(family, role, options) {
      this.ensureRuntime("run.anatomy.partsOf");
      if (options?.missing === "null")
        return this.tryOrderedPartsOf(family, role);
      if (options?.missing === "empty")
        return this.tryOrderedPartsOf(family, role) ?? [];
      const domain = this.resolveCurrentDomain(family);
      return domain.claims.filter((claim) => claim.role === role).map((claim) => this.toPartView(claim));
    }
    orderedParts(family, options) {
      this.ensureRuntime("run.anatomy.order.parts");
      if (options?.missing === "null")
        return this.tryOrderedParts(family);
      if (options?.missing === "empty")
        return this.tryOrderedParts(family) ?? [];
      return this.orderedPartsInternal(family);
    }
    tryOrderedParts(family) {
      const domain = this.resolveCurrentDomain(family, false);
      if (!domain.rootInstance)
        return null;
      return this.sortClaims(domain.claims).map((claim) => this.toPartView(claim));
    }
    orderVersion(family, options) {
      this.ensureRuntime("run.anatomy.order.version");
      if (options?.missing === "null")
        return this.tryOrderVersion(family);
      return this.getOrderVersion(family);
    }
    tryOrderVersion(family) {
      const domain = this.resolveCurrentDomain(family, false);
      if (!domain.rootInstance)
        return null;
      this.primeOrderSignature(family);
      return this.orderVersionByFamily.get(family) ?? 0;
    }
    orderedPartsOf(family, role, options) {
      this.ensureRuntime("run.anatomy.order.partsOf");
      if (options?.missing === "null")
        return this.tryOrderedPartsOf(family, role);
      if (options?.missing === "empty")
        return this.tryOrderedPartsOf(family, role) ?? [];
      return this.orderedPartsOfInternal(family, role);
    }
    tryOrderedPartsOf(family, role) {
      const domain = this.resolveCurrentDomain(family, false);
      if (!domain.rootInstance)
        return null;
      return this.sortClaims(domain.claims.filter((claim) => claim.role === role)).map((claim) => this.toPartView(claim));
    }
    indexOfSelf(family, role, options) {
      this.ensureRuntime("run.anatomy.order.indexOfSelf");
      if (options?.missing === "null")
        return this.tryIndexOfSelf(family, role);
      return this.indexOfSelfInternal(family, role);
    }
    tryIndexOfSelf(family, role) {
      const domain = this.resolveCurrentDomain(family, false);
      if (!domain.rootInstance)
        return null;
      const self = this.getSelfToken();
      const claims = this.sortClaims(domain.claims.filter((claim) => claim.role === role));
      return claims.findIndex((claim) => claim.instance === self);
    }
    prevOfSelf(family, role, options) {
      this.ensureRuntime("run.anatomy.order.prevOfSelf");
      if (options?.missing === "null")
        return this.tryPrevOfSelf(family, role);
      return this.prevOfSelfInternal(family, role);
    }
    tryPrevOfSelf(family, role) {
      const domain = this.resolveCurrentDomain(family, false);
      if (!domain.rootInstance)
        return null;
      const self = this.getSelfToken();
      const claims = this.sortClaims(domain.claims.filter((claim) => claim.role === role));
      const index = claims.findIndex((claim) => claim.instance === self);
      if (index <= 0)
        return null;
      return this.toPartView(claims[index - 1]);
    }
    nextOfSelf(family, role, options) {
      this.ensureRuntime("run.anatomy.order.nextOfSelf");
      if (options?.missing === "null")
        return this.tryNextOfSelf(family, role);
      return this.nextOfSelfInternal(family, role);
    }
    tryNextOfSelf(family, role) {
      const domain = this.resolveCurrentDomain(family, false);
      if (!domain.rootInstance)
        return null;
      const self = this.getSelfToken();
      const claims = this.sortClaims(domain.claims.filter((claim) => claim.role === role));
      const index = claims.findIndex((claim) => claim.instance === self);
      if (index < 0 || index >= claims.length - 1)
        return null;
      return this.toPartView(claims[index + 1]);
    }
    port = {
      getDiagnostics: () => {
        const out = [];
        for (const family of this.claimFamilies) {
          out.push(...this.computeDiagnostics(family));
        }
        return out;
      },
      resolvePartTarget: (part) => {
        const claim = CLAIM_BY_PART_VIEW.get(part);
        return claim?.getRootTarget(claim.instance) ?? null;
      },
      resolveDomainScope: (family) => this.resolveCurrentDomain(family, false).rootInstance,
      descendantsOf: (family, ancestor, role) => {
        const ancestorClaim = CLAIM_BY_PART_VIEW.get(ancestor);
        if (!ancestorClaim || ancestorClaim.family !== family)
          return [];
        const domain = this.resolveCurrentDomain(family, false);
        if (!domain.rootInstance || !domain.claims.includes(ancestorClaim))
          return [];
        const getParent = this.getParentGetter();
        return this.sortClaims(domain.claims.filter((claim) => {
          if (claim.role !== role)
            return false;
          let current = getParent(claim.instance);
          while (current) {
            if (current === ancestorClaim.instance)
              return true;
            if (current === domain.rootInstance)
              return false;
            current = getParent(current);
          }
          return false;
        })).map((claim) => this.toPartView(claim));
      },
      parts: (family, options) => {
        if (options?.missing === "null")
          return this.tryParts(family);
        if (options?.missing === "empty")
          return this.tryParts(family) ?? [];
        const domain = this.resolveCurrentDomain(family);
        return domain.claims.map((claim) => this.toPartView(claim));
      },
      order: {
        version: (family, options) => {
          if (options?.missing === "null")
            return this.tryOrderVersion(family);
          return this.getOrderVersion(family);
        },
        parts: (family, options) => {
          if (options?.missing === "null")
            return this.tryOrderedParts(family);
          if (options?.missing === "empty")
            return this.tryOrderedParts(family) ?? [];
          return this.orderedPartsInternal(family);
        },
        partsOf: (family, role, options) => {
          if (options?.missing === "null")
            return this.tryOrderedPartsOf(family, role);
          if (options?.missing === "empty")
            return this.tryOrderedPartsOf(family, role) ?? [];
          return this.orderedPartsOfInternal(family, role);
        },
        indexOfSelf: (family, role, options) => {
          if (options?.missing === "null")
            return this.tryIndexOfSelf(family, role);
          return this.indexOfSelfInternal(family, role);
        },
        prevOfSelf: (family, role, options) => {
          if (options?.missing === "null")
            return this.tryPrevOfSelf(family, role);
          return this.prevOfSelfInternal(family, role);
        },
        nextOfSelf: (family, role, options) => {
          if (options?.missing === "null")
            return this.tryNextOfSelf(family, role);
          return this.nextOfSelfInternal(family, role);
        },
        tryVersion: (family) => {
          const domain = this.resolveCurrentDomain(family, false);
          if (!domain.rootInstance)
            return null;
          this.primeOrderSignature(family);
          return this.orderVersionByFamily.get(family) ?? 0;
        },
        tryParts: (family) => {
          const domain = this.resolveCurrentDomain(family, false);
          if (!domain.rootInstance)
            return null;
          return this.sortClaims(domain.claims).map((claim) => this.toPartView(claim));
        },
        tryPartsOf: (family, role) => {
          const domain = this.resolveCurrentDomain(family, false);
          if (!domain.rootInstance)
            return null;
          return this.sortClaims(domain.claims.filter((claim) => claim.role === role)).map((claim) => this.toPartView(claim));
        },
        tryIndexOfSelf: (family, role) => {
          const domain = this.resolveCurrentDomain(family, false);
          if (!domain.rootInstance)
            return null;
          const self = this.getSelfToken();
          const claims = this.sortClaims(domain.claims.filter((claim) => claim.role === role));
          return claims.findIndex((claim) => claim.instance === self);
        },
        tryPrevOfSelf: (family, role) => {
          const domain = this.resolveCurrentDomain(family, false);
          if (!domain.rootInstance)
            return null;
          const self = this.getSelfToken();
          const claims = this.sortClaims(domain.claims.filter((claim) => claim.role === role));
          const index = claims.findIndex((claim) => claim.instance === self);
          if (index <= 0)
            return null;
          return this.toPartView(claims[index - 1]);
        },
        tryNextOfSelf: (family, role) => {
          const domain = this.resolveCurrentDomain(family, false);
          if (!domain.rootInstance)
            return null;
          const self = this.getSelfToken();
          const claims = this.sortClaims(domain.claims.filter((claim) => claim.role === role));
          const index = claims.findIndex((claim) => claim.instance === self);
          if (index < 0 || index >= claims.length - 1)
            return null;
          return this.toPartView(claims[index + 1]);
        }
      },
      setOrderCallbackDispatcher: (dispatch) => {
        this.orderDispatch = dispatch;
      },
      subscribeOrder: (family, cb) => this.subscribeOrder(family, cb),
      subscribeTargets: (family, cb) => this.subscribeTargets(family, cb)
    };
    onProtoPhase(phase) {
      super.onProtoPhase(phase);
      if (phase === "mounted") {
        for (const family of this.claimFamilies) {
          AnatomyModuleImpl.notifyStructuralChange(family);
          AnatomyModuleImpl.notifyTargetChange(family);
        }
      }
      if (phase === "unmounted") {
        for (const family of this.claimFamilies)
          AnatomyModuleImpl.notifyTargetChange(family);
        this.dispose();
      }
    }
    onMountPhase(phase, epoch) {
      super.onMountPhase(phase, epoch);
      if (phase === "unmounting" || phase === "detached") {
        for (const family of this.claimFamilies)
          AnatomyModuleImpl.notifyTargetChange(family);
        for (const family of Array.from(this.observedOrderRoots.keys())) {
          this.teardownOrderObserver(family);
        }
        return;
      }
      if (phase === "mounted") {
        for (const family of this.orderListeners.keys())
          this.ensureOrderObserver(family);
        for (const family of this.claimFamilies)
          AnatomyModuleImpl.notifyTargetChange(family);
      }
    }
    onCapsEpoch(_epoch) {
      for (const family of this.claimFamilies)
        AnatomyModuleImpl.notifyTargetChange(family);
      if (this.mountPhase !== "mounted")
        return;
      for (const family of Array.from(this.observedOrderRoots.keys())) {
        this.teardownOrderObserver(family);
      }
      for (const family of this.orderListeners.keys())
        this.ensureOrderObserver(family);
    }
    dispose() {
      if (this.disposed)
        return;
      this.disposed = true;
      AnatomyModuleImpl.liveInstances.delete(this);
      for (const family of this.observedOrderRoots.keys()) {
        this.teardownOrderObserver(family);
      }
      this.observedOrderRoots.clear();
      this.orderListeners.clear();
      this.targetListeners.clear();
      if (this.claimFamilies.size === 0)
        return;
      if (!this.caps.has(ANATOMY_INSTANCE_TOKEN_CAP))
        return;
      const instance = this.caps.get(ANATOMY_INSTANCE_TOKEN_CAP);
      for (const family of this.claimFamilies) {
        CENTER.deleteClaim(instance, family);
        AnatomyModuleImpl.notifyStructuralChange(family);
      }
      this.claimFamilies.clear();
    }
    getOrderedClaimsOf(family, role) {
      const domain = this.resolveCurrentDomain(family);
      return this.sortClaims(domain.claims.filter((claim) => claim.role === role));
    }
    orderedPartsInternal(family) {
      const domain = this.resolveCurrentDomain(family);
      return this.sortClaims(domain.claims).map((claim) => this.toPartView(claim));
    }
    orderedPartsOfInternal(family, role) {
      return this.getOrderedClaimsOf(family, role).map((claim) => this.toPartView(claim));
    }
    indexOfSelfInternal(family, role) {
      const self = this.getSelfToken();
      const claims = this.getOrderedClaimsOf(family, role);
      return claims.findIndex((claim) => claim.instance === self);
    }
    prevOfSelfInternal(family, role) {
      const self = this.getSelfToken();
      const claims = this.getOrderedClaimsOf(family, role);
      const index = claims.findIndex((claim) => claim.instance === self);
      if (index <= 0)
        return null;
      return this.toPartView(claims[index - 1]);
    }
    nextOfSelfInternal(family, role) {
      const self = this.getSelfToken();
      const claims = this.getOrderedClaimsOf(family, role);
      const index = claims.findIndex((claim) => claim.instance === self);
      if (index < 0 || index >= claims.length - 1)
        return null;
      return this.toPartView(claims[index + 1]);
    }
    subscribeOrder(family, cb) {
      let set = this.orderListeners.get(family);
      if (!set) {
        set = new Set;
        this.orderListeners.set(family, set);
      }
      set.add(cb);
      this.primeOrderSignature(family);
      this.ensureOrderObserver(family);
      return () => {
        const current = this.orderListeners.get(family);
        current?.delete(cb);
        if (!current || current.size > 0)
          return;
        this.orderListeners.delete(family);
        this.teardownOrderObserver(family);
      };
    }
    subscribeTargets(family, cb) {
      let set = this.targetListeners.get(family);
      if (!set) {
        set = new Set;
        this.targetListeners.set(family, set);
      }
      set.add(cb);
      return () => {
        const current = this.targetListeners.get(family);
        current?.delete(cb);
        if (current?.size === 0)
          this.targetListeners.delete(family);
      };
    }
    static notifyStructuralChange(family) {
      for (const impl of AnatomyModuleImpl.liveInstances) {
        if (!impl.orderListeners.has(family))
          continue;
        impl.emitOrderChangeIfNeeded(family);
      }
    }
    static notifyTargetChange(family) {
      for (const impl of AnatomyModuleImpl.liveInstances) {
        const listeners = impl.targetListeners.get(family);
        if (!listeners?.size)
          continue;
        impl.orderDispatch((ctx) => {
          for (const listener of listeners)
            listener(ctx);
        });
      }
    }
    ensureOrderObserver(family) {
      if (this.mountPhase !== "mounted")
        return;
      if (this.observedOrderRoots.has(family))
        return;
      if (!this.caps.has(ANATOMY_ORDER_OBSERVER_CAP))
        return;
      const domain = this.resolveCurrentDomain(family, false);
      if (!domain.rootInstance)
        return;
      const target = this.getObservedRootTarget(family);
      if (!target)
        return;
      let byRoot = AnatomyModuleImpl.sharedOrderObservers.get(family);
      if (!byRoot) {
        byRoot = new Map;
        AnatomyModuleImpl.sharedOrderObservers.set(family, byRoot);
      }
      let entry = byRoot.get(domain.rootInstance);
      if (!entry) {
        const observer = this.caps.get(ANATOMY_ORDER_OBSERVER_CAP);
        const listeners = new Set;
        const off = observer(target, () => {
          for (const impl of listeners) {
            impl.emitOrderChangeIfNeeded(family);
          }
        });
        entry = { off, listeners };
        byRoot.set(domain.rootInstance, entry);
      }
      entry.listeners.add(this);
      this.observedOrderRoots.set(family, domain.rootInstance);
    }
    teardownOrderObserver(family) {
      const rootInstance = this.observedOrderRoots.get(family);
      if (!rootInstance)
        return;
      this.observedOrderRoots.delete(family);
      const byRoot = AnatomyModuleImpl.sharedOrderObservers.get(family);
      const entry = byRoot?.get(rootInstance);
      if (!entry)
        return;
      entry.listeners.delete(this);
      if (entry.listeners.size > 0)
        return;
      entry.off();
      byRoot?.delete(rootInstance);
      if (byRoot && byRoot.size === 0) {
        AnatomyModuleImpl.sharedOrderObservers.delete(family);
      }
    }
    primeOrderSignature(family) {
      if (this.orderSignatureByFamily.has(family))
        return;
      const signature = this.computeOrderSignature(family);
      this.orderSignatureByFamily.set(family, signature);
      this.orderVersionByFamily.set(family, 0);
    }
    getOrderVersion(family) {
      this.primeOrderSignature(family);
      return this.orderVersionByFamily.get(family) ?? 0;
    }
    emitOrderChangeIfNeeded(family) {
      const prev = this.orderSignatureByFamily.get(family);
      const next = this.computeOrderSignature(family);
      if (typeof prev !== "undefined" && prev === next)
        return;
      this.orderSignatureByFamily.set(family, next);
      const nextVersion = (this.orderVersionByFamily.get(family) ?? 0) + 1;
      this.orderVersionByFamily.set(family, nextVersion);
      const listeners = this.orderListeners.get(family);
      if (!listeners || listeners.size === 0)
        return;
      this.orderDispatch((ctx) => {
        for (const listener of listeners)
          listener(ctx);
      });
    }
    computeOrderSignature(family) {
      const domain = this.resolveCurrentDomain(family, false);
      if (!domain.rootInstance)
        return "missing-domain";
      const ordered = this.sortClaims(domain.claims);
      return ordered.map((claim) => `${claim.role}:${this.getIdentityId(claim.instance)}`).join("|");
    }
    computeRoleOrderSignature(family, role) {
      const domain = this.resolveCurrentDomain(family, false);
      if (!domain.rootInstance)
        return "missing-domain";
      const ordered = this.sortClaims(domain.claims.filter((claim) => claim.role === role));
      return ordered.map((claim) => this.getIdentityId(claim.instance)).join("|");
    }
    getIdentityId(value) {
      if (value && typeof value === "object") {
        const obj = value;
        const existing = this.objectIds.get(obj);
        if (existing)
          return existing;
        const next2 = this.nextIdentityId++;
        this.objectIds.set(obj, next2);
        return next2;
      }
      if (this.primitiveIds.has(value)) {
        return this.primitiveIds.get(value);
      }
      const next = this.nextIdentityId++;
      this.primitiveIds.set(value, next);
      return next;
    }
    getObservedRootTarget(family) {
      const domain = this.resolveCurrentDomain(family, false);
      if (!domain.rootInstance)
        return null;
      const rootClaim = CENTER.getClaim(domain.rootInstance, family);
      if (!rootClaim)
        return null;
      return rootClaim.getRootTarget(rootClaim.instance);
    }
    sortClaims(claims) {
      return claims.map((claim, index) => ({ claim, index })).sort((left, right) => {
        const cmp = this.compareClaims(left.claim, right.claim);
        if (cmp !== 0)
          return cmp;
        return left.index - right.index;
      }).map((entry) => entry.claim);
    }
    compareClaims(a, b) {
      const aTarget = a.getRootTarget(a.instance);
      const bTarget = b.getRootTarget(b.instance);
      if (!aTarget || !bTarget || aTarget === bTarget)
        return 0;
      if (typeof aTarget.compareDocumentPosition !== "function")
        return 0;
      const pos = aTarget.compareDocumentPosition(bTarget);
      if (pos & ORDER_FOLLOWING)
        return -1;
      if (pos & ORDER_PRECEDING)
        return 1;
      return 0;
    }
    computeDiagnostics(family) {
      const familyDef = CENTER.getFamily(family);
      if (!familyDef)
        return [];
      const domain = this.resolveCurrentDomain(family, false);
      const claims = domain.claims;
      const diagnostics = [];
      const counts = new Map;
      for (const claim of claims)
        counts.set(claim.role, (counts.get(claim.role) ?? 0) + 1);
      for (const [role, roleState] of Object.entries(familyDef.roles)) {
        const count = counts.get(role) ?? 0;
        if (count < roleState.cardinality.min) {
          diagnostics.push({
            level: "error",
            scope: "family",
            code: "ANATOMY_FAMILY_MIN",
            message: `[Anatomy] role '${role}' is below family min in '${family.debugName}'`,
            family,
            role
          });
        }
        if (exceedsCardinalityMax(count, roleState.cardinality.max)) {
          diagnostics.push({
            level: "error",
            scope: "family",
            code: "ANATOMY_FAMILY_MAX",
            message: `[Anatomy] role '${role}' exceeds family max in '${family.debugName}'`,
            family,
            role
          });
        }
      }
      for (const claim of claims) {
        const familyRole = familyDef.roles[claim.role];
        for (const req of familyRole?.requires ?? []) {
          if (!getHookNames(claim.prototype).has(req.name)) {
            diagnostics.push({
              level: "error",
              scope: "family",
              code: "ANATOMY_FAMILY_HOOK_REQUIRED",
              message: `[Anatomy] role '${claim.role}' requires hook '${req.name}'`,
              family,
              role: claim.role
            });
          }
        }
      }
      for (const relation of familyDef.relations) {
        for (const claim of claims.filter((it) => it.role === relation.child)) {
          if (!this.hasAncestorRole(claim.instance, family, relation.parent, domain.rootInstance)) {
            diagnostics.push({
              level: "error",
              scope: "family",
              code: "ANATOMY_FAMILY_RELATION",
              message: `[Anatomy] role '${relation.child}' must be contained by '${relation.parent}'`,
              family,
              role: relation.child
            });
          }
        }
      }
      const profileName = domain.profile;
      if (!profileName)
        return diagnostics;
      const profile = familyDef.profiles[profileName];
      if (!profile)
        return diagnostics;
      for (const [role, roleState] of Object.entries(profile.roles)) {
        const count = counts.get(role) ?? 0;
        if (count < roleState.cardinality.min) {
          diagnostics.push({
            level: "warning",
            scope: "profile",
            code: "ANATOMY_PROFILE_MIN",
            message: `[Anatomy] role '${role}' is below profile min in '${family.debugName}/${profileName}'`,
            family,
            role,
            profile: profileName
          });
        }
        if (exceedsCardinalityMax(count, roleState.cardinality.max)) {
          diagnostics.push({
            level: "warning",
            scope: "profile",
            code: "ANATOMY_PROFILE_MAX",
            message: `[Anatomy] role '${role}' exceeds profile max in '${family.debugName}/${profileName}'`,
            family,
            role,
            profile: profileName
          });
        }
      }
      for (const claim of claims) {
        const profileRole = profile.roles[claim.role];
        const familyReqNames = new Set((familyDef.roles[claim.role]?.requires ?? []).map((it) => it.name));
        for (const req of profileRole?.requires ?? []) {
          if (familyReqNames.has(req.name))
            continue;
          if (!getHookNames(claim.prototype).has(req.name)) {
            diagnostics.push({
              level: "warning",
              scope: "profile",
              code: "ANATOMY_PROFILE_HOOK_REQUIRED",
              message: `[Anatomy] role '${claim.role}' is missing profile hook '${req.name}'`,
              family,
              role: claim.role,
              profile: profileName
            });
          }
        }
      }
      for (const relation of profile.relations.slice(familyDef.relations.length)) {
        for (const claim of claims.filter((it) => it.role === relation.child)) {
          if (!this.hasAncestorRole(claim.instance, family, relation.parent, domain.rootInstance)) {
            diagnostics.push({
              level: "warning",
              scope: "profile",
              code: "ANATOMY_PROFILE_RELATION",
              message: `[Anatomy] role '${relation.child}' should be contained by '${relation.parent}'`,
              family,
              role: relation.child,
              profile: profileName
            });
          }
        }
      }
      return diagnostics;
    }
    resolveCurrentDomain(family, strict = true) {
      const instance = this.getSelfToken();
      const rootInstance = this.findDomainRoot(instance, family);
      if (!rootInstance) {
        if (!strict)
          return { rootInstance: null, claims: [], profile: null };
        throw anatomyError(ERR.CLAIM_INVALID, `[Anatomy] current instance is not part of a valid domain for '${family.debugName}'`);
      }
      const claims = CENTER.listClaims(family).filter((claim) => this.findDomainRoot(claim.instance, family) === rootInstance);
      const rootClaim = claims.find((claim) => claim.role === "root" && claim.instance === rootInstance) ?? null;
      return {
        rootInstance,
        claims,
        profile: rootClaim?.profile ?? null
      };
    }
    findDomainRoot(instance, family) {
      const getParent = this.getParentGetter();
      let cur = instance;
      while (cur) {
        const claim = CENTER.getClaim(cur, family);
        if (claim?.role === "root")
          return cur;
        cur = getParent(cur);
      }
      return null;
    }
    hasAncestorRole(instance, family, role, domainRoot) {
      const getParent = this.getParentGetter();
      let cur = getParent(instance);
      while (cur) {
        const claim = CENTER.getClaim(cur, family);
        if (claim?.role === role)
          return true;
        if (domainRoot && cur === domainRoot)
          break;
        cur = getParent(cur);
      }
      return false;
    }
    toPartView(claim) {
      const part = {
        role: claim.role,
        hasExpose: (key) => claim.exposePort.has(key),
        getExpose: (key) => {
          if (!claim.exposePort.has(key))
            return null;
          const exposed = claim.exposePort.get(key) ?? null;
          if (typeof exposed !== "function")
            return exposed;
          return (...args) => {
            let result;
            let thrown;
            claim.invoke(() => {
              try {
                result = exposed(...args);
              } catch (error4) {
                thrown = error4;
              }
            });
            if (thrown)
              throw thrown;
            return result;
          };
        },
        hasHook: (name) => getHookNames(claim.prototype).has(name)
      };
      CLAIM_BY_PART_VIEW.set(part, claim);
      return part;
    }
    ensureSetup(op) {
      if (this.sys) {
        try {
          this.sys.ensureSetup(op);
          return;
        } catch (error4) {
          throw anatomyError(ERR.PHASE, `[Anatomy] setup-only: ${op}`);
        }
      }
      if (this.protoPhase !== "setup") {
        throw illegalPhase(op, this.protoPhase, { prototypeName: this.prototypeName });
      }
    }
    ensureRuntime(op) {
      if (this.sys) {
        try {
          this.sys.ensureRuntime(op);
          return;
        } catch {
          throw anatomyError(ERR.PHASE, `[Anatomy] runtime-only: ${op}`);
        }
      }
      if (this.protoPhase === "setup") {
        throw illegalPhase(op, this.protoPhase, { prototypeName: this.prototypeName });
      }
    }
    ensureCallback(op) {
      if (this.sys) {
        try {
          this.sys.ensureCallback(op);
          return;
        } catch {
          throw anatomyError(ERR.PHASE, `[Anatomy] callback-only: ${op}`);
        }
      }
      if (this.protoPhase === "setup") {
        throw illegalPhase(op, this.protoPhase, { prototypeName: this.prototypeName });
      }
    }
    getSelfToken() {
      if (!this.caps.has(ANATOMY_INSTANCE_TOKEN_CAP)) {
        throw anatomyError(ERR.CAP, `[Anatomy] host caps missing: instance token`);
      }
      return this.caps.get(ANATOMY_INSTANCE_TOKEN_CAP);
    }
    getParentGetter() {
      if (!this.caps.has(ANATOMY_PARENT_CAP)) {
        throw anatomyError(ERR.CAP, `[Anatomy] host caps missing: parent getter`);
      }
      return this.caps.get(ANATOMY_PARENT_CAP);
    }
    getPrototypeGetter() {
      if (!this.caps.has(ANATOMY_GET_PROTO_CAP)) {
        throw anatomyError(ERR.CAP, `[Anatomy] host caps missing: prototype getter`);
      }
      return this.caps.get(ANATOMY_GET_PROTO_CAP);
    }
    getRootTargetGetter() {
      if (!this.caps.has(ANATOMY_ROOT_TARGET_CAP)) {
        return () => null;
      }
      return this.caps.get(ANATOMY_ROOT_TARGET_CAP);
    }
  }

  // ../packages/modules/anatomy/src/create.ts
  function createAnatomyModule(ctx) {
    const { init, caps: caps6, deps } = ctx;
    const exposePort = deps.requirePort("expose");
    return createModule({
      name: "anatomy",
      scope: "instance",
      init,
      caps: caps6,
      deps,
      build: ({ init: init2, caps: caps7 }) => {
        const impl = new AnatomyModuleImpl(caps7, init2.prototypeName, exposePort);
        return {
          facade: {
            claim: (family, decl) => impl.claim(family, decl),
            subscribeParts: (family, role, cb) => impl.subscribeParts(family, role, cb),
            has: (family, role) => impl.has(family, role),
            parts: (family, options) => impl.parts(family, options),
            partsOf: (family, role, options) => impl.partsOf(family, role, options),
            order: {
              version: (family, options) => impl.orderVersion(family, options),
              parts: (family, options) => impl.orderedParts(family, options),
              partsOf: (family, role, options) => impl.orderedPartsOf(family, role, options),
              indexOfSelf: (family, role, options) => impl.indexOfSelf(family, role, options),
              prevOfSelf: (family, role, options) => impl.prevOfSelf(family, role, options),
              nextOfSelf: (family, role, options) => impl.nextOfSelf(family, role, options)
            }
          },
          port: impl.port,
          hooks: {
            onMountPhase: (p, epoch) => impl.onMountPhase(p, epoch),
            onProtoPhase: (p) => impl.onProtoPhase(p),
            dispose: () => impl.dispose()
          }
        };
      }
    });
  }
  var AnatomyModuleDef = defineModule({
    name: "anatomy",
    resourceOwnership: "mixed",
    deps: ["expose"],
    create: createAnatomyModule
  });
  // ../packages/modules/expose-state/src/types.ts
  var EXPOSE_STATE_EXTERNAL_HANDLE = Symbol.for("@proto.ui/expose-state/external-handle");
  function isExposeStateExternalHandle(value) {
    return !!value && typeof value === "object" && value[EXPOSE_STATE_EXTERNAL_HANDLE] === true;
  }

  // ../packages/modules/expose-state/src/caps.ts
  var EXPOSES_RECORD_SINK_CAP = cap("@proto.ui/expose-state/setExposes");
  var EXPOSE_STATE_SET_EXPOSES_CAP = EXPOSES_RECORD_SINK_CAP;

  // ../packages/modules/expose-state/src/impl.ts
  var STATE_ID = "__stateId";
  var STATE_SPEC = "__stateSpec";
  function isStateHandleLike2(x) {
    return !!x && typeof x === "object" && typeof x.get === "function" && !!x[STATE_ID];
  }
  function getSpecFromHandle(handle) {
    const spec2 = handle?.[STATE_SPEC];
    return spec2 ?? null;
  }
  function toDiag2(key, value, isState) {
    return {
      key,
      kind: isState ? "state" : "value",
      valueType: typeof value
    };
  }

  class ExposeStateModuleImpl extends ModuleBase {
    exposePort;
    statePort;
    disposed = false;
    cache = new Map;
    externalHandleCache = new WeakMap;
    externalSubscriptions = new Set;
    publishedSink = null;
    constructor(caps7, deps) {
      super(caps7);
      this.exposePort = deps.requirePort("expose");
      this.statePort = deps.requirePort("state");
    }
    port = {
      get: (key) => {
        this.ensureAlive("rt.exposeState.get");
        this.sync();
        return this.cache.get(key);
      },
      getAll: () => {
        this.ensureAlive("rt.exposeState.getAll");
        this.sync();
        const out = {};
        for (const [k, v] of this.cache) {
          Object.defineProperty(out, k, {
            value: v,
            enumerable: true,
            configurable: true,
            writable: true
          });
        }
        return out;
      },
      getDiagnostics: () => {
        this.ensureAlive("rt.exposeState.getDiagnostics");
        this.sync();
        const diags = [];
        for (const [k, v] of this.cache) {
          const isState = isStateHandleLike2(v) || v?.spec !== undefined;
          diags.push(toDiag2(k, v, isState));
        }
        return diags;
      }
    };
    onInstancePhase(phase) {
      super.onInstancePhase(phase);
      if (phase === "alive")
        this.publishToHost();
    }
    afterRenderCommit() {
      this.publishToHost();
    }
    onCapsEpoch(_epoch) {
      this.publishToHost();
    }
    dispose() {
      if (this.disposed)
        return;
      this.disposed = true;
      for (const off of this.externalSubscriptions) {
        try {
          off();
        } catch {}
      }
      this.externalSubscriptions.clear();
      this.cache.clear();
      this.publishToHost(true);
    }
    ensureAlive(op) {
      this.sys?.ensureNotDisposed(op);
      if (this.disposed)
        throw new Error(`[ExposeState] disposed. op=${op}`);
    }
    sync() {
      const raw = this.exposePort.getAll();
      this.cache.clear();
      for (const [key, value] of Object.entries(raw)) {
        if (!isStateHandleLike2(value)) {
          this.cache.set(key, value);
          continue;
        }
        const spec2 = getSpecFromHandle(value);
        if (!spec2) {
          throw new Error(`[ExposeState] missing StateSpec on exposed handle: ${key}`);
        }
        let external = this.externalHandleCache.get(value);
        if (!external) {
          external = this.wrapExternalHandle(value, spec2);
          this.externalHandleCache.set(value, external);
        }
        this.cache.set(key, external);
      }
    }
    wrapExternalHandle(handle, spec2) {
      const external = {
        [EXPOSE_STATE_EXTERNAL_HANDLE]: true,
        get: () => {
          this.ensureAlive("rt.exposeState.external.get");
          return handle.get();
        },
        subscribe: (cb) => {
          this.ensureAlive("rt.exposeState.external.subscribe");
          const off = this.statePort.watch(handle, (_ctx, e) => {
            if (this.disposed)
              return;
            cb(e);
          });
          const trackedOff = () => {
            this.externalSubscriptions.delete(trackedOff);
            off();
          };
          this.externalSubscriptions.add(trackedOff);
          return trackedOff;
        },
        unsubscribe: (off) => {
          if (typeof off === "function")
            off();
        },
        spec: spec2
      };
      external.__stateSemantic = handle.__stateSemantic;
      external.__stateId = handle.__stateId;
      return external;
    }
    publishToHost(clear = false) {
      const nextSink = !clear && this.caps.has(EXPOSES_RECORD_SINK_CAP) ? this.caps.get(EXPOSES_RECORD_SINK_CAP) : null;
      if (this.publishedSink && this.publishedSink !== nextSink) {
        try {
          this.publishedSink({});
        } catch {}
      }
      this.publishedSink = nextSink;
      if (clear || !nextSink || this.instancePhase === "setup")
        return;
      const record = this.port.getAll();
      try {
        nextSink(record);
      } catch {}
    }
  }

  // ../packages/modules/expose-state/src/create.ts
  function createExposeStateModule(ctx) {
    const { init, caps: caps7, deps } = ctx;
    return createModule({
      name: "expose-state",
      scope: "instance",
      init,
      caps: caps7,
      deps,
      build: ({ caps: caps8, deps: deps2 }) => {
        const impl = new ExposeStateModuleImpl(caps8, deps2);
        return {
          facade: {},
          port: impl.port,
          hooks: {
            onInstancePhase: (p) => impl.onInstancePhase(p),
            afterRenderCommit: () => impl.afterRenderCommit(),
            dispose: () => impl.dispose()
          }
        };
      }
    });
  }
  var ExposeStateModuleDef = defineModule({
    name: "expose-state",
    resourceOwnership: "instance",
    deps: ["expose", "state"],
    create: createExposeStateModule
  });
  // ../packages/modules/expose-state-web/src/caps.ts
  var EXPOSE_STATE_WEB_MAP_CAP = cap("@proto.ui/expose-state-web/nameMap");
  var EXPOSE_STATE_WEB_MODE_CAP = cap("@proto.ui/expose-state-web/mode");
  var EXPOSE_STATE_WEB_MIRROR_TARGETS_CAP = cap("@proto.ui/expose-state-web/mirrorTargets");

  // ../packages/modules/expose-state-web/src/utils.ts
  function mapOfficialSemanticName(semantic) {
    switch (semantic) {
      case "@interaction/disabled":
        return "disabled";
      case "@interaction/hovered":
        return "hovered";
      case "@interaction/pressed":
        return "pressed";
      case "@interaction/focused":
      case "@focus/focused":
        return "focused";
      case "@interaction/focusVisible":
      case "@focus/focusVisible":
        return "focus-visible";
      case "@accessibility/expanded":
        return "expanded";
      case "@accessibility/invalid":
        return "invalid";
      case "@accessibility/selected":
        return "selected";
      case "@accessibility/checked":
        return "checked";
      case "@accessibility/current":
        return "current";
      default:
        return null;
    }
  }
  function createExposeStateWebNameMap(semantic) {
    const official = mapOfficialSemanticName(semantic);
    if (official) {
      return {
        dataAttr: `data-${official}`,
        cssVar: `--pui-${official}`
      };
    }
    const base = semantic.trim().replace(/\s+/g, "-").replace(/\./g, "-").replace(/([a-z0-9])([A-Z])/g, "$1-$2").replace(/[^a-zA-Z0-9\-]/g, "-").replace(/-+/g, "-").replace(/^-+|-+$/g, "").toLowerCase();
    return {
      dataAttr: `data-${base}`,
      cssVar: `--pui-${base}`
    };
  }

  // ../packages/modules/expose-state-web/src/impl.ts
  class ExposeStateWebModuleImpl extends ModuleBase {
    exposeState;
    disposed = false;
    bindings = [];
    active = false;
    exposedByStateId = new Map;
    constructor(caps8, deps) {
      super(caps8);
      this.exposeState = deps.requirePort("expose-state");
    }
    onMountPhase(phase, epoch) {
      super.onMountPhase(phase, epoch);
      if (phase !== "detached")
        return;
      this.active = false;
      this.clearBindings();
      this.exposedByStateId.clear();
    }
    afterRenderCommit() {
      this.refresh();
    }
    onCapsEpoch(_epoch) {
      this.refresh();
    }
    dispose() {
      if (this.disposed)
        return;
      this.disposed = true;
      this.clearBindings();
    }
    refresh() {
      if (this.disposed)
        return;
      if (this.mountPhase === "detached" || this.mountPhase === "unmounting")
        return;
      if (!this.caps.has(HOST_ELEMENT_CAP)) {
        this.active = false;
        this.exposedByStateId.clear();
        return;
      }
      const host = this.caps.get(HOST_ELEMENT_CAP);
      if (!host) {
        this.active = false;
        this.exposedByStateId.clear();
        return;
      }
      this.active = true;
      const nameMap = this.caps.has(EXPOSE_STATE_WEB_MAP_CAP) ? this.caps.get(EXPOSE_STATE_WEB_MAP_CAP) : createExposeStateWebNameMap;
      const mode = this.caps.has(EXPOSE_STATE_WEB_MODE_CAP) ? this.caps.get(EXPOSE_STATE_WEB_MODE_CAP) : {};
      const all = this.exposeState.getAll();
      this.clearBindings();
      this.exposedByStateId.clear();
      for (const [key, value] of Object.entries(all)) {
        if (!isExposeStateExternalHandle(value))
          continue;
        const spec2 = value.spec;
        const semantic = value.__stateSemantic || key;
        const stateId = String(value.__stateId ?? "");
        const mapping = nameMap(semantic);
        const binding = {
          key,
          stateId,
          kind: spec2.kind,
          attr: this.allowAttrForKind(spec2.kind, mode) ? mapping.dataAttr : undefined,
          cssVar: mapping.cssVar
        };
        if (stateId) {
          this.exposedByStateId.set(stateId, {
            stateId,
            key,
            semantic,
            kind: spec2.kind,
            attr: binding.attr,
            cssVar: binding.cssVar
          });
        }
        this.applySnapshot(host, value, binding, mode);
        const off = value.subscribe((e) => {
          if (e.type === "disconnect")
            return;
          this.applyValue(host, e.next, binding, mode);
        });
        binding.off = () => value.unsubscribe(off);
        this.bindings.push(binding);
      }
    }
    applySnapshot(host, h, binding, mode) {
      const v = h.get();
      this.applyValue(host, v, binding, mode);
    }
    applyValue(host, v, binding, mode) {
      const kind = binding.kind;
      if (!kind)
        return;
      const attr = binding.attr;
      const cssVar = binding.cssVar;
      const setAttr = (val) => {
        if (!attr)
          return;
        for (const target of this.resolveProjectionTargets(host)) {
          if (val === null)
            target.removeAttribute(attr);
          else
            target.setAttribute(attr, val);
        }
      };
      const setVar = (val) => {
        if (!cssVar)
          return;
        for (const target of this.resolveProjectionTargets(host)) {
          if (val === null)
            target.style.removeProperty(cssVar);
          else
            target.style.setProperty(cssVar, val);
        }
      };
      switch (kind) {
        case "bool": {
          if (v)
            setAttr("");
          else
            setAttr(null);
          break;
        }
        case "enum":
        case "string": {
          const value = v == null ? "" : String(v);
          setAttr(value);
          if (mode.allowStringVar)
            setVar(value);
          break;
        }
        case "number.discrete": {
          const value = v == null ? "" : String(v);
          setAttr(value);
          setVar(value);
          break;
        }
        case "number.range": {
          const value = v == null ? "" : String(v);
          if (mode.allowContinuousAttr)
            setAttr(value);
          setVar(value);
          break;
        }
        default: {
          break;
        }
      }
    }
    resolveProjectionTargets(host) {
      const targets = [host];
      const seen = new Set(targets);
      if (!this.caps.has(EXPOSE_STATE_WEB_MIRROR_TARGETS_CAP))
        return targets;
      for (const target of this.caps.get(EXPOSE_STATE_WEB_MIRROR_TARGETS_CAP)()) {
        if (!target || seen.has(target))
          continue;
        seen.add(target);
        targets.push(target);
      }
      return targets;
    }
    clearBindings() {
      for (const b of this.bindings) {
        try {
          b.off?.();
        } catch {}
      }
      this.bindings = [];
      this.active = false;
      this.exposedByStateId.clear();
    }
    allowAttrForKind(kind, mode) {
      switch (kind) {
        case "bool":
        case "enum":
        case "string":
        case "number.discrete":
          return true;
        case "number.range":
          return !!mode.allowContinuousAttr;
        default:
          return false;
      }
    }
    port = {
      isActive: () => this.active,
      getExposedStateMap: () => this.exposedByStateId
    };
  }

  // ../packages/modules/expose-state-web/src/create.ts
  function createExposeStateWebModule(ctx) {
    const { init, caps: caps8, deps } = ctx;
    return createModule({
      name: "expose-state-web",
      scope: "instance",
      init,
      caps: caps8,
      deps,
      build: ({ caps: caps9, deps: deps2 }) => {
        const impl = new ExposeStateWebModuleImpl(caps9, deps2);
        return {
          facade: {},
          port: impl.port,
          hooks: {
            onInstancePhase: (p) => impl.onInstancePhase(p),
            onMountPhase: (p, epoch) => impl.onMountPhase(p, epoch),
            afterRenderCommit: () => impl.afterRenderCommit(),
            dispose: () => impl.dispose()
          }
        };
      }
    });
  }
  var ExposeStateWebModuleDef = defineModule({
    name: "expose-state-web",
    resourceOwnership: "mixed",
    deps: ["expose-state"],
    optionalDeps: ["expose"],
    create: createExposeStateWebModule
  });
  // ../packages/modules/rule-expose-state-web/src/generated/lowered-variant-order.ts
  var LOWERED_VARIANT_RANK = ["dark", "hover", "active", "focus", "focus-visible", "disabled"];
  function compareLoweredVariants(a, b) {
    const ai = LOWERED_VARIANT_RANK.indexOf(a);
    const bi = LOWERED_VARIANT_RANK.indexOf(b);
    if (ai !== -1 || bi !== -1)
      return (ai === -1 ? 999 : ai) - (bi === -1 ? 999 : bi);
    return a.localeCompare(b);
  }
  function canonicalizeLoweredVariants(variants) {
    return Array.from(new Set(variants)).sort(compareLoweredVariants);
  }

  // ../packages/modules/rule-expose-state-web/src/caps.ts
  var RULE_EXPOSE_STATE_WEB_NATIVE_VARIANT_POLICY_CAP = cap("@proto.ui/rule-expose-state-web/nativeVariantPolicy");

  // ../packages/modules/rule-expose-state-web/src/create.ts
  function isStateMetaDeps(rule) {
    let hasLowerableDep = false;
    for (const dep of rule.deps) {
      if (dep.kind === "state" || dep.kind === "meta") {
        hasLowerableDep = true;
        continue;
      }
      return false;
    }
    return hasLowerableDep;
  }
  function extractConditions(expr) {
    switch (expr.type) {
      case "eq":
        if (expr.left.type === "state") {
          return [{ kind: "state", stateId: expr.left.id, literal: expr.right }];
        }
        if (expr.left.type === "meta" && expr.left.key === "colorScheme" && expr.right === "dark") {
          return [{ kind: "meta.dark" }];
        }
        return null;
      case "all": {
        const all = [];
        for (const e of expr.exprs) {
          const c = extractConditions(e);
          if (!c)
            return null;
          all.push(...c);
        }
        return all;
      }
      default:
        return null;
    }
  }
  function stripDataPrefix(attr) {
    return attr.startsWith("data-") ? attr.slice("data-".length) : attr;
  }
  function buildSemanticVariant(semantic, condition, allowNativeVariant) {
    if (!semantic || condition.kind !== "state" || condition.literal !== true)
      return null;
    let variant = null;
    switch (semantic) {
      case "@interaction/hovered":
        variant = "hover";
        break;
      case "@interaction/pressed":
        variant = "active";
        break;
      case "@interaction/disabled":
        variant = "disabled";
        break;
      case "@interaction/focused":
        variant = "focus";
        break;
      case "@interaction/focusVisible":
        variant = "focus-visible";
        break;
      default:
        variant = null;
    }
    if (!variant)
      return null;
    if (allowNativeVariant && !allowNativeVariant({ semantic, variant }))
      return null;
    return variant;
  }
  function buildVariant(condition, map, allowNativeVariant) {
    if (condition.kind === "meta.dark") {
      return "dark";
    }
    const binding = map.get(String(condition.stateId));
    if (!binding)
      return null;
    if (binding.kind === "number.range")
      return null;
    const semanticVariant = buildSemanticVariant(binding.semantic, condition, allowNativeVariant);
    if (semanticVariant)
      return semanticVariant;
    const attr = binding.attr;
    if (!attr)
      return null;
    const key = stripDataPrefix(attr);
    if (binding.kind === "bool") {
      if (condition.literal === true)
        return `data-[${key}]`;
      if (condition.literal === false)
        return `not-[data-${key}]`;
      return null;
    }
    if (condition.literal === null)
      return null;
    if (binding.kind === "enum" || binding.kind === "string" || binding.kind === "number.discrete") {
      return `data-[${key}=${String(condition.literal)}]`;
    }
    return null;
  }
  function isNegativeDataVariant(variant) {
    return /^not-\[data-[a-zA-Z0-9-]+\]$/.test(variant);
  }

  class RuleExposeStateWebImpl extends ModuleBase {
    rulePort;
    exposeStateWeb;
    feedbackPort;
    candidates = [];
    candidatesReady = false;
    optimizedIds = new Set;
    constructor(caps9, deps) {
      super(caps9);
      this.rulePort = deps.requirePort("rule");
      this.exposeStateWeb = deps.requirePort("expose-state-web");
      this.feedbackPort = deps.requirePort("feedback");
      this.rulePort.registerExtension({
        transformRules: (rules) => rules.filter((r) => !this.optimizedIds.has(r.id))
      });
    }
    onProtoPhase(phase) {
      super.onProtoPhase(phase);
      if (phase === "mounted")
        this.tryApply();
    }
    onMountPhase(phase, epoch) {
      super.onMountPhase(phase, epoch);
    }
    onCapsEpoch(_epoch) {
      this.tryApply();
    }
    afterRenderCommit() {
      this.tryApply();
    }
    collectCandidates() {
      const ir = this.rulePort.exportIR();
      const out = [];
      let order = 0;
      for (const r of ir) {
        if (!isStateMetaDeps(r))
          continue;
        const conditions = extractConditions(r.when);
        if (!conditions || conditions.length === 0)
          continue;
        if (r.intent.kind !== "ops")
          continue;
        const tokens = [];
        let ok = true;
        for (const op of r.intent.ops) {
          if (op.kind !== "feedback.style.use") {
            ok = false;
            break;
          }
          for (const h of op.handles) {
            if (!h || h.kind !== "tw") {
              ok = false;
              break;
            }
            tokens.push(...h.tokens);
          }
        }
        if (!ok || tokens.length === 0)
          continue;
        out.push({ id: r.id, order: order++, conditions, tokens });
      }
      return out;
    }
    getAllowNativeVariant() {
      return this.caps.has(RULE_EXPOSE_STATE_WEB_NATIVE_VARIANT_POLICY_CAP) ? this.caps.get(RULE_EXPOSE_STATE_WEB_NATIVE_VARIANT_POLICY_CAP) : null;
    }
    tryApply() {
      if (this.mountPhase === "detached" || this.mountPhase === "unmounting")
        return;
      const map = this.exposeStateWeb.getExposedStateMap() ?? new Map;
      const allowNativeVariant = this.getAllowNativeVariant();
      if (map.size === 0 && !allowNativeVariant)
        return;
      if (!this.candidatesReady) {
        this.candidates = this.collectCandidates();
        this.candidatesReady = true;
      }
      if (this.candidates.length === 0)
        return;
      const appliedIds = [];
      for (const c of this.candidates) {
        if (this.optimizedIds.has(c.id))
          continue;
        const variants = [];
        let ok = true;
        for (const cond of c.conditions) {
          const v = buildVariant(cond, map, allowNativeVariant);
          if (!v) {
            ok = false;
            break;
          }
          variants.push(v);
        }
        if (!ok || variants.length === 0)
          continue;
        if (variants.every(isNegativeDataVariant))
          continue;
        const prefix = canonicalizeLoweredVariants(variants).join(":");
        const tokens = c.tokens.map((t) => `${prefix}:${t}`);
        const handle = { kind: "tw", tokens };
        this.feedbackPort.useStyleUnsafe(handle);
        appliedIds.push(c.id);
      }
      for (const id of appliedIds)
        this.optimizedIds.add(id);
    }
  }
  function createRuleExposeStateWebModule(ctx) {
    const { init, caps: caps9, deps } = ctx;
    return createModule({
      name: "rule-expose-state-web",
      scope: "instance",
      init,
      caps: caps9,
      deps,
      build: ({ caps: caps10, deps: deps2 }) => {
        const impl = new RuleExposeStateWebImpl(caps10, deps2);
        return {
          facade: {},
          hooks: {
            onMountPhase: (p, epoch) => impl.onMountPhase(p, epoch),
            onProtoPhase: (p) => impl.onProtoPhase(p),
            afterRenderCommit: () => impl.afterRenderCommit()
          }
        };
      }
    });
  }
  var RuleExposeStateWebModuleDef = defineModule({
    name: "rule-expose-state-web",
    resourceOwnership: "mixed",
    deps: ["rule", "expose-state-web", "feedback"],
    create: createRuleExposeStateWebModule
  });
  // ../packages/modules/rule-meta/src/caps.ts
  var RULE_META_GET_CAP = cap("@proto.ui/rule-meta/get");
  // ../packages/modules/rule-meta/src/create.ts
  class RuleMetaModuleImpl extends ModuleBase {
    rulePort;
    constructor(caps10, deps) {
      super(caps10);
      this.rulePort = deps.requirePort("rule");
      this.rulePort.registerExtension({
        beforePlan: (ctx) => {
          if (ctx.readMeta)
            return { kind: "continue" };
          const getter = this.caps.has(RULE_META_GET_CAP) ? this.caps.get(RULE_META_GET_CAP) : null;
          if (!getter)
            return { kind: "continue" };
          ctx.readMeta = (key) => getter(key);
          return { kind: "continue" };
        }
      });
    }
    get(key) {
      const getter = this.caps.has(RULE_META_GET_CAP) ? this.caps.get(RULE_META_GET_CAP) : null;
      return getter ? getter(key) : undefined;
    }
  }
  function createRuleMetaModule(ctx) {
    const { init, caps: caps10, deps } = ctx;
    return createModule({
      name: "rule-meta",
      scope: "instance",
      init,
      caps: caps10,
      deps,
      build: ({ caps: caps11, deps: deps2 }) => {
        const impl = new RuleMetaModuleImpl(caps11, deps2);
        return {
          facade: {
            get: (key) => impl.get(key)
          },
          hooks: {
            onProtoPhase: (p) => impl.onProtoPhase(p)
          }
        };
      }
    });
  }
  var RuleMetaModuleDef = defineModule({
    name: "rule-meta",
    resourceOwnership: "mixed",
    deps: ["rule"],
    create: createRuleMetaModule
  });
  // ../packages/modules/rule/src/when-builder.ts
  function stateIdOf(s) {
    const id = s?.__stateId ?? s?.id ?? s;
    if (typeof id === "string" || typeof id === "number")
      return String(id);
    return `obj:${String(id)}`;
  }
  function createWhenBuilder(opts) {
    const deps = [];
    const depKeySet = new Set;
    const pushDep = (d) => {
      const key = d.kind === "prop" ? `prop:${String(d.key)}` : d.kind === "state" ? `state:${stateIdOf(d.id)}` : d.kind === "context" ? `context:${String(d.key)}` : `meta:${String(d.key)}`;
      if (depKeySet.has(key))
        return;
      depKeySet.add(key);
      deps.push(d);
    };
    const makeSignal = (left) => ({
      eq(lit) {
        return { type: "eq", left, right: lit };
      }
    });
    const w = {
      prop(key) {
        pushDep({ kind: "prop", key });
        return makeSignal({ type: "prop", key });
      },
      state(s) {
        const id = s?.__stateId ?? s?.id ?? s;
        pushDep({ kind: "state", id });
        if (opts?.onStateHandle)
          opts.onStateHandle(id, s);
        return makeSignal({ type: "state", id });
      },
      ctx(key) {
        pushDep({ kind: "context", key });
        return makeSignal({ type: "context", key });
      },
      meta(key) {
        pushDep({ kind: "meta", key });
        return makeSignal({ type: "meta", key });
      },
      all(...exprs) {
        return { type: "all", exprs };
      },
      any(...exprs) {
        return { type: "any", exprs };
      },
      not(expr) {
        return { type: "not", expr };
      },
      t() {
        return { type: "true" };
      },
      f() {
        return { type: "false" };
      }
    };
    return {
      w,
      getDeps: () => deps.slice()
    };
  }
  // ../packages/modules/rule/src/intent-builder.ts
  function createIntentBuilder() {
    const ops = [];
    const builder = {
      feedback: {
        style: {
          use: (...handles3) => {
            ops.push({ kind: "feedback.style.use", handles: handles3 });
          }
        }
      },
      state: (handle) => ({
        be(value) {
          ops.push({
            kind: "state.set",
            handle,
            value
          });
        }
      })
    };
    const exportIntent = () => ({ kind: "ops", ops: ops.slice() });
    return { builder, exportIntent };
  }
  // ../packages/modules/rule/src/compile.ts
  function attachDefaultReasons(ops, spec2) {
    return ops.map((op, idx) => {
      if (op.kind !== "state.set")
        return op;
      if (op.reason !== undefined)
        return op;
      return {
        ...op,
        reason: {
          kind: "rule",
          label: spec2.label,
          note: spec2.note,
          opIndex: idx
        }
      };
    });
  }
  function compileRule(spec2, opt) {
    const { w, getDeps } = createWhenBuilder({
      onStateHandle: opt?.registerStateHandle
    });
    const when = spec2.when(w);
    const { builder, exportIntent } = createIntentBuilder();
    spec2.intent(builder);
    const intent = exportIntent();
    const ops = attachDefaultReasons(intent.ops, spec2);
    return {
      label: spec2.label,
      note: spec2.note,
      deps: getDeps(),
      when,
      intent: { kind: "ops", ops }
    };
  }
  // ../packages/modules/rule/src/eval.ts
  function evalValue(v, ctx) {
    switch (v.type) {
      case "prop":
        return ctx.props[v.key];
      case "state":
        return ctx.readState ? ctx.readState(v.id) : undefined;
      case "context":
        return ctx.readContext ? ctx.readContext(v.key) : undefined;
      case "meta":
        return ctx.readMeta ? ctx.readMeta(v.key) : undefined;
    }
  }
  function evalExpr(e, ctx) {
    switch (e.type) {
      case "true":
        return true;
      case "false":
        return false;
      case "eq":
        return evalValue(e.left, ctx) === e.right;
      case "not":
        return !evalExpr(e.expr, ctx);
      case "all":
        for (const it of e.exprs)
          if (!evalExpr(it, ctx))
            return false;
        return true;
      case "any":
        for (const it of e.exprs)
          if (evalExpr(it, ctx))
            return true;
        return false;
    }
  }
  function evaluateRulesToPlan(rules, ctx) {
    const active = rules.map((r, idx) => ({ r, idx })).filter(({ r }) => evalExpr(r.when, ctx)).sort((a, b) => a.idx - b.idx);
    const tokens = [];
    for (const { r } of active) {
      if (r.intent.kind !== "ops")
        continue;
      for (const op of r.intent.ops) {
        if (op.kind === "feedback.style.use") {
          for (const h of op.handles) {
            if (!h || h.kind !== "tw") {
              throw new Error(`[rule] unsupported style handle in v0`);
            }
            tokens.push(...h.tokens);
          }
        }
      }
    }
    const merged = mergeTwTokensV0(tokens);
    return { kind: "style.tokens", tokens: merged.tokens };
  }
  // ../packages/modules/rule/src/impl.ts
  class RuleModuleImpl {
    rules = [];
    extensions = [];
    stateHandleById = new Map;
    nextRuleId = 1;
    deps = {};
    depsResolver;
    stateWatchOffs = [];
    stateWatchesInstalled = false;
    driverActive = false;
    unUseRuleStyle = null;
    define(spec2) {
      const ir = compileRule(spec2, {
        registerStateHandle: (id, handle2) => {
          if (handle2 && typeof handle2.get === "function") {
            this.stateHandleById.set(id, handle2);
          }
        }
      });
      const withId = { ...ir, id: this.nextRuleId++ };
      this.rules.push(withId);
      const handle = {
        id: withId.id,
        dispose: () => {
          const before = this.rules.length;
          this.rules = this.rules.filter((r) => r.id !== withId.id);
          if (this.rules.length === before)
            return;
          if (this.driverActive)
            this.evaluateAndApply();
        }
      };
      return handle;
    }
    exportIR() {
      return this.rules.slice();
    }
    resolveStateHandle(id) {
      return this.stateHandleById.get(id);
    }
    registerExtension(ext) {
      this.extensions.push(ext);
    }
    evaluate(ctx) {
      const readState = ctx.readState ?? ((id) => {
        const h = this.stateHandleById.get(id);
        return h ? h.get() : undefined;
      });
      const evalCtx = {
        ...ctx,
        readState
      };
      let rules = this.rules;
      for (const ext of this.extensions) {
        if (ext.transformRules) {
          rules = ext.transformRules(rules, evalCtx) ?? rules;
        }
      }
      for (const ext of this.extensions) {
        const before = ext.beforePlan?.(evalCtx);
        if (before?.kind === "short-circuit") {
          if (before.execute)
            before.execute(evalCtx);
          return { kind: "short-circuit", executed: !!before.execute };
        }
      }
      let plan = evaluateRulesToPlan(rules, evalCtx);
      for (const ext of this.extensions) {
        plan = ext.afterPlan ? ext.afterPlan(plan, evalCtx) : plan;
      }
      return { kind: "plan", plan };
    }
    attachExecutor(resolveDeps) {
      this.depsResolver = resolveDeps;
      this.deps = resolveDeps();
    }
    onProtoPhase(phase) {
      if (phase === "mounted") {
        this.ensureDeps();
        if (!this.stateWatchesInstalled)
          this.installStateWatches();
        this.driverActive = true;
        this.evaluateAndApply();
        return;
      }
      if (phase === "updated") {
        this.ensureDeps();
        if (!this.stateWatchesInstalled)
          this.installStateWatches();
        if (this.driverActive)
          this.evaluateAndApply();
        return;
      }
      if (phase === "unmounted") {
        this.stopDriver();
        return;
      }
    }
    onMountPhase(phase) {
      if (phase === "detached")
        this.stopDriver();
    }
    dispose() {
      this.stopDriver();
    }
    installStateWatches() {
      this.ensureDeps();
      const statePort = this.deps.statePort;
      if (!statePort)
        return;
      const ir = this.exportIR();
      const seen = new Set;
      for (const r of ir) {
        for (const dep of r.deps) {
          if (dep.kind !== "state")
            continue;
          if (seen.has(dep.id))
            continue;
          seen.add(dep.id);
          const h = this.resolveStateHandle(dep.id);
          if (!h)
            continue;
          const off = statePort.watch(h, () => {
            if (this.driverActive)
              this.evaluateAndApply();
          });
          this.stateWatchOffs.push(off);
        }
      }
      this.stateWatchesInstalled = true;
    }
    stopDriver() {
      this.driverActive = false;
      while (this.stateWatchOffs.length) {
        try {
          this.stateWatchOffs.pop()();
        } catch {}
      }
      this.stateWatchesInstalled = false;
      if (this.unUseRuleStyle) {
        try {
          this.unUseRuleStyle();
        } catch {}
        this.unUseRuleStyle = null;
      }
    }
    evaluateAndApply() {
      if (!this.driverActive)
        return;
      this.ensureDeps();
      const { propsFacade, propsPort, feedbackPort, contextFacade } = this.deps;
      if (!feedbackPort)
        return;
      propsPort?.syncFromHost();
      const props = propsFacade?.get?.() ?? {};
      const res = this.evaluate({
        props,
        readContext: (key) => {
          if (!contextFacade)
            return;
          return contextFacade.tryRead(key) ?? undefined;
        }
      });
      if (res.kind !== "plan" || res.plan.kind !== "style.tokens") {
        if (this.unUseRuleStyle) {
          this.unUseRuleStyle = feedbackPort.replaceStyleRuntime(this.unUseRuleStyle);
        }
        return;
      }
      const tokens = res.plan.tokens ?? [];
      if (tokens.length === 0 && !this.unUseRuleStyle)
        return;
      this.unUseRuleStyle = feedbackPort.replaceStyleRuntime(this.unUseRuleStyle, ...tokens.length > 0 ? [{ kind: "tw", tokens }] : []);
    }
    ensureDeps() {
      if (!this.depsResolver)
        return;
      const next = this.depsResolver();
      this.deps = {
        propsFacade: next.propsFacade ?? this.deps.propsFacade,
        propsPort: next.propsPort ?? this.deps.propsPort,
        statePort: next.statePort ?? this.deps.statePort,
        feedbackPort: next.feedbackPort ?? this.deps.feedbackPort,
        contextFacade: next.contextFacade ?? this.deps.contextFacade
      };
    }
  }
  // ../packages/modules/rule/src/create.ts
  function createRuleModule(ctx) {
    const { init, caps: caps11, deps } = ctx;
    return createModule({
      name: "rule",
      scope: "instance",
      init,
      caps: caps11,
      deps,
      build: () => {
        const impl = new RuleModuleImpl;
        const resolveDeps = () => ({
          propsFacade: deps.tryFacade("props"),
          propsPort: deps.tryPort("props"),
          statePort: deps.tryPort("state"),
          feedbackPort: deps.tryPort("feedback"),
          contextFacade: deps.tryFacade("context")
        });
        impl.attachExecutor(resolveDeps);
        return {
          facade: {
            rule: (spec2) => impl.define(spec2)
          },
          port: {
            exportIR: () => impl.exportIR(),
            resolveStateHandle: (id) => impl.resolveStateHandle(id),
            evaluate: (ctx2) => impl.evaluate(ctx2),
            registerExtension: (ext) => impl.registerExtension(ext)
          },
          hooks: {
            onMountPhase: (p) => impl.onMountPhase(p),
            onProtoPhase: (p) => impl.onProtoPhase(p),
            dispose: () => impl.dispose()
          }
        };
      }
    });
  }
  var RuleModuleDef = defineModule({
    name: "rule",
    resourceOwnership: "mixed",
    deps: [],
    optionalDeps: ["props", "state", "context", "feedback"],
    create: createRuleModule
  });
  // ../packages/modules/state/src/kernel.ts
  class StateKernel {
    nextId = 1;
    records = new Map;
    emitting = false;
    pending = [];
    define(name, spec2, defaultValue) {
      if (typeof name !== "string" || name.length === 0) {
        throw new Error(`[State] state name must be a non-empty string.`);
      }
      const id = this.nextId++;
      const rec = {
        id,
        name,
        semantic: name,
        spec: spec2,
        value: defaultValue,
        subscribers: new Set
      };
      this.records.set(id, rec);
      const h = {
        get: () => this.getById(id),
        setDefault: (v) => {
          this.setDefaultById(id, v);
        },
        set: (v, reason) => {
          this.setById(id, v, reason);
        }
      };
      h.__stateId = id;
      h.__stateName = name;
      h.__stateSemantic = name;
      h.__stateKind = spec2.kind;
      h.__stateSpec = spec2;
      return h;
    }
    subscribe(handle, cb) {
      const id = this.getIdFromHandle(handle);
      const rec = this.getRecord(id);
      rec.subscribers.add(cb);
      return () => rec.subscribers.delete(cb);
    }
    getSemantic(handle) {
      const id = this.getIdFromHandle(handle);
      return this.getRecord(id).semantic;
    }
    getKind(handle) {
      const id = this.getIdFromHandle(handle);
      return this.getRecord(id).spec.kind;
    }
    setInternal(handle, next, reason) {
      const id = this.getIdFromHandle(handle);
      this.setById(id, next, reason);
    }
    setDefaultInternal(handle, next) {
      const id = this.getIdFromHandle(handle);
      this.setDefaultById(id, next);
    }
    getById(id) {
      return this.getRecord(id).value;
    }
    setDefaultById(id, v) {
      const rec = this.getRecord(id);
      rec.value = v;
    }
    setById(id, next, reason) {
      const rec = this.getRecord(id);
      const prev = rec.value;
      if (Object.is(prev, next))
        return;
      rec.value = next;
      const emit = () => {
        const e = { type: "next", prev, next, reason };
        for (const cb of rec.subscribers)
          cb(e);
      };
      if (this.emitting) {
        this.pending.push(emit);
        return;
      }
      this.emitting = true;
      try {
        emit();
        while (this.pending.length) {
          const task = this.pending.shift();
          task();
        }
      } finally {
        this.emitting = false;
      }
    }
    getIdFromHandle(handle) {
      const id = handle.__stateId;
      if (!id) {
        throw new Error(`[StateKernel] expects handle created by this kernel`);
      }
      return id;
    }
    getRecord(id) {
      const rec = this.records.get(id);
      if (!rec)
        throw new Error(`[StateKernel] unknown state id: ${id}`);
      return rec;
    }
    dispose() {
      this.records.clear();
      this.pending = [];
      this.emitting = false;
    }
  }
  // ../packages/modules/state/src/impl.ts
  function opOf(semantic, method) {
    return `state(${semantic}).${method}`;
  }
  function getIdFromHandle(handle) {
    const id = handle.__stateId;
    if (!id)
      throw new Error(`[StateModule] expects handle created by state-kernel`);
    return id;
  }

  class StateModuleImpl {
    sys;
    kernel = new StateKernel;
    disposed = false;
    watchersById = new Map;
    kernelOffById = new Map;
    ctxStack = [];
    callbackDispatcher = null;
    constructor(sys) {
      this.sys = sys;
    }
    ensureAlive(op) {
      this.sys.ensureNotDisposed(op);
      if (this.disposed) {
        throw new Error(`[StateModule] disposed. op=${op}`);
      }
    }
    getCallbackCtx() {
      return this.sys.getCallbackCtx?.() ?? this.sys.getCurrentCallbackCtx?.() ?? this.sys.getRun?.() ?? undefined;
    }
    withCtx(ctx, fn) {
      this.ctxStack.push(ctx);
      try {
        return fn();
      } finally {
        this.ctxStack.pop();
      }
    }
    currentCtx() {
      return this.ctxStack.length ? this.ctxStack[this.ctxStack.length - 1] : undefined;
    }
    dispatchWatcher(cb, ctx, event2) {
      if (this.callbackDispatcher) {
        this.callbackDispatcher((callbackCtx) => cb(callbackCtx, event2));
        return;
      }
      cb(ctx, event2);
    }
    ensureKernelForwarder(id, handle) {
      if (this.kernelOffById.has(id))
        return;
      const off = this.kernel.subscribe(handle, (e) => {
        const watchers = this.watchersById.get(id);
        if (!watchers || watchers.size === 0)
          return;
        const ctx = this.currentCtx();
        const list = Array.from(watchers);
        for (const cb of list)
          this.dispatchWatcher(cb, ctx, e);
      });
      this.kernelOffById.set(id, off);
    }
    addWatcher(handle, cb) {
      const id = getIdFromHandle(handle);
      let set = this.watchersById.get(id);
      if (!set) {
        set = new Set;
        this.watchersById.set(id, set);
      }
      set.add(cb);
      this.ensureKernelForwarder(id, handle);
      return () => {
        const s = this.watchersById.get(id);
        if (!s)
          return;
        s.delete(cb);
        if (s.size === 0) {
          this.watchersById.delete(id);
          const off = this.kernelOffById.get(id);
          if (off) {
            off();
            this.kernelOffById.delete(id);
          }
        }
      };
    }
    emitDisconnect(handle) {
      const id = getIdFromHandle(handle);
      const watchers = this.watchersById.get(id);
      if (!watchers || watchers.size === 0)
        return;
      const e = { type: "disconnect", reason: "unmount" };
      const ctx = this.currentCtx();
      const list = Array.from(watchers);
      for (const cb of list)
        this.dispatchWatcher(cb, ctx, e);
    }
    wrapOwnedHandle(raw, semantic) {
      const wrapped = {
        get: () => {
          this.ensureAlive(opOf(semantic, "get"));
          return raw.get();
        },
        setDefault: (v) => {
          this.ensureAlive(opOf(semantic, "setDefault"));
          this.sys.ensureSetup(opOf(semantic, "setDefault"));
          return raw.setDefault(v);
        },
        set: (v, reason) => {
          this.ensureAlive(opOf(semantic, "set"));
          this.sys.ensureCallback(opOf(semantic, "set"));
          const ctx = this.getCallbackCtx();
          return this.withCtx(ctx, () => raw.set(v, reason));
        }
      };
      wrapped.__stateId = raw.__stateId;
      wrapped.__stateName = raw.__stateName ?? semantic;
      wrapped.__stateSemantic = raw.__stateSemantic ?? semantic;
      wrapped.__stateKind = raw.__stateKind;
      wrapped.__stateSpec = raw.__stateSpec;
      return wrapped;
    }
    port = {
      setCallbackDispatcher: (dispatch) => {
        this.callbackDispatcher = dispatch;
      },
      watch: (handle, cb) => {
        this.ensureAlive(`state.port.watch`);
        return this.addWatcher(handle, cb);
      },
      disconnect: (handle) => {
        this.ensureAlive(`state.port.disconnect`);
        this.emitDisconnect(handle);
      },
      set: (handle, value, reason, ctx) => {
        this.ensureAlive(`state.port.set`);
        return this.withCtx(ctx ?? this.getCallbackCtx(), () => this.kernel.setInternal(handle, value, reason));
      },
      setDefault: (handle, value) => {
        this.ensureAlive(`state.port.setDefault`);
        return this.kernel.setDefaultInternal(handle, value);
      },
      createObservedHandle: (handle) => {
        this.ensureAlive(`state.port.createObservedHandle`);
        const observed = {
          get: () => {
            this.ensureAlive(`state.port.createObservedHandle.get`);
            return handle.get();
          },
          watch: (cb) => {
            this.ensureAlive(`state.port.createObservedHandle.watch`);
            this.sys.ensureSetup(`state.port.createObservedHandle.watch`);
            return this.addWatcher(handle, cb);
          }
        };
        observed.__stateId = handle.__stateId;
        observed.__stateName = handle.__stateName;
        observed.__stateSemantic = handle.__stateSemantic;
        observed.__stateKind = handle.__stateKind;
        observed.__stateSpec = handle.__stateSpec;
        return observed;
      },
      createBorrowedHandle: (handle) => {
        this.ensureAlive(`state.port.createBorrowedHandle`);
        const borrowed = {
          get: () => {
            this.ensureAlive(`state.port.createBorrowedHandle.get`);
            return handle.get();
          },
          setDefault: (v) => {
            this.ensureAlive(`state.port.createBorrowedHandle.setDefault`);
            return handle.setDefault(v);
          },
          set: (v, reason) => {
            this.ensureAlive(`state.port.createBorrowedHandle.set`);
            const ctx = this.getCallbackCtx();
            return this.withCtx(ctx, () => handle.set(v, reason));
          },
          watch: (cb) => {
            this.ensureAlive(`state.port.createBorrowedHandle.watch`);
            this.sys.ensureSetup(`state.port.createBorrowedHandle.watch`);
            return this.addWatcher(handle, cb);
          }
        };
        borrowed.__stateId = handle.__stateId;
        borrowed.__stateName = handle.__stateName;
        borrowed.__stateSemantic = handle.__stateSemantic;
        borrowed.__stateKind = handle.__stateKind;
        borrowed.__stateSpec = handle.__stateSpec;
        return borrowed;
      }
    };
    facade = {
      bool: (semantic, defaultValue) => {
        const raw = this.kernel.define(semantic, { kind: "bool" }, defaultValue);
        return this.wrapOwnedHandle(raw, semantic);
      },
      enum: (semantic, defaultValue, spec2) => {
        const raw = this.kernel.define(semantic, { kind: "enum", ...spec2 }, defaultValue);
        return this.wrapOwnedHandle(raw, semantic);
      },
      string: (semantic, defaultValue, spec2 = {}) => {
        const raw = this.kernel.define(semantic, { kind: "string", ...spec2 }, defaultValue);
        return this.wrapOwnedHandle(raw, semantic);
      },
      numberRange: (semantic, defaultValue, spec2) => {
        const raw = this.kernel.define(semantic, { kind: "number.range", ...spec2 }, defaultValue);
        return this.wrapOwnedHandle(raw, semantic);
      },
      numberDiscrete: (semantic, defaultValue, spec2 = {}) => {
        const raw = this.kernel.define(semantic, { kind: "number.discrete", ...spec2 }, defaultValue);
        return this.wrapOwnedHandle(raw, semantic);
      }
    };
    dispose() {
      if (this.disposed)
        return;
      this.disposed = true;
      for (const [, watchers] of this.watchersById) {
        const e = { type: "disconnect", reason: "unmount" };
        const list = Array.from(watchers);
        for (const cb of list)
          cb(undefined, e);
      }
      for (const [, off] of this.kernelOffById)
        off();
      this.kernelOffById.clear();
      this.watchersById.clear();
      this.ctxStack = [];
      this.kernel.dispose();
    }
  }
  // ../packages/modules/state/src/create.ts
  function createStateModule(ctx) {
    const { init, caps: caps11, deps } = ctx;
    return createModule({
      name: "state",
      scope: "instance",
      init,
      caps: caps11,
      deps,
      build: ({ caps: caps12 }) => {
        const sys = caps12.get(SYS_CAP);
        const impl2 = new StateModuleImpl(sys);
        return {
          facade: impl2.facade,
          port: impl2.port,
          hooks: {
            dispose: () => impl2.dispose()
          }
        };
      }
    });
  }
  var StateModuleDef = defineModule({
    name: "state",
    resourceOwnership: "instance",
    deps: [],
    create: createStateModule
  });
  // ../packages/modules/state-interaction/src/create.ts
  class StateInteractionModuleImpl {
    stateFacade;
    statePort;
    eventPort;
    handles = new Map;
    constructor(stateFacade, statePort, eventPort) {
      this.stateFacade = stateFacade;
      this.statePort = statePort;
      this.eventPort = eventPort;
    }
    get(name) {
      const existing = this.handles.get(name);
      if (existing)
        return existing;
      const owned = this.stateFacade.bool(`@interaction/${name}`, false);
      owned.__stateName = name;
      const borrowed = this.statePort.createBorrowedHandle(owned);
      borrowed.__stateId = owned.__stateId;
      borrowed.__stateName = name;
      borrowed.__stateSemantic = owned.__stateSemantic;
      borrowed.__stateKind = owned.__stateKind;
      borrowed.__stateSpec = owned.__stateSpec;
      this.handles.set(name, borrowed);
      this.wireInteractionState(name, borrowed);
      return borrowed;
    }
    wireInteractionState(name, state2) {
      switch (name) {
        case "hovered": {
          this.eventPort.on("pointer.enter", () => {
            if (this.isDisabled())
              return;
            state2.set(true, "reason: state-interaction.pointer.enter => hovered");
          });
          this.eventPort.on("pointer.leave", () => {
            state2.set(false, "reason: state-interaction.pointer.leave => hovered");
          });
          this.eventPort.on("pointer.cancel", () => {
            state2.set(false, "reason: state-interaction.pointer.cancel => hovered");
          });
          return;
        }
        case "pressed": {
          this.eventPort.on("pointer.down", () => {
            if (this.isDisabled())
              return;
            state2.set(true, "reason: state-interaction.pointer.down => pressed");
          });
          this.eventPort.on("pointer.up", () => {
            state2.set(false, "reason: state-interaction.pointer.up => pressed");
          });
          this.eventPort.on("pointer.cancel", () => {
            state2.set(false, "reason: state-interaction.pointer.cancel => pressed");
          });
          this.eventPort.on("pointer.leave", () => {
            state2.set(false, "reason: state-interaction.pointer.leave => pressed");
          });
          this.eventPort.on("press.commit", () => {
            state2.set(false, "reason: state-interaction.press.commit => pressed");
          });
          return;
        }
        case "focused":
        case "focusVisible": {
          return;
        }
        case "disabled":
          state2.watch((_ctx, event2) => {
            if (event2?.type !== "next" || event2.next !== true)
              return;
            this.clearTransientInteractionStates("reason: state-interaction.disabled => reset");
          });
          return;
      }
    }
    isDisabled() {
      return this.handles.get("disabled")?.get?.() === true;
    }
    clearTransientInteractionStates(reason) {
      for (const name of ["hovered", "pressed"]) {
        this.handles.get(name)?.set?.(false, reason);
      }
    }
  }
  function createStateInteractionModule(ctx) {
    const { init, caps: caps11, deps } = ctx;
    return createModule({
      name: "state-interaction",
      scope: "instance",
      init,
      caps: caps11,
      deps,
      build: ({ deps: deps2 }) => {
        const stateFacade = deps2.requireFacade("state");
        const statePort = deps2.requirePort("state");
        const eventPort = deps2.requirePort("event");
        const impl3 = new StateInteractionModuleImpl(stateFacade, statePort, eventPort);
        return {
          facade: {
            get: (name) => impl3.get(name)
          }
        };
      }
    });
  }
  var StateInteractionModuleDef = defineModule({
    name: "state-interaction",
    resourceOwnership: "instance",
    deps: ["state", "event"],
    create: createStateInteractionModule
  });
  // ../packages/modules/state-accessibility/src/create.ts
  class StateAccessibilityModuleImpl {
    stateFacade;
    statePort;
    handles = new Map;
    constructor(stateFacade, statePort) {
      this.stateFacade = stateFacade;
      this.statePort = statePort;
    }
    get(name) {
      const existing = this.handles.get(name);
      if (existing)
        return existing;
      const owned = this.stateFacade.bool(`@accessibility/${name}`, false);
      owned.__stateName = name;
      const borrowed = this.statePort.createBorrowedHandle(owned);
      borrowed.__stateId = owned.__stateId;
      borrowed.__stateName = name;
      borrowed.__stateSemantic = owned.__stateSemantic;
      borrowed.__stateKind = owned.__stateKind;
      borrowed.__stateSpec = owned.__stateSpec;
      this.handles.set(name, borrowed);
      return borrowed;
    }
  }
  function createStateAccessibilityModule(ctx) {
    const { init, caps: caps11, deps } = ctx;
    return createModule({
      name: "state-accessibility",
      scope: "instance",
      init,
      caps: caps11,
      deps,
      build: ({ deps: deps2 }) => {
        const stateFacade = deps2.requireFacade("state");
        const statePort = deps2.requirePort("state");
        const impl3 = new StateAccessibilityModuleImpl(stateFacade, statePort);
        return {
          facade: {
            get: (name) => impl3.get(name)
          }
        };
      }
    });
  }
  var StateAccessibilityModuleDef = defineModule({
    name: "state-accessibility",
    resourceOwnership: "instance",
    deps: ["state"],
    create: createStateAccessibilityModule
  });
  // ../packages/modules/a11y/src/caps.ts
  var A11Y_PROJECT_CAP = cap("@proto.ui/a11y/project");
  // ../packages/modules/a11y/src/create.ts
  class A11yModuleImpl extends ModuleBase {
    statePort;
    ir = {
      states: new Map,
      actions: new Map,
      relations: new Map
    };
    stateWatchOffs = [];
    stateWatchesInstalled = false;
    constructor(caps11, statePort) {
      super(caps11);
      this.statePort = statePort;
    }
    facade = {
      id: (target) => {
        this.ensureSetup("def.a11y.id");
        this.ir.id = target;
        this.applyProjection();
      },
      role: (role) => {
        this.ensureSetup("def.a11y.role");
        this.ir.role = role;
        this.applyProjection();
      },
      name: (value) => {
        this.ensureSetup("def.a11y.name");
        this.ir.name = { kind: "text", value };
        this.applyProjection();
      },
      nameFromContent: () => {
        this.ensureSetup("def.a11y.nameFromContent");
        this.ir.name = { kind: "content" };
        this.applyProjection();
      },
      description: (value) => {
        this.ensureSetup("def.a11y.description");
        this.ir.description = { kind: "text", value };
        this.applyProjection();
      },
      state: (key, handle) => {
        this.ensureSetup("def.a11y.state");
        this.ir.states.set(key, { key, handle });
        this.applyProjection();
      },
      action: (key, spec2 = {}) => {
        this.ensureSetup("def.a11y.action");
        this.ir.actions.set(key, { ...spec2 });
        this.applyProjection();
      },
      relation: (key, spec2) => {
        this.ensureSetup("def.a11y.relation");
        this.ir.relations.set(key, { key, spec: { ...spec2 } });
        this.applyProjection();
      },
      tree: (patch) => {
        this.ensureSetup("def.a11y.tree");
        this.ir.tree = { ...this.ir.tree ?? {}, ...patch };
        this.applyProjection();
      }
    };
    port = {
      getSnapshot: () => this.getSnapshot(),
      getIR: () => ({
        role: this.ir.role,
        id: this.ir.id,
        name: cloneTextAlternative(this.ir.name),
        description: cloneTextAlternative(this.ir.description),
        states: new Map(this.ir.states),
        actions: new Map(this.ir.actions),
        relations: new Map(this.ir.relations),
        tree: this.ir.tree ? { ...this.ir.tree } : undefined
      })
    };
    onProtoPhase(phase) {
      super.onProtoPhase(phase);
      if (phase === "mounted" || phase === "updated") {
        this.installStateWatches();
        this.applyProjection();
      }
      if (phase === "unmounted") {
        this.dispose();
      }
    }
    onMountPhase(phase, epoch) {
      super.onMountPhase(phase, epoch);
      if (phase === "detached")
        this.dispose();
    }
    afterRenderCommit() {
      this.installStateWatches();
      this.applyProjection();
    }
    dispose() {
      while (this.stateWatchOffs.length) {
        this.stateWatchOffs.pop()?.();
      }
      this.stateWatchesInstalled = false;
    }
    ensureSetup(op) {
      this.sys.ensureSetup(op);
    }
    installStateWatches() {
      if (this.stateWatchesInstalled)
        return;
      for (const binding of this.ir.states.values()) {
        const off = this.statePort.watch(binding.handle, () => {
          this.applyProjection();
        });
        this.stateWatchOffs.push(off);
      }
      if (isState(this.ir.id)) {
        const off = this.statePort.watch(this.ir.id, () => {
          this.applyProjection();
        });
        this.stateWatchOffs.push(off);
      }
      if (isState(this.ir.role)) {
        const off = this.statePort.watch(this.ir.role, () => {
          this.applyProjection();
        });
        this.stateWatchOffs.push(off);
      }
      if (this.ir.name?.kind === "text" && isState(this.ir.name.value)) {
        const off = this.statePort.watch(this.ir.name.value, () => {
          this.applyProjection();
        });
        this.stateWatchOffs.push(off);
      }
      if (this.ir.description?.kind === "text" && isState(this.ir.description.value)) {
        const off = this.statePort.watch(this.ir.description.value, () => {
          this.applyProjection();
        });
        this.stateWatchOffs.push(off);
      }
      for (const binding of this.ir.relations.values()) {
        if (!isState(binding.spec.target))
          continue;
        const off = this.statePort.watch(binding.spec.target, () => {
          this.applyProjection();
        });
        this.stateWatchOffs.push(off);
      }
      if (isState(this.ir.tree?.hidden)) {
        const off = watchState(this.statePort, this.ir.tree.hidden, () => {
          this.applyProjection();
        });
        this.stateWatchOffs.push(off);
      }
      if (isState(this.ir.tree?.mergeChildren)) {
        const off = watchState(this.statePort, this.ir.tree.mergeChildren, () => {
          this.applyProjection();
        });
        this.stateWatchOffs.push(off);
      }
      this.stateWatchesInstalled = true;
    }
    getSnapshot() {
      const states = {};
      for (const [key, binding] of this.ir.states) {
        states[key] = binding.handle.get();
      }
      const relations = {};
      const relationModes = {};
      for (const [key, binding] of this.ir.relations) {
        const target = binding.spec.target;
        relations[key] = isState(target) ? target.get() : target;
        if (binding.spec.mode === "append")
          relationModes[key] = "append";
      }
      const tree = this.ir.tree ? Object.fromEntries(Object.entries({
        hidden: isState(this.ir.tree.hidden) ? this.ir.tree.hidden.get() : this.ir.tree.hidden,
        mergeChildren: isState(this.ir.tree.mergeChildren) ? this.ir.tree.mergeChildren.get() : this.ir.tree.mergeChildren
      }).filter(([, value]) => typeof value !== "undefined")) : undefined;
      return {
        id: isState(this.ir.id) ? this.ir.id.get() : this.ir.id,
        role: isState(this.ir.role) ? this.ir.role.get() : this.ir.role,
        name: resolveTextAlternative(this.ir.name),
        description: resolveTextAlternative(this.ir.description),
        states,
        actions: Object.fromEntries(this.ir.actions),
        relations,
        ...Object.keys(relationModes).length ? { relationModes } : {},
        tree
      };
    }
    applyProjection() {
      if (this.mountPhase === "detached" || this.mountPhase === "unmounting")
        return;
      if (!this.caps.has(A11Y_PROJECT_CAP))
        return;
      this.caps.get(A11Y_PROJECT_CAP)(this.getSnapshot());
    }
  }
  function watchState(statePort, handle, callback) {
    if ("watch" in handle && typeof handle.watch === "function") {
      return handle.watch(() => callback());
    }
    return statePort.watch(handle, () => callback());
  }
  function isState(value) {
    return !!value && typeof value === "object" && typeof value.get === "function";
  }
  function cloneTextAlternative(value) {
    if (!value)
      return;
    return value.kind === "text" ? { kind: "text", value: value.value } : { kind: "content" };
  }
  function resolveTextAlternative(value) {
    if (!value)
      return;
    if (value.kind === "content")
      return { kind: "content" };
    return {
      kind: "text",
      value: isState(value.value) ? value.value.get() || "" : value.value
    };
  }
  function createA11yModule(ctx) {
    const { init, caps: caps11, deps } = ctx;
    return createModule({
      name: "a11y",
      scope: "instance",
      init,
      caps: caps11,
      deps,
      build: ({ caps: caps12, deps: deps2 }) => {
        const impl3 = new A11yModuleImpl(caps12, deps2.requirePort("state"));
        return {
          facade: impl3.facade,
          port: impl3.port,
          hooks: {
            onInstancePhase: (p) => impl3.onInstancePhase(p),
            onMountPhase: (p, epoch) => impl3.onMountPhase(p, epoch),
            onProtoPhase: (p) => impl3.onProtoPhase(p),
            afterRenderCommit: () => impl3.afterRenderCommit(),
            dispose: () => impl3.dispose()
          }
        };
      }
    });
  }
  var A11yModuleDef = defineModule({
    name: "a11y",
    resourceOwnership: "mixed",
    deps: ["state"],
    create: createA11yModule
  });
  // ../packages/modules/collection/src/create.ts
  var DEFAULT_POSITION = Object.freeze({
    index: -1,
    total: 0,
    first: false,
    last: false
  });

  class CollectionModuleImpl extends ModuleBase {
    anatomy;
    providerConfig = null;
    itemConfig = null;
    constructor(caps12, anatomy2) {
      super(caps12);
      this.anatomy = anatomy2;
    }
    providerHandle = {
      configure: (config) => this.configureProvider(config),
      readItems: () => this.readProviderItems(),
      readCount: () => this.readProviderCount(),
      subscribe: (cb) => this.subscribeProvider(cb)
    };
    itemHandle = {
      configure: (config) => this.configureItem(config),
      readPosition: () => this.readItemPosition(),
      buildSnapshot: (meta) => this.buildItemSnapshot(meta),
      subscribe: (cb) => this.subscribeItem(cb)
    };
    facade = {
      getCollection: () => this.providerHandle,
      getCollectionItem: () => this.itemHandle
    };
    port = {
      configureProvider: (config) => this.configureProvider(config),
      configureItem: (config) => this.configureItem(config),
      readProviderItems: () => this.readProviderItems(),
      readProviderCount: () => this.readProviderCount(),
      readItemPosition: () => this.readItemPosition(),
      buildItemSnapshot: (meta) => this.buildItemSnapshot(meta),
      subscribeProvider: (cb) => this.subscribeProvider(cb),
      subscribeItem: (cb) => this.subscribeItem(cb)
    };
    ensureSetup(op) {
      this.sys.ensureSetup(op);
    }
    configureProvider(config) {
      this.ensureSetup("collection.configureProvider");
      this.providerConfig = Object.freeze({ ...config });
    }
    configureItem(config) {
      this.ensureSetup("collection.configureItem");
      this.itemConfig = Object.freeze({ ...config });
    }
    readProviderItems() {
      const config = this.providerConfig;
      if (!config)
        return [];
      const parts = this.anatomy.order.partsOf(config.family, config.itemRole, {
        missing: "empty"
      });
      const total = parts.length;
      return parts.map((part, index) => ({
        ...readItemSnapshot(part, config.itemMetaExposeKey),
        index,
        total,
        first: index === 0,
        last: index === total - 1
      }));
    }
    readProviderCount() {
      const config = this.providerConfig;
      if (!config)
        return 0;
      return this.anatomy.order.partsOf(config.family, config.itemRole, {
        missing: "empty"
      }).length;
    }
    readItemPosition() {
      const config = this.itemConfig;
      if (!config)
        return DEFAULT_POSITION;
      const index = this.anatomy.order.indexOfSelf(config.family, config.role, {
        missing: "null"
      });
      const parts = this.anatomy.order.partsOf(config.family, config.role, {
        missing: "null"
      });
      if (index == null || parts == null)
        return DEFAULT_POSITION;
      const total = parts.length;
      return {
        index,
        total,
        first: index === 0 && total > 0,
        last: index >= 0 && index === total - 1
      };
    }
    buildItemSnapshot(meta) {
      return {
        ...meta,
        ...this.readItemPosition()
      };
    }
    subscribeProvider(cb) {
      const config = this.providerConfig;
      if (!config)
        return () => {};
      return this.anatomy.subscribeOrder(config.family, cb);
    }
    subscribeItem(cb) {
      const config = this.itemConfig;
      if (!config)
        return () => {};
      return this.anatomy.subscribeOrder(config.family, cb);
    }
  }
  function readItemSnapshot(part, exposeKey) {
    const value = part.getExpose(exposeKey);
    if (!value)
      return {};
    if (typeof value === "function") {
      const next = value();
      return next && typeof next === "object" ? next : {};
    }
    return typeof value === "object" ? value : {};
  }
  function createCollectionModule(ctx) {
    const { init, caps: caps12, deps } = ctx;
    const anatomy2 = deps.requirePort("anatomy");
    return createModule({
      name: "collection",
      scope: "instance",
      init,
      caps: caps12,
      deps,
      build: ({ caps: caps13 }) => {
        const impl3 = new CollectionModuleImpl(caps13, anatomy2);
        return {
          facade: impl3.facade,
          port: impl3.port,
          hooks: {
            onProtoPhase: (p) => impl3.onProtoPhase(p)
          }
        };
      }
    });
  }
  var CollectionModuleDef = defineModule({
    name: "collection",
    resourceOwnership: "instance",
    deps: ["anatomy"],
    create: createCollectionModule
  });
  // ../packages/modules/context/src/caps.ts
  var CONTEXT_INSTANCE_TOKEN_CAP = cap("@proto.ui/context/instanceToken");
  var CONTEXT_PARENT_CAP = cap("@proto.ui/context/getParent");
  // ../packages/modules/context/src/center.ts
  class ContextCenter {
    providers = new Map;
    subscriptions = new Map;
    callbackQueue = [];
    provide(instance, key, value) {
      let byKey = this.providers.get(key);
      if (!byKey) {
        byKey = new Map;
        this.providers.set(key, byKey);
      }
      if (byKey.has(instance)) {
        throw new Error(`[Context] duplicate provide for key: ${key?.debugName ?? "(unknown)"}`);
      }
      byKey.set(instance, value);
    }
    unprovide(instance, key) {
      const byKey = this.providers.get(key);
      if (!byKey)
        return;
      byKey.delete(instance);
      if (byKey.size === 0)
        this.providers.delete(key);
    }
    getProviderValue(instance, key) {
      const byKey = this.providers.get(key);
      if (!byKey)
        return null;
      return byKey.get(instance) ?? null;
    }
    subscribe(instance, key, mode, cb) {
      let byInstance = this.subscriptions.get(instance);
      if (!byInstance) {
        byInstance = new Map;
        this.subscriptions.set(instance, byInstance);
      }
      const existing = byInstance.get(key);
      if (existing) {
        if (existing.mode !== mode) {
          throw new Error(`[Context] subscription mode mismatch for key: ${key?.debugName ?? "(unknown)"}`);
        }
        if (cb)
          existing.callbacks.push(cb);
        return;
      }
      byInstance.set(key, {
        mode,
        callbacks: cb ? [cb] : []
      });
    }
    hasSubscription(instance, key, mode) {
      const byInstance = this.subscriptions.get(instance);
      if (!byInstance)
        return false;
      const rec = byInstance.get(key);
      if (!rec)
        return false;
      if (!mode)
        return true;
      return rec.mode === mode;
    }
    resolveProvider(consumer, key, getParent) {
      const byKey = this.providers.get(key);
      if (!byKey)
        return null;
      let cur = consumer;
      while (cur) {
        if (byKey.has(cur))
          return cur;
        cur = getParent(cur);
      }
      return null;
    }
    updateFromProvider(provider, key, next, ctx, getParent) {
      const byKey = this.providers.get(key);
      if (!byKey || !byKey.has(provider)) {
        throw new Error(`[Context] update on missing provider for key: ${key?.debugName ?? "(unknown)"}`);
      }
      const prev = byKey.get(provider);
      byKey.set(provider, next);
      this.dispatch(provider, key, next, prev, ctx, getParent);
    }
    updateFromConsumer(consumer, key, next, ctx, getParent) {
      const provider = this.resolveProvider(consumer, key, getParent);
      if (!provider)
        return false;
      this.updateFromProvider(provider, key, next, ctx, getParent);
      return true;
    }
    dispatch(provider, key, next, prev, ctx, getParent) {
      this.callbackQueue = [];
      for (const [instance, byKey] of this.subscriptions.entries()) {
        const rec = byKey.get(key);
        if (!rec || rec.callbacks.length === 0)
          continue;
        const bound = this.resolveProvider(instance, key, getParent);
        if (bound !== provider)
          continue;
        const task = {
          instance,
          key,
          next,
          prev,
          callbackCount: rec.callbacks.length
        };
        this.callbackQueue.push(task);
        for (const cb of rec.callbacks) {
          cb(ctx, next, prev);
        }
      }
      this.callbackQueue = [];
    }
    dumpProviders() {
      const out = [];
      for (const [key, byInstance] of this.providers.entries()) {
        for (const [instance, value] of byInstance.entries()) {
          out.push({ instance, key, value });
        }
      }
      return out;
    }
    dumpSubscriptions() {
      const out = [];
      for (const [instance, byKey] of this.subscriptions.entries()) {
        for (const [key, rec] of byKey.entries()) {
          out.push({
            instance,
            key,
            mode: rec.mode,
            callbackCount: rec.callbacks.length
          });
        }
      }
      return out;
    }
    dumpCallbackQueue() {
      return [...this.callbackQueue];
    }
    removeInstance(instance) {
      this.subscriptions.delete(instance);
      for (const [key, byInstance] of this.providers.entries()) {
        byInstance.delete(instance);
        if (byInstance.size === 0)
          this.providers.delete(key);
      }
    }
  }
  var CONTEXT_CENTER = new ContextCenter;

  // ../packages/modules/context/src/impl.ts
  var ERR2 = {
    PHASE: "CONTEXT_PHASE_VIOLATION",
    PROVIDER_MISSING: "CONTEXT_PROVIDER_MISSING",
    SUB_REQUIRED: "CONTEXT_SUBSCRIPTION_REQUIRED",
    DUP_PROVIDE: "CONTEXT_DUPLICATE_PROVIDE",
    DISCONNECTED: "CONTEXT_DISCONNECTED",
    VALUE_INVALID: "CONTEXT_VALUE_INVALID"
  };
  function contextError(code, message) {
    const err = new Error(message);
    err.code = code;
    return err;
  }
  function isPlainObject2(v) {
    if (!v || typeof v !== "object")
      return false;
    return Object.getPrototypeOf(v) === Object.prototype;
  }
  function validateJsonValue(value, seen) {
    if (value === null)
      return;
    const t = typeof value;
    if (t === "string" || t === "number" || t === "boolean")
      return;
    if (t === "undefined" || t === "function" || t === "symbol" || t === "bigint") {
      throw contextError(ERR2.VALUE_INVALID, `[Context] illegal value type: ${t}`);
    }
    if (Array.isArray(value)) {
      if (seen.has(value)) {
        throw contextError(ERR2.VALUE_INVALID, `[Context] circular reference detected`);
      }
      seen.add(value);
      for (const item of value)
        validateJsonValue(item, seen);
      seen.delete(value);
      return;
    }
    if (!isPlainObject2(value)) {
      throw contextError(ERR2.VALUE_INVALID, `[Context] value must be JSON-serializable`);
    }
    if (seen.has(value)) {
      throw contextError(ERR2.VALUE_INVALID, `[Context] circular reference detected`);
    }
    seen.add(value);
    for (const k of Object.keys(value)) {
      const v = value[k];
      validateJsonValue(v, seen);
    }
    seen.delete(value);
  }
  function assertJsonObject(value, op) {
    if (value === null) {
      throw contextError(ERR2.VALUE_INVALID, `[Context] ${op} does not allow null as context value`);
    }
    if (!isPlainObject2(value)) {
      throw contextError(ERR2.VALUE_INVALID, `[Context] ${op} requires a plain JSON object`);
    }
    validateJsonValue(value, new WeakSet);
  }

  class ContextModuleImpl extends ModuleBase {
    prototypeName;
    callbackDispatcher = null;
    constructor(caps12, prototypeName) {
      super(caps12);
      this.prototypeName = prototypeName;
    }
    provide(key, defaultValue) {
      this.guardSetupOnly("def.context.provide");
      const self = this.getSelfToken();
      assertJsonObject(defaultValue, "provide");
      try {
        CONTEXT_CENTER.provide(self, key, defaultValue);
      } catch {
        throw contextError(ERR2.DUP_PROVIDE, `[Context] duplicate provide for key: ${key?.debugName ?? "(unknown)"}`);
      }
    }
    subscribe(key, onChange) {
      this.guardSetupOnly("def.context.subscribe");
      const self = this.getSelfToken();
      const getParent = this.getParentGetter();
      const provider = CONTEXT_CENTER.resolveProvider(self, key, getParent);
      if (!provider) {
        throw contextError(ERR2.PROVIDER_MISSING, `[Context] provider missing for key: ${key?.debugName ?? "(unknown)"}`);
      }
      let active = true;
      const wrapped = typeof onChange === "function" ? (ctx, next, prev) => {
        if (!active)
          return;
        if (this.callbackDispatcher) {
          this.callbackDispatcher((callbackCtx) => {
            onChange(callbackCtx, next, prev);
          });
          return;
        }
        onChange(ctx, next, prev);
      } : undefined;
      CONTEXT_CENTER.subscribe(self, key, "required", wrapped);
      return () => {
        active = false;
      };
    }
    trySubscribe(key, onChange) {
      this.guardSetupOnly("def.context.trySubscribe");
      const self = this.getSelfToken();
      let active = true;
      const wrapped = typeof onChange === "function" ? (ctx, next, prev) => {
        if (!active)
          return;
        if (this.callbackDispatcher) {
          this.callbackDispatcher((callbackCtx) => {
            onChange(callbackCtx, next, prev);
          });
          return;
        }
        onChange(ctx, next, prev);
      } : undefined;
      CONTEXT_CENTER.subscribe(self, key, "optional", wrapped);
      return () => {
        active = false;
      };
    }
    read(key) {
      this.guardRuntimeOnly("run.context.read");
      const self = this.getSelfToken();
      this.ensureSubscribed(self, key, "required", "read");
      const getParent = this.getParentGetter();
      const provider = CONTEXT_CENTER.resolveProvider(self, key, getParent);
      if (!provider) {
        throw contextError(ERR2.DISCONNECTED, `[Context] provider missing for key: ${key?.debugName ?? "(unknown)"}`);
      }
      const value = CONTEXT_CENTER.getProviderValue(provider, key);
      if (!value) {
        throw contextError(ERR2.DISCONNECTED, `[Context] provider missing for key: ${key?.debugName ?? "(unknown)"}`);
      }
      return value;
    }
    tryRead(key) {
      this.guardRuntimeOnly("run.context.tryRead");
      const self = this.getSelfToken();
      this.ensureSubscribed(self, key, "optional", "tryRead");
      const getParent = this.getParentGetter();
      const provider = CONTEXT_CENTER.resolveProvider(self, key, getParent);
      if (!provider)
        return null;
      return CONTEXT_CENTER.getProviderValue(provider, key) ?? null;
    }
    update(key, next) {
      this.guardCallbackOnly("run.context.update");
      const self = this.getSelfToken();
      const selfProvidesKey = CONTEXT_CENTER.getProviderValue(self, key) !== null;
      if (!selfProvidesKey) {
        this.ensureSubscribedAny(self, key, "update");
      }
      const getParent = this.getParentGetter();
      const provider = selfProvidesKey ? self : CONTEXT_CENTER.resolveProvider(self, key, getParent);
      if (!provider) {
        throw contextError(ERR2.DISCONNECTED, `[Context] provider missing for key: ${key?.debugName ?? "(unknown)"}`);
      }
      const prev = CONTEXT_CENTER.getProviderValue(provider, key);
      if (!prev) {
        throw contextError(ERR2.DISCONNECTED, `[Context] provider missing for key: ${key?.debugName ?? "(unknown)"}`);
      }
      const resolved = typeof next === "function" ? next(prev) : next;
      assertJsonObject(resolved, "update");
      const ctx = this.sys.getCallbackCtx();
      CONTEXT_CENTER.updateFromProvider(provider, key, resolved, ctx, getParent);
    }
    tryUpdate(key, next) {
      this.guardCallbackOnly("run.context.tryUpdate");
      const self = this.getSelfToken();
      this.ensureSubscribed(self, key, "optional", "tryUpdate");
      const getParent = this.getParentGetter();
      const provider = CONTEXT_CENTER.resolveProvider(self, key, getParent);
      if (!provider)
        return false;
      const prev = CONTEXT_CENTER.getProviderValue(provider, key);
      if (!prev)
        return false;
      const resolved = typeof next === "function" ? next(prev) : next;
      assertJsonObject(resolved, "update");
      const ctx = this.sys.getCallbackCtx();
      CONTEXT_CENTER.updateFromProvider(provider, key, resolved, ctx, getParent);
      return true;
    }
    onProtoPhase(phase) {
      super.onProtoPhase(phase);
      if (phase === "unmounted") {
        this.dispose();
      }
    }
    dispose() {
      const self = this.tryGetSelfToken();
      if (self) {
        CONTEXT_CENTER.removeInstance(self);
      }
    }
    portDumpProviders() {
      return CONTEXT_CENTER.dumpProviders();
    }
    portDumpSubscriptions() {
      return CONTEXT_CENTER.dumpSubscriptions();
    }
    portDumpCallbackQueue() {
      return CONTEXT_CENTER.dumpCallbackQueue();
    }
    setCallbackDispatcher(dispatch) {
      this.callbackDispatcher = dispatch;
    }
    resolveScope(key, consumer) {
      const from = consumer ?? this.tryGetSelfToken();
      if (!from)
        return null;
      return CONTEXT_CENTER.resolveProvider(from, key, this.getParentGetter());
    }
    guardSetupOnly(op) {
      if (this.sys.execPhase() !== "setup") {
        throw contextError(ERR2.PHASE, `[Context] illegal phase for ${op}: ${this.sys.execPhase()}`);
      }
    }
    guardCallbackOnly(op) {
      if (this.sys.execPhase() !== "callback") {
        throw contextError(ERR2.PHASE, `[Context] illegal phase for ${op}: ${this.sys.execPhase()}`);
      }
    }
    guardRuntimeOnly(op) {
      try {
        this.sys.ensureRuntime(op);
      } catch {
        throw contextError(ERR2.PHASE, `[Context] illegal phase for ${op}: ${this.sys.execPhase()}`);
      }
    }
    ensureSubscribed(instance, key, mode, op) {
      if (!CONTEXT_CENTER.hasSubscription(instance, key, mode)) {
        throw contextError(ERR2.SUB_REQUIRED, `[Context] ${op} requires ${mode} subscription: ${key?.debugName ?? "(unknown)"}`);
      }
    }
    ensureSubscribedAny(instance, key, op) {
      if (!CONTEXT_CENTER.hasSubscription(instance, key)) {
        throw contextError(ERR2.SUB_REQUIRED, `[Context] ${op} requires prior subscription: ${key?.debugName ?? "(unknown)"}`);
      }
    }
    getParentGetter() {
      if (!this.caps.has(CONTEXT_PARENT_CAP)) {
        throw contextError(ERR2.PROVIDER_MISSING, `[Context] host caps missing: parent getter (${this.prototypeName})`);
      }
      const fn = this.caps.get(CONTEXT_PARENT_CAP);
      if (typeof fn !== "function") {
        throw contextError(ERR2.PROVIDER_MISSING, `[Context] host caps invalid: parent getter (${this.prototypeName})`);
      }
      return fn;
    }
    getSelfToken() {
      if (!this.caps.has(CONTEXT_INSTANCE_TOKEN_CAP)) {
        throw contextError(ERR2.PROVIDER_MISSING, `[Context] host caps missing: instance token (${this.prototypeName})`);
      }
      return this.caps.get(CONTEXT_INSTANCE_TOKEN_CAP);
    }
    tryGetSelfToken() {
      if (!this.caps.has(CONTEXT_INSTANCE_TOKEN_CAP))
        return null;
      return this.caps.get(CONTEXT_INSTANCE_TOKEN_CAP);
    }
  }

  // ../packages/modules/context/src/create.ts
  function createContextModule(ctx) {
    const { init, caps: caps12, deps } = ctx;
    return createModule({
      name: "context",
      scope: "singleton",
      init,
      caps: caps12,
      deps,
      build: ({ init: init2, caps: caps13 }) => {
        const impl3 = new ContextModuleImpl(caps13, init2.prototypeName);
        return {
          facade: {
            provide: (key, value) => impl3.provide(key, value),
            subscribe: (key, cb) => impl3.subscribe(key, cb),
            trySubscribe: (key, cb) => impl3.trySubscribe(key, cb),
            read: (key) => impl3.read(key),
            tryRead: (key) => impl3.tryRead(key),
            update: (key, next) => impl3.update(key, next),
            tryUpdate: (key, next) => impl3.tryUpdate(key, next)
          },
          hooks: {
            onProtoPhase: (p) => impl3.onProtoPhase(p),
            dispose: () => impl3.dispose()
          },
          port: {
            setCallbackDispatcher: (dispatch) => impl3.setCallbackDispatcher(dispatch),
            resolveScope: (key, consumer) => impl3.resolveScope(key, consumer),
            dumpProviders: () => impl3.portDumpProviders(),
            dumpSubscriptions: () => impl3.portDumpSubscriptions(),
            dumpCallbackQueue: () => impl3.portDumpCallbackQueue()
          }
        };
      }
    });
  }
  var ContextModuleDef = defineModule({
    name: "context",
    resourceOwnership: "instance",
    deps: [],
    create: createContextModule
  });
  // ../packages/modules/as-trigger/src/caps.ts
  var AS_TRIGGER_INSTANCE_CAP = cap("@proto.ui/as-trigger/instanceToken");
  var AS_TRIGGER_PARENT_CAP = cap("@proto.ui/as-trigger/getParent");
  var AS_TRIGGER_GET_PROTO_CAP = cap("@proto.ui/as-trigger/getPrototype");
  var AS_TRIGGER_MERGE_GROUP_CAP = cap("@proto.ui/as-trigger/mergeGroup");
  var AS_TRIGGER_GET_GROUP_EVENT_TARGET_CAP = cap("@proto.ui/as-trigger/getGroupEventTarget");

  // ../packages/modules/as-trigger/src/impl.ts
  var TRIGGER_OWNER_MARK = Symbol.for("@proto.ui/as-trigger/confirm-owner");

  class AsTriggerModuleImpl extends ModuleBase {
    prototypeName;
    eventPort;
    constructor(caps13, prototypeName, eventPort) {
      super(caps13);
      this.prototypeName = prototypeName;
      this.eventPort = eventPort;
    }
    ensureSetup(op) {
      this.sys?.ensureSetup(op);
      if (!this.sys && this.protoPhase !== "setup") {
        throw illegalPhase(op, this.protoPhase, {
          prototypeName: this.prototypeName
        });
      }
    }
    getInstanceToken() {
      if (!this.caps.has(AS_TRIGGER_INSTANCE_CAP)) {
        throw capUnavailable(AS_TRIGGER_INSTANCE_CAP.id, {
          prototypeName: this.prototypeName
        });
      }
      return this.caps.get(AS_TRIGGER_INSTANCE_CAP);
    }
    getParentGetter() {
      if (!this.caps.has(AS_TRIGGER_PARENT_CAP)) {
        throw capUnavailable(AS_TRIGGER_PARENT_CAP.id, {
          prototypeName: this.prototypeName
        });
      }
      return this.caps.get(AS_TRIGGER_PARENT_CAP);
    }
    getPrototypeGetter() {
      if (!this.caps.has(AS_TRIGGER_GET_PROTO_CAP)) {
        throw capUnavailable(AS_TRIGGER_GET_PROTO_CAP.id, {
          prototypeName: this.prototypeName
        });
      }
      return this.caps.get(AS_TRIGGER_GET_PROTO_CAP);
    }
    apply() {
      this.ensureSetup("asTrigger.apply");
      const self = this.getInstanceToken();
      const getParent = this.getParentGetter();
      const getPrototype = this.getPrototypeGetter();
      let cur = getParent(self);
      let groupAnchor = self;
      const groupMembers = [self];
      while (cur) {
        const curProto = getPrototype(cur);
        if (!curProto)
          break;
        const trace = curProto.__asHooks;
        const hasTriggerMark = !!cur && (typeof cur === "object" || typeof cur === "function") && !!cur[TRIGGER_OWNER_MARK];
        const hasTriggerTrace = Array.isArray(trace) ? trace.some((e) => e?.name === "as-trigger" || e?.name === "asTrigger") : false;
        const hasTrigger = hasTriggerMark || hasTriggerTrace;
        if (!hasTrigger)
          break;
        groupAnchor = cur;
        groupMembers.push(cur);
        cur = getParent(cur);
      }
      if (this.caps.has(AS_TRIGGER_MERGE_GROUP_CAP)) {
        const mergeGroup = this.caps.get(AS_TRIGGER_MERGE_GROUP_CAP);
        for (const member of groupMembers)
          mergeGroup(member, groupAnchor);
      } else if (self && (typeof self === "object" || typeof self === "function")) {
        self[TRIGGER_OWNER_MARK] = groupAnchor === self ? true : groupAnchor;
      }
      const eventTarget = this.caps.has(AS_TRIGGER_GET_GROUP_EVENT_TARGET_CAP) ? this.caps.get(AS_TRIGGER_GET_GROUP_EVENT_TARGET_CAP)(self) ?? this.caps.get(AS_TRIGGER_GET_GROUP_EVENT_TARGET_CAP)(groupAnchor) : self;
      if (!eventTarget) {
        throw capUnavailable(AS_TRIGGER_GET_GROUP_EVENT_TARGET_CAP.id, {
          prototypeName: this.prototypeName
        });
      }
      this.eventPort.redirectSemanticRoot(eventTarget);
    }
  }

  // ../packages/modules/as-trigger/src/create.ts
  function createAsTriggerModule(ctx) {
    const { init, caps: caps13, deps } = ctx;
    const eventPort = deps.requirePort("event");
    return createModule({
      name: "as-trigger",
      scope: "instance",
      init,
      caps: caps13,
      deps,
      build: ({ init: init2, caps: caps14 }) => {
        const impl3 = new AsTriggerModuleImpl(caps14, init2.prototypeName, eventPort);
        return {
          facade: {
            apply: () => impl3.apply()
          },
          hooks: {
            onProtoPhase: (p) => impl3.onProtoPhase(p)
          }
        };
      }
    });
  }
  var AsTriggerModuleDef = defineModule({
    name: "as-trigger",
    resourceOwnership: "instance",
    deps: ["event"],
    create: createAsTriggerModule
  });
  // ../packages/modules/focus/src/caps.ts
  var FOCUS_ROOT_TARGET_CAP = cap("@proto.ui/focus/getRootTarget");
  var FOCUS_TARGET_READY_CAP = cap("@proto.ui/focus/subscribeTargetReady");
  var FOCUS_INSTANCE_TOKEN_CAP = cap("@proto.ui/focus/instanceToken");
  var FOCUS_PARENT_CAP = cap("@proto.ui/focus/getParent");
  var FOCUS_IS_NATIVELY_FOCUSABLE_CAP = cap("@proto.ui/focus/isNativelyFocusable");
  var FOCUS_SET_FOCUSABLE_CAP = cap("@proto.ui/focus/setFocusable");
  var FOCUS_REQUEST_FOCUS_CAP = cap("@proto.ui/focus/requestFocus");
  var FOCUS_BLUR_CAP = cap("@proto.ui/focus/blur");
  var FOCUS_RESOLVE_ENTRY_TARGET_CAP = cap("@proto.ui/focus/resolveEntryTarget");
  var FOCUS_SET_ENTRY_FOCUSABLE_CAP = cap("@proto.ui/focus/setEntryFocusable");
  var FOCUS_RUN_IN_CALLBACK_CAP = cap("@proto.ui/focus/runInCallback");
  // ../packages/modules/focus/src/center.ts
  class FocusCenter {
    entries = new Map;
    activeScopes = [];
    lastFocusedByScope = new Map;
    currentFocused = null;
    pendingRovingEntries = new Map;
    upsert(entry) {
      this.entries.set(entry.instance, entry);
      this.fulfillPendingRovingEntries(entry.instance);
    }
    remove(instance) {
      this.entries.delete(instance);
      this.pendingRovingEntries.delete(instance);
      if (this.currentFocused === instance)
        this.currentFocused = null;
      this.lastFocusedByScope.delete(instance);
      for (const [scope, focused] of this.lastFocusedByScope) {
        if (focused === instance)
          this.lastFocusedByScope.delete(scope);
      }
      for (let i = this.activeScopes.length - 1;i >= 0; i--) {
        if (this.activeScopes[i]?.scope === instance || this.activeScopes[i]?.previous === instance) {
          this.activeScopes.splice(i, 1);
        }
      }
    }
    detach(instance) {
      this.entries.delete(instance);
      if (this.currentFocused === instance)
        this.currentFocused = null;
      for (const [scope, focused] of this.lastFocusedByScope) {
        if (focused === instance)
          this.lastFocusedByScope.delete(scope);
      }
    }
    resolveKeyedRovingProvider(instance, groupKey, getParent) {
      let cur = instance;
      while (cur) {
        const entry = this.entries.get(cur);
        if (entry?.getRovingConfig().key === groupKey) {
          return cur;
        }
        cur = getParent(cur);
      }
      return null;
    }
    resolveNearestRovingProvider(instance, getParent) {
      let cur = getParent(instance);
      while (cur) {
        const entry = this.entries.get(cur);
        if (entry?.isRovingProvider())
          return cur;
        cur = getParent(cur);
      }
      return null;
    }
    compareEntries(a, b) {
      const aEl = a.getRootTarget();
      const bEl = b.getRootTarget();
      if (!aEl || !bEl || aEl === bEl)
        return 0;
      const pos = aEl.compareDocumentPosition(bEl);
      if (pos & Node.DOCUMENT_POSITION_FOLLOWING)
        return -1;
      if (pos & Node.DOCUMENT_POSITION_PRECEDING)
        return 1;
      return 0;
    }
    isDescendantOf(entry, ancestor) {
      if (entry.instance === ancestor.instance)
        return true;
      let cur = entry.getParent(entry.instance);
      while (cur) {
        if (cur === ancestor.instance)
          return true;
        cur = entry.getParent(cur);
      }
      return false;
    }
    getTopActiveScope() {
      for (let i = this.activeScopes.length - 1;i >= 0; i--) {
        const entry = this.entries.get(this.activeScopes[i].scope);
        if (entry?.isScopeProvider())
          return entry;
        if (this.pendingRovingEntries.has(this.activeScopes[i].scope))
          return null;
        this.activeScopes.splice(i, 1);
      }
      return null;
    }
    getFocusedEntry() {
      if (this.currentFocused) {
        const entry = this.entries.get(this.currentFocused);
        if (entry?.getFacts().focused)
          return entry;
        this.currentFocused = null;
      }
      return Array.from(this.entries.values()).find((entry) => entry.getFacts().focused) ?? null;
    }
    clearOtherFocusedEntries(next, reason) {
      for (const entry of this.entries.values()) {
        if (entry.instance === next.instance)
          continue;
        if (!entry.isFocusable())
          continue;
        const facts = entry.getFacts();
        if (!facts.focused && !facts.focusVisible && !facts.active && !entry.hasPendingFocus()) {
          continue;
        }
        entry.clearFocus(reason);
      }
    }
    getScopeMembers(scope) {
      if (!scope.isScopeProvider())
        return [];
      const members = Array.from(this.entries.values()).filter((entry) => {
        if (!entry.isFocusable())
          return false;
        if (entry.instance === scope.instance)
          return false;
        const focusable = entry.getFocusableConfig();
        if (focusable.disabled)
          return false;
        return this.isDescendantOf(entry, scope);
      });
      return this.dedupeSharedHostTargets(members.sort((a, b) => this.compareEntries(a, b)));
    }
    dedupeSharedHostTargets(entries) {
      const deduped = [];
      for (const entry of entries) {
        const target = entry.getRootTarget();
        const existingIndex = target ? deduped.findIndex((candidate) => candidate.getRootTarget() === target) : -1;
        if (existingIndex < 0) {
          deduped.push(entry);
          continue;
        }
        const existing = deduped[existingIndex];
        if (this.isDescendantOf(entry, existing)) {
          deduped[existingIndex] = entry;
        }
      }
      return deduped;
    }
    requestFocusAllowed(entry) {
      const scope = this.getTopActiveScope();
      if (!scope)
        return true;
      return this.isDescendantOf(entry, scope);
    }
    requestFocusOutcome(entry, options, behavior) {
      if (!entry.getRootTarget()) {
        return entry.requestFocus(options, behavior);
      }
      if (!behavior?.bypassGate && !this.requestFocusAllowed(entry)) {
        const scope = this.getTopActiveScope();
        entry.pushWarning(`[Focus] requestFocus ignored: active scope ${String(scope?.getScopeConfig().key?.meta?.debugLabel ?? scope?.instance ?? "unknown")} does not contain the requesting focus target.`);
        return "rejected";
      }
      const outcome = entry.requestFocus(options, behavior);
      if (outcome === "rejected")
        return "rejected";
      if (outcome === "pending")
        return "pending";
      this.clearOtherFocusedEntries(entry, options?.reason ?? "focus.request");
      if (behavior?.syncFacts !== false) {
        this.currentFocused = entry.instance;
      }
      this.noteFocused(entry);
      return "applied";
    }
    requestFocus(entry, options, behavior) {
      return this.requestFocusOutcome(entry, options, behavior) !== "rejected";
    }
    noteFocused(entry) {
      this.clearOtherFocusedEntries(entry, "focus.host:focus");
      this.currentFocused = entry.instance;
      for (const record of this.activeScopes) {
        const scope = this.entries.get(record.scope);
        if (!scope?.isScopeProvider())
          continue;
        if (this.isDescendantOf(entry, scope)) {
          this.lastFocusedByScope.set(scope.instance, entry.instance);
        }
      }
    }
    activateScope(scope, options) {
      if (!scope.isScopeProvider())
        return false;
      this.entries.set(scope.instance, scope);
      const existingIndex = this.activeScopes.findIndex((record) => record.scope === scope.instance);
      if (existingIndex >= 0) {
        this.activeScopes.splice(existingIndex, 1);
      }
      const focused = this.getFocusedEntry();
      const previous = focused && !this.isDescendantOf(focused, scope) ? focused.instance : null;
      this.activeScopes.push({ scope: scope.instance, previous });
      scope.setScopeActive(true);
      if (scope.getScopeConfig().entry === "manual")
        return true;
      const target = scope.isRovingProvider() ? this.getRovingMembers(scope)[0] ?? null : this.getScopeMembers(scope)[0] ?? null;
      if (target) {
        this.requestFocus(target, options ?? { reason: "programmatic" }, { syncFacts: true });
        return true;
      }
      if (scope.getScopeConfig().emptyPolicy === "container")
        return true;
      this.activeScopes.pop();
      scope.setScopeActive(false);
      return false;
    }
    deactivateScope(scope, options) {
      let index = -1;
      for (let i = this.activeScopes.length - 1;i >= 0; i--) {
        if (this.activeScopes[i]?.scope === scope.instance) {
          index = i;
          break;
        }
      }
      if (index < 0) {
        scope.setScopeActive(false);
        return false;
      }
      const [{ previous }] = this.activeScopes.splice(index, 1);
      scope.setScopeActive(false);
      for (const entry of this.entries.values()) {
        if (entry.instance === scope.instance || !entry.isFocusable())
          continue;
        if (!this.isDescendantOf(entry, scope))
          continue;
        const facts = entry.getFacts();
        if (!facts.focused && !facts.focusVisible && !facts.active && !entry.hasPendingFocus()) {
          continue;
        }
        entry.clearFocus(options?.reason ?? "focus.scope.deactivate");
        if (this.currentFocused === entry.instance)
          this.currentFocused = null;
      }
      const previousEntry = previous ? this.entries.get(previous) ?? null : null;
      if (previousEntry) {
        this.requestFocus(previousEntry, options ?? { reason: "programmatic" }, {
          bypassGate: true,
          syncFacts: true
        });
      }
      return true;
    }
    isScopeActive(scope) {
      return this.activeScopes.some((record) => record.scope === scope.instance);
    }
    isTopActiveScope(scope) {
      return this.getTopActiveScope()?.instance === scope.instance;
    }
    focusInScope(scope, op) {
      if (!scope.isScopeProvider() || !this.isTopActiveScope(scope))
        return false;
      const members = this.getScopeMembers(scope);
      if (members.length === 0)
        return false;
      const focused = this.getFocusedEntry();
      const remembered = this.lastFocusedByScope.get(scope.instance) ?? null;
      const currentIndex = focused ? members.findIndex((entry) => entry.instance === focused.instance) : remembered ? members.findIndex((entry) => entry.instance === remembered) : -1;
      const delta = op === "next" ? 1 : -1;
      let nextIndex = currentIndex >= 0 ? currentIndex + delta : op === "next" ? 0 : members.length - 1;
      if (scope.getScopeConfig().loop) {
        nextIndex = (nextIndex + members.length) % members.length;
      } else {
        nextIndex = Math.max(0, Math.min(members.length - 1, nextIndex));
      }
      const target = members[nextIndex] ?? null;
      if (!target)
        return false;
      this.requestFocus(target, { reason: "keyboard" }, { syncFacts: false });
      return true;
    }
    getRovingMembers(provider) {
      const groupKey = provider.getRovingConfig().key;
      if (!provider.isRovingProvider())
        return [];
      const members = Array.from(this.entries.values()).filter((entry) => {
        if (!entry.isFocusable())
          return false;
        if (entry.instance === provider.instance)
          return false;
        const focusable = entry.getFocusableConfig();
        if (focusable.disabled)
          return false;
        if (focusable.groupKey) {
          if (!groupKey || focusable.groupKey !== groupKey)
            return false;
          const resolved2 = this.resolveKeyedRovingProvider(entry.instance, focusable.groupKey, entry.getParent);
          if (resolved2 === provider.instance)
            return true;
          return resolved2 === null && entry.getParent(entry.instance) === null;
        }
        const resolved = this.resolveNearestRovingProvider(entry.instance, entry.getParent);
        return resolved === provider.instance;
      });
      return this.dedupeSharedHostTargets(members.sort((a, b) => this.compareEntries(a, b)));
    }
    focusInRoving(provider, op, options) {
      this.entries.set(provider.instance, provider);
      const members = this.getRovingMembers(provider);
      if (members.length === 0) {
        if (options?.entryRequest?.defer && (op === "first" || op === "last" || op === "selected")) {
          this.pendingRovingEntries.set(provider.instance, {
            op,
            options: options.entryRequest
          });
          return true;
        }
        return false;
      }
      const focusedIndex = members.findIndex((entry) => entry.getFacts().focused);
      const activeIndex = members.findIndex((entry) => entry.getFacts().rovingActive);
      if (options?.requireFocusedMember && focusedIndex < 0)
        return false;
      const currentIndex = focusedIndex >= 0 ? focusedIndex : activeIndex;
      const loop = provider.getRovingConfig().loop;
      let target = null;
      if (op === "first") {
        target = members[0] ?? null;
      } else if (op === "selected") {
        const selected = members.filter((entry) => entry.getFacts().rovingSelected);
        if (selected.length > 1) {
          provider.pushWarning(`[Focus] roving set has multiple selected members; focusSelected uses the first member in host order.`);
        }
        target = selected[0] ?? members[0] ?? null;
      } else if (op === "last") {
        target = members[members.length - 1] ?? null;
      } else if (currentIndex >= 0) {
        const delta = op === "next" ? 1 : -1;
        let nextIndex = currentIndex + delta;
        if (loop) {
          nextIndex = (nextIndex + members.length) % members.length;
        }
        target = members[nextIndex] ?? null;
      } else {
        target = op === "prev" ? members[members.length - 1] ?? null : members[0] ?? null;
      }
      if (!target)
        return false;
      this.pendingRovingEntries.delete(provider.instance);
      const outcome = this.requestFocusOutcome(target, {
        reason: options?.entryRequest?.reason ?? "keyboard",
        preventScroll: options?.entryRequest?.preventScroll
      }, { syncFacts: true });
      if (outcome === "pending" && options?.entryRequest?.defer) {
        this.pendingRovingEntries.set(provider.instance, {
          op,
          options: options.entryRequest,
          attempted: target.instance
        });
      } else {
        this.pendingRovingEntries.delete(provider.instance);
      }
      return outcome !== "rejected";
    }
    fulfillPendingRovingEntries(changedInstance) {
      for (const [instance, pending] of [...this.pendingRovingEntries]) {
        const provider = this.entries.get(instance);
        if (!provider)
          continue;
        if (!provider.isRovingProvider()) {
          this.pendingRovingEntries.delete(instance);
          continue;
        }
        if (pending.attempted && this.entries.has(pending.attempted) && changedInstance !== pending.attempted && changedInstance !== provider.instance) {
          continue;
        }
        if (this.getRovingMembers(provider).length === 0)
          continue;
        this.focusInRoving(provider, pending.op, { entryRequest: pending.options });
      }
    }
  }
  var FOCUS_CENTER = new FocusCenter;

  // ../packages/modules/focus/src/create.ts
  var DEFAULT_FOCUSABLE_CONFIG = Object.freeze({
    autoFocus: false,
    disabled: false,
    navParticipation: "auto"
  });
  var DEFAULT_ENTRY_CONFIG = Object.freeze({
    strategy: "self",
    fallback: "self",
    disabled: false
  });
  var DEFAULT_SCOPE_CONFIG = Object.freeze({
    trap: false,
    loop: false,
    navigation: "tab",
    orientation: "vertical",
    entry: "first",
    restore: "none",
    emptyPolicy: "none"
  });
  var DEFAULT_ROVING_CONFIG = Object.freeze({
    loop: false,
    navigation: "none",
    orientation: "vertical",
    entry: "first",
    selectOnFocus: false
  });
  function mergeMeta(prev, next) {
    if (!next)
      return prev;
    return Object.freeze({
      ...prev ?? {},
      ...next
    });
  }
  function pushOverrideWarning(warnings, owner, field, prev, next) {
    if (typeof prev === "undefined" || Object.is(prev, next))
      return;
    warnings.push(`[Focus] ${owner}.${field} overridden: ${String(prev)} -> ${String(next)}`);
  }
  function readNativeFocusVisible(el) {
    if (!el || typeof el.matches !== "function")
      return { supported: false, value: false };
    try {
      return { supported: true, value: el.matches(":focus-visible") };
    } catch {
      return { supported: false, value: false };
    }
  }

  class FocusModuleImpl extends ModuleBase {
    eventPort;
    statePort;
    focusableConfig = DEFAULT_FOCUSABLE_CONFIG;
    focusableDeclared = false;
    entryDeclared = false;
    entryConfig = DEFAULT_ENTRY_CONFIG;
    scopeDeclared = false;
    rovingDeclared = false;
    rovingConfig = DEFAULT_ROVING_CONFIG;
    scopeConfig = DEFAULT_SCOPE_CONFIG;
    prototypeName;
    warnings = [];
    didAutoFocus = false;
    keyboardModality = false;
    currentHostFocusTarget = null;
    hostFocusTargetGeneration = 0;
    hostEventsWired = false;
    scopeEventsWired = false;
    rovingEventsWired = false;
    pendingFocusRequest;
    offTargetReady;
    lastHostFocusableTarget = null;
    lastHostEntryTarget = null;
    focusedOwned;
    focusVisibleOwned;
    focusableOwned;
    activeOwned;
    hasFocusedOwned;
    focusedState;
    focusVisibleState;
    focusableState;
    activeState;
    hasFocusedState;
    rovingSelected = false;
    rovingActive = false;
    focusableHandle;
    entryHandle;
    scopeHandle;
    rovingHandle;
    constructor(caps13, prototypeName, eventPort, statePort, stateFacade) {
      super(caps13);
      this.eventPort = eventPort;
      this.statePort = statePort;
      this.prototypeName = prototypeName;
      this.focusedOwned = stateFacade.bool("@focus/focused", false);
      this.focusVisibleOwned = stateFacade.bool("@focus/focusVisible", false);
      this.focusableOwned = stateFacade.bool("@focus/focusable", false);
      this.activeOwned = stateFacade.bool("@focus/active", false);
      this.hasFocusedOwned = stateFacade.bool("@focus/hasFocused", false);
      this.focusedOwned.__stateName = "focused";
      this.focusVisibleOwned.__stateName = "focusVisible";
      this.focusableOwned.__stateName = "focusable";
      this.activeOwned.__stateName = "active";
      this.hasFocusedOwned.__stateName = "hasFocused";
      this.focusedState = statePort.createObservedHandle(this.focusedOwned);
      this.focusVisibleState = statePort.createObservedHandle(this.focusVisibleOwned);
      this.focusableState = statePort.createObservedHandle(this.focusableOwned);
      this.activeState = statePort.createObservedHandle(this.activeOwned);
      this.hasFocusedState = statePort.createObservedHandle(this.hasFocusedOwned);
      this.focusableHandle = {
        focused: this.focusedState,
        focusVisible: this.focusVisibleState,
        focusable: this.focusableState,
        focus: (options) => this.requestFocus(options),
        focusSelf: (options) => this.requestNativeFocus(options),
        blur: () => this.blur(),
        isFocused: () => this.focusedState.get(),
        setDisabled: (disabled) => this.setDisabled(disabled),
        setNavParticipation: (navParticipation) => this.setNavParticipation(navParticipation),
        setRovingStatus: (status) => this.setRovingStatus(status),
        configure: (patch) => this.configureFocusable(patch)
      };
      this.entryHandle = {
        focus: (options) => this.requestEntryFocus(options),
        setDisabled: (disabled) => this.setEntryDisabled(disabled),
        configure: (patch) => this.configureEntry(patch)
      };
      this.scopeHandle = {
        active: this.activeState,
        hasFocused: this.hasFocusedState,
        focusFirst: () => this.focusFirst(),
        focusLast: () => this.focusLast(),
        focusNext: () => this.focusNext(),
        focusPrev: () => this.focusPrev(),
        focusSelected: () => this.focusSelected(),
        restoreFocus: () => this.restoreFocus(),
        activate: (options) => this.activateScope(options),
        deactivate: (options) => this.deactivateScope(options),
        isActive: () => this.isScopeActive(),
        configure: (patch) => this.configureScope(patch),
        getRoving: () => this.getRoving()
      };
      this.rovingHandle = {
        active: this.activeState,
        hasFocused: this.hasFocusedState,
        focusFirst: (options) => this.focusFirst(options),
        focusLast: (options) => this.focusLast(options),
        focusNext: () => this.focusNext(),
        focusPrev: () => this.focusPrev(),
        focusSelected: (options) => this.focusSelected(options),
        configure: (patch) => this.configureRoving(patch),
        setLoop: (loop) => this.setRovingLoop(loop),
        setOrientation: (orientation) => this.setRovingOrientation(orientation)
      };
      this.syncTargetReadySubscription();
    }
    onCapsEpoch() {
      this.syncTargetReadySubscription();
    }
    syncTargetReadySubscription() {
      this.offTargetReady?.();
      this.offTargetReady = undefined;
      if (!this.caps.has(FOCUS_TARGET_READY_CAP))
        return;
      this.offTargetReady = this.caps.get(FOCUS_TARGET_READY_CAP)(() => {
        this.runInCallbackScope(() => {
          this.syncCenter();
          this.syncHostFocusable();
          this.syncHostEntry();
          if (this.fulfillPendingFocus())
            return;
          if (this.focusedState.get()) {
            this.requestNativeFocus({
              reason: this.focusVisibleState.get() ? "keyboard" : "programmatic"
            });
          }
        });
      });
    }
    ensureSetup(op) {
      this.sys?.ensureSetup(op);
      if (!this.sys && this.protoPhase !== "setup") {
        throw illegalPhase(op, this.protoPhase, {
          prototypeName: this.prototypeName
        });
      }
    }
    getRootTarget() {
      if (!this.caps.has(FOCUS_ROOT_TARGET_CAP))
        return null;
      const getter = this.caps.get(FOCUS_ROOT_TARGET_CAP);
      return getter?.() ?? null;
    }
    getCallbackCtx() {
      return this.sys?.getCallbackCtx?.() ?? undefined;
    }
    setFocusState(handle, next, reason, options) {
      if (Object.is(handle.get(), next))
        return;
      if (options?.defaultOnly) {
        this.statePort.setDefault(handle, next);
        return;
      }
      this.statePort.set(handle, next, reason, this.getCallbackCtx());
    }
    getSelfToken() {
      if (!this.caps.has(FOCUS_INSTANCE_TOKEN_CAP))
        return this.getRootTarget();
      return this.caps.get(FOCUS_INSTANCE_TOKEN_CAP);
    }
    getParentGetter() {
      if (!this.caps.has(FOCUS_PARENT_CAP))
        return () => null;
      return this.caps.get(FOCUS_PARENT_CAP);
    }
    runInCallbackScope(fn) {
      if (this.caps.has(FOCUS_RUN_IN_CALLBACK_CAP)) {
        this.caps.get(FOCUS_RUN_IN_CALLBACK_CAP)(fn);
        return;
      }
      fn();
    }
    createCenterEntry() {
      const self = this.getSelfToken();
      if (!self)
        return null;
      return {
        instance: self,
        getParent: this.getParentGetter(),
        isFocusable: () => this.focusableDeclared,
        isScopeProvider: () => this.scopeDeclared,
        isRovingProvider: () => this.rovingDeclared,
        getFocusableConfig: () => this.focusableConfig,
        getScopeConfig: () => this.scopeConfig,
        getRovingConfig: () => this.rovingConfig,
        getFacts: () => this.getFacts(),
        getRootTarget: () => this.getRootTarget(),
        requestFocus: (options, behavior) => {
          let outcome = "rejected";
          this.runInCallbackScope(() => {
            if (behavior?.syncFacts === false) {
              outcome = this.requestNativeFocusDirect(options);
              return;
            }
            outcome = this.requestFocusDirect(options);
          });
          return outcome;
        },
        hasPendingFocus: () => !!this.pendingFocusRequest,
        clearFocus: (reason) => {
          this.runInCallbackScope(() => this.clearFocus(reason));
        },
        setScopeActive: (active) => this.setScopeActive(active),
        pushWarning: (message) => this.warnings.push(message)
      };
    }
    syncCenter() {
      const entry = this.createCenterEntry();
      if (!entry)
        return;
      FOCUS_CENTER.upsert(entry);
    }
    syncHostFocusable() {
      const target = this.getRootTarget();
      if (this.lastHostFocusableTarget && this.lastHostFocusableTarget !== target) {
        if (this.caps.has(FOCUS_SET_FOCUSABLE_CAP)) {
          this.caps.get(FOCUS_SET_FOCUSABLE_CAP)(this.lastHostFocusableTarget, false);
        } else {
          this.lastHostFocusableTarget.tabIndex = -1;
        }
      }
      this.lastHostFocusableTarget = target;
      if (!target)
        return;
      const enabled = this.focusableDeclared && !this.focusableConfig.disabled && this.focusableConfig.navParticipation !== "none";
      const isNative = this.caps.has(FOCUS_IS_NATIVELY_FOCUSABLE_CAP) ? this.caps.get(FOCUS_IS_NATIVELY_FOCUSABLE_CAP)(target) : false;
      if (this.caps.has(FOCUS_SET_FOCUSABLE_CAP)) {
        this.caps.get(FOCUS_SET_FOCUSABLE_CAP)(target, enabled, {
          programmatic: this.focusableDeclared && !this.focusableConfig.disabled
        });
        return;
      }
      if (!enabled && isNative) {
        target.tabIndex = -1;
      }
    }
    syncHostEntry() {
      const target = this.getRootTarget();
      if (this.lastHostEntryTarget && this.lastHostEntryTarget !== target) {
        if (this.caps.has(FOCUS_SET_ENTRY_FOCUSABLE_CAP)) {
          this.caps.get(FOCUS_SET_ENTRY_FOCUSABLE_CAP)(this.lastHostEntryTarget, this.entryConfig, false);
        } else if (this.caps.has(FOCUS_SET_FOCUSABLE_CAP)) {
          this.caps.get(FOCUS_SET_FOCUSABLE_CAP)(this.lastHostEntryTarget, false);
        }
      }
      this.lastHostEntryTarget = target;
      if (!target || !this.entryDeclared)
        return;
      const enabled = !this.entryConfig.disabled;
      if (this.focusableDeclared && !this.focusableConfig.disabled)
        return;
      if (this.caps.has(FOCUS_SET_ENTRY_FOCUSABLE_CAP)) {
        this.caps.get(FOCUS_SET_ENTRY_FOCUSABLE_CAP)(target, this.entryConfig, enabled);
        return;
      }
      if (this.caps.has(FOCUS_SET_FOCUSABLE_CAP)) {
        if (enabled && this.entryConfig.fallback === "self") {
          this.caps.get(FOCUS_SET_FOCUSABLE_CAP)(target, true);
        } else if (!this.focusableDeclared || this.focusableConfig.disabled) {
          this.caps.get(FOCUS_SET_FOCUSABLE_CAP)(target, false);
        }
      }
    }
    declareFocusable() {
      if (!this.focusableDeclared) {
        this.focusableDeclared = true;
        this.setFocusState(this.focusableOwned, !this.focusableConfig.disabled, "focus declared", {
          defaultOnly: true
        });
      }
      this.wireHostFocusEvents();
      this.syncHostFocusable();
      this.syncCenter();
    }
    declareEntry() {
      this.entryDeclared = true;
      this.syncHostEntry();
    }
    declareScope() {
      this.scopeDeclared = true;
      this.wireScopeKeyEvents();
      this.syncCenter();
    }
    declareRoving() {
      this.rovingDeclared = true;
      this.wireRovingKeyEvents();
      this.syncCenter();
    }
    readHostFocusTarget(event2) {
      return event2?.nativeEvent?.target ?? event2?.target ?? null;
    }
    invalidateHostFocusTarget() {
      this.currentHostFocusTarget = null;
      this.hostFocusTargetGeneration += 1;
    }
    resampleCurrentFocusVisible(reason) {
      if (!this.focusableDeclared || this.focusableConfig.disabled)
        return;
      if (!this.focusedOwned.get())
        return;
      const generation = this.hostFocusTargetGeneration;
      const target = this.currentHostFocusTarget;
      const native = readNativeFocusVisible(target);
      const next = native.supported ? native.value : this.keyboardModality;
      if (generation !== this.hostFocusTargetGeneration || target !== this.currentHostFocusTarget) {
        return;
      }
      this.setFocusState(this.focusVisibleOwned, next, reason);
    }
    wireHostFocusEvents() {
      if (this.hostEventsWired)
        return;
      this.hostEventsWired = true;
      this.eventPort.onGlobal("key.down", () => {
        this.keyboardModality = true;
        this.resampleCurrentFocusVisible("reason: focus.key.down => focusVisible resample");
      });
      this.eventPort.on("pointer.down", () => {
        this.keyboardModality = false;
        this.resampleCurrentFocusVisible("reason: focus.pointer.down => focusVisible resample");
      });
      this.eventPort.on("host:focus", (ev) => {
        if (!this.focusableDeclared || this.focusableConfig.disabled)
          return;
        this.currentHostFocusTarget = this.readHostFocusTarget(ev);
        this.hostFocusTargetGeneration += 1;
        this.setFocusState(this.focusedOwned, true, "reason: focus.host:focus => focused");
        this.resampleCurrentFocusVisible("reason: focus.host:focus => focusVisible");
        this.setFocusState(this.activeOwned, true, "reason: focus.host:focus => active");
        this.setFocusState(this.hasFocusedOwned, true, "reason: focus.host:focus => hasFocused");
        const entry = this.createCenterEntry();
        if (entry)
          FOCUS_CENTER.noteFocused(entry);
      });
      this.eventPort.on("host:blur", (ev) => {
        const target = this.readHostFocusTarget(ev);
        if (this.currentHostFocusTarget && target && target !== this.currentHostFocusTarget) {
          return;
        }
        this.invalidateHostFocusTarget();
        this.setFocusState(this.focusedOwned, false, "reason: focus.host:blur => focused");
        this.setFocusState(this.focusVisibleOwned, false, "reason: focus.host:blur => focusVisible");
        this.setFocusState(this.activeOwned, false, "reason: focus.host:blur => active");
      });
    }
    wireScopeKeyEvents() {
      if (this.scopeEventsWired)
        return;
      this.scopeEventsWired = true;
      this.eventPort.onGlobal("key.down", (ev) => {
        if (!this.scopeDeclared)
          return;
        if (!this.scopeConfig.trap)
          return;
        if (this.scopeConfig.navigation !== "tab" && this.scopeConfig.navigation !== "tab+arrow") {
          return;
        }
        if (ev.key !== "Tab")
          return;
        const entry = this.createCenterEntry();
        if (!entry || !FOCUS_CENTER.isTopActiveScope(entry))
          return;
        ev.control.requestDefaultActionPrevention({
          reason: "focus.scope.trap",
          source: this.prototypeName
        });
        FOCUS_CENTER.focusInScope(entry, ev.shiftKey ? "prev" : "next");
      });
    }
    wireRovingKeyEvents() {
      if (this.rovingEventsWired)
        return;
      this.rovingEventsWired = true;
      this.eventPort.onGlobal("key.down", (ev) => {
        if (!this.rovingDeclared)
          return;
        const op = this.resolveRovingKeyOperation(ev);
        if (!op)
          return;
        const entry = this.createCenterEntry();
        if (!entry)
          return;
        const handled = FOCUS_CENTER.focusInRoving(entry, op, { requireFocusedMember: true });
        if (!handled)
          return;
        ev.control.requestDefaultActionPrevention({
          reason: "focus.roving.keyboard",
          source: this.prototypeName
        });
      });
    }
    resolveRovingKeyOperation(detail) {
      if (this.rovingConfig.navigation !== "arrow" && this.rovingConfig.navigation !== "tab+arrow") {
        return null;
      }
      const key = detail?.key;
      if (key === "Home")
        return "first";
      if (key === "End")
        return "last";
      const orientation = this.rovingConfig.orientation;
      if ((orientation === "horizontal" || orientation === "both") && key === "ArrowRight") {
        return "next";
      }
      if ((orientation === "horizontal" || orientation === "both") && key === "ArrowLeft") {
        return "prev";
      }
      if ((orientation === "vertical" || orientation === "both") && key === "ArrowDown") {
        return "next";
      }
      if ((orientation === "vertical" || orientation === "both") && key === "ArrowUp") {
        return "prev";
      }
      return null;
    }
    getFocusable() {
      this.declareFocusable();
      return this.focusableHandle;
    }
    getEntry() {
      this.declareEntry();
      return this.entryHandle;
    }
    getScope() {
      this.declareScope();
      return this.scopeHandle;
    }
    getRoving() {
      this.declareRoving();
      return this.rovingHandle;
    }
    configureFocusable(patch) {
      this.ensureSetup("focus.configureFocusable");
      this.declareFocusable();
      if (typeof patch.autoFocus !== "undefined") {
        pushOverrideWarning(this.warnings, "focusable", "autoFocus", this.focusableConfig.autoFocus, patch.autoFocus);
      }
      if (typeof patch.disabled !== "undefined") {
        pushOverrideWarning(this.warnings, "focusable", "disabled", this.focusableConfig.disabled, patch.disabled);
      }
      if (typeof patch.navParticipation !== "undefined") {
        pushOverrideWarning(this.warnings, "focusable", "navParticipation", this.focusableConfig.navParticipation, patch.navParticipation);
      }
      if (typeof patch.scopeKey !== "undefined") {
        pushOverrideWarning(this.warnings, "focusable", "scopeKey", this.focusableConfig.scopeKey?.meta?.debugLabel ?? this.focusableConfig.scopeKey?.id, patch.scopeKey?.meta?.debugLabel ?? patch.scopeKey?.id);
      }
      if (typeof patch.groupKey !== "undefined") {
        pushOverrideWarning(this.warnings, "focusable", "groupKey", this.focusableConfig.groupKey?.meta?.debugLabel ?? this.focusableConfig.groupKey?.id, patch.groupKey?.meta?.debugLabel ?? patch.groupKey?.id);
      }
      this.focusableConfig = Object.freeze({
        ...this.focusableConfig,
        ...patch,
        meta: mergeMeta(this.focusableConfig.meta, patch.meta)
      });
      this.setDisabled(this.focusableConfig.disabled, "focus config updated");
      this.syncHostFocusable();
      this.syncCenter();
    }
    configureEntry(patch) {
      this.ensureSetup("focus.configureEntry");
      this.declareEntry();
      if (typeof patch.strategy !== "undefined") {
        pushOverrideWarning(this.warnings, "entry", "strategy", this.entryConfig.strategy, patch.strategy);
      }
      if (typeof patch.fallback !== "undefined") {
        pushOverrideWarning(this.warnings, "entry", "fallback", this.entryConfig.fallback, patch.fallback);
      }
      if (typeof patch.disabled !== "undefined") {
        pushOverrideWarning(this.warnings, "entry", "disabled", this.entryConfig.disabled, patch.disabled);
      }
      this.entryConfig = Object.freeze({
        ...this.entryConfig,
        ...patch,
        meta: mergeMeta(this.entryConfig.meta, patch.meta)
      });
      this.syncHostEntry();
    }
    configureScope(patch) {
      this.ensureSetup("focus.configureScope");
      this.declareScope();
      if (typeof patch.key !== "undefined") {
        pushOverrideWarning(this.warnings, "scope", "key", this.scopeConfig.key?.meta?.debugLabel ?? this.scopeConfig.key?.id, patch.key?.meta?.debugLabel ?? patch.key?.id);
      }
      if (typeof patch.trap !== "undefined") {
        pushOverrideWarning(this.warnings, "scope", "trap", this.scopeConfig.trap, patch.trap);
      }
      if (typeof patch.loop !== "undefined") {
        pushOverrideWarning(this.warnings, "scope", "loop", this.scopeConfig.loop, patch.loop);
      }
      if (typeof patch.navigation !== "undefined") {
        pushOverrideWarning(this.warnings, "scope", "navigation", this.scopeConfig.navigation, patch.navigation);
      }
      if (typeof patch.orientation !== "undefined") {
        pushOverrideWarning(this.warnings, "scope", "orientation", this.scopeConfig.orientation, patch.orientation);
      }
      if (typeof patch.entry !== "undefined") {
        pushOverrideWarning(this.warnings, "scope", "entry", this.scopeConfig.entry, patch.entry);
      }
      if (typeof patch.restore !== "undefined") {
        pushOverrideWarning(this.warnings, "scope", "restore", this.scopeConfig.restore, patch.restore);
      }
      if (typeof patch.emptyPolicy !== "undefined") {
        pushOverrideWarning(this.warnings, "scope", "emptyPolicy", this.scopeConfig.emptyPolicy, patch.emptyPolicy);
      }
      if (typeof patch.group !== "undefined") {
        pushOverrideWarning(this.warnings, "scope", "group", this.scopeConfig.group, patch.group);
        if (patch.group && typeof patch.group === "object") {
          this.configureRoving(patch.group);
        }
      }
      this.scopeConfig = Object.freeze({
        ...this.scopeConfig,
        ...patch,
        meta: mergeMeta(this.scopeConfig.meta, patch.meta)
      });
    }
    configureRoving(patch) {
      this.ensureSetup("focus.configureRoving");
      this.declareRoving();
      if (typeof patch.key !== "undefined") {
        pushOverrideWarning(this.warnings, "scope", "roving.key", this.rovingConfig.key?.meta?.debugLabel ?? this.rovingConfig.key?.id, patch.key?.meta?.debugLabel ?? patch.key?.id);
      }
      if (typeof patch.loop !== "undefined") {
        pushOverrideWarning(this.warnings, "scope", "roving.loop", this.rovingConfig.loop, patch.loop);
      }
      if (typeof patch.navigation !== "undefined") {
        pushOverrideWarning(this.warnings, "scope", "roving.navigation", this.rovingConfig.navigation, patch.navigation);
      }
      if (typeof patch.orientation !== "undefined") {
        pushOverrideWarning(this.warnings, "scope", "roving.orientation", this.rovingConfig.orientation, patch.orientation);
      }
      if (typeof patch.entry !== "undefined") {
        pushOverrideWarning(this.warnings, "scope", "roving.entry", this.rovingConfig.entry, patch.entry);
      }
      if (typeof patch.selectOnFocus !== "undefined") {
        pushOverrideWarning(this.warnings, "scope", "roving.selectOnFocus", this.rovingConfig.selectOnFocus, patch.selectOnFocus);
      }
      this.rovingConfig = Object.freeze({
        ...this.rovingConfig,
        ...patch,
        meta: mergeMeta(this.rovingConfig.meta, patch.meta)
      });
      this.syncCenter();
    }
    setRovingLoop(loop) {
      this.rovingConfig = Object.freeze({
        ...this.rovingConfig,
        loop
      });
      this.syncCenter();
    }
    setRovingOrientation(orientation) {
      this.rovingConfig = Object.freeze({
        ...this.rovingConfig,
        orientation
      });
      this.syncCenter();
    }
    queuePendingFocus(options, syncFacts) {
      this.pendingFocusRequest = { options, syncFacts };
    }
    clearPendingFocus() {
      this.pendingFocusRequest = undefined;
    }
    fulfillPendingFocus() {
      const pending = this.pendingFocusRequest;
      if (!pending || !this.getRootTarget() || !this.caps.has(FOCUS_REQUEST_FOCUS_CAP))
        return false;
      this.pendingFocusRequest = undefined;
      if (pending.syncFacts)
        this.requestFocus(pending.options);
      else
        this.requestNativeFocus(pending.options);
      return true;
    }
    requestFocusDirect(options) {
      if (!this.focusableDeclared || this.focusableConfig.disabled)
        return "rejected";
      const target = this.getRootTarget();
      if (!target || !this.caps.has(FOCUS_REQUEST_FOCUS_CAP)) {
        this.queuePendingFocus(options, true);
        return "pending";
      }
      this.clearPendingFocus();
      const applied = this.caps.get(FOCUS_REQUEST_FOCUS_CAP)(target, options);
      if (applied === false) {
        this.queuePendingFocus(options, true);
        return "pending";
      }
      this.setFocusState(this.focusedOwned, true, options?.reason ?? "programmatic");
      this.setFocusState(this.focusVisibleOwned, options?.reason === "keyboard", options?.reason);
      this.setFocusState(this.activeOwned, true, options?.reason ?? "programmatic");
      this.setFocusState(this.hasFocusedOwned, true, options?.reason ?? "programmatic");
      return "applied";
    }
    clearFocus(reason) {
      this.clearPendingFocus();
      this.setFocusState(this.focusedOwned, false, reason);
      this.setFocusState(this.focusVisibleOwned, false, reason);
      this.setFocusState(this.activeOwned, false, reason);
    }
    requestFocus(options) {
      if (!this.focusableDeclared || this.focusableConfig.disabled)
        return;
      const entry = this.createCenterEntry();
      if (!entry) {
        this.requestFocusDirect(options);
        return;
      }
      FOCUS_CENTER.requestFocus(entry, options, { syncFacts: true });
    }
    requestEntryFocus(options) {
      if (!this.entryDeclared || this.entryConfig.disabled)
        return;
      const target = this.getRootTarget();
      if (!target || !this.caps.has(FOCUS_REQUEST_FOCUS_CAP))
        return;
      const resolved = this.caps.has(FOCUS_RESOLVE_ENTRY_TARGET_CAP) ? this.caps.get(FOCUS_RESOLVE_ENTRY_TARGET_CAP)(target, this.entryConfig) : this.entryConfig.fallback === "self" ? target : null;
      if (!resolved)
        return;
      this.caps.get(FOCUS_REQUEST_FOCUS_CAP)(resolved, options);
    }
    requestNativeFocusDirect(options) {
      if (!this.focusableDeclared || this.focusableConfig.disabled)
        return "rejected";
      if (options?.reason === "keyboard")
        this.keyboardModality = true;
      else if (options?.reason === "pointer")
        this.keyboardModality = false;
      const target = this.getRootTarget();
      if (!target || !this.caps.has(FOCUS_REQUEST_FOCUS_CAP)) {
        this.queuePendingFocus(options, false);
        return "pending";
      }
      this.clearPendingFocus();
      const applied = this.caps.get(FOCUS_REQUEST_FOCUS_CAP)(target, options);
      if (applied === false) {
        this.queuePendingFocus(options, false);
        return "pending";
      }
      return "applied";
    }
    requestNativeFocus(options) {
      if (!this.focusableDeclared || this.focusableConfig.disabled)
        return;
      const entry = this.createCenterEntry();
      if (!entry) {
        this.requestNativeFocusDirect(options);
        return;
      }
      FOCUS_CENTER.requestFocus(entry, options, { syncFacts: false });
    }
    blur() {
      this.clearPendingFocus();
      const target = this.getRootTarget();
      if (target && this.caps.has(FOCUS_BLUR_CAP)) {
        this.caps.get(FOCUS_BLUR_CAP)(target);
      }
      this.setFocusState(this.focusedOwned, false, "blur");
      this.setFocusState(this.focusVisibleOwned, false, "blur");
      this.setFocusState(this.activeOwned, false, "blur");
    }
    focusFirst(options) {
      const entry = this.createCenterEntry();
      if (entry && this.rovingDeclared) {
        FOCUS_CENTER.focusInRoving(entry, "first", { entryRequest: options });
        return;
      }
      if (this.focusableConfig.disabled)
        return;
      if (this.scopeConfig.emptyPolicy === "container") {
        this.setFocusState(this.activeOwned, true, "focusFirst:container");
        this.setFocusState(this.hasFocusedOwned, false, "focusFirst:container");
        this.setFocusState(this.focusedOwned, false, "focusFirst:container");
        this.setFocusState(this.focusVisibleOwned, false, "focusFirst:container");
        return;
      }
      this.requestFocus({ reason: "programmatic" });
    }
    focusLast(options) {
      const entry = this.createCenterEntry();
      if (entry && this.rovingDeclared) {
        FOCUS_CENTER.focusInRoving(entry, "last", { entryRequest: options });
        return;
      }
      this.requestFocus({ reason: "programmatic" });
    }
    focusNext() {
      const entry = this.createCenterEntry();
      if (entry && this.rovingDeclared) {
        FOCUS_CENTER.focusInRoving(entry, "next");
        return;
      }
      this.requestFocus({ reason: "programmatic" });
    }
    focusPrev() {
      const entry = this.createCenterEntry();
      if (entry && this.rovingDeclared) {
        FOCUS_CENTER.focusInRoving(entry, "prev");
        return;
      }
      this.requestFocus({ reason: "programmatic" });
    }
    focusSelected(options) {
      const entry = this.createCenterEntry();
      if (entry && this.rovingDeclared) {
        FOCUS_CENTER.focusInRoving(entry, "selected", { entryRequest: options });
        return;
      }
      this.requestFocus({ reason: "programmatic" });
    }
    restoreFocus() {
      this.requestFocus({ reason: "programmatic" });
    }
    activateScope(options) {
      this.declareScope();
      const entry = this.createCenterEntry();
      if (!entry)
        return;
      FOCUS_CENTER.activateScope(entry, options);
    }
    deactivateScope(options) {
      const entry = this.createCenterEntry();
      if (!entry) {
        this.setScopeActive(false);
        return;
      }
      FOCUS_CENTER.deactivateScope(entry, options);
    }
    isScopeActive() {
      const entry = this.createCenterEntry();
      return entry ? FOCUS_CENTER.isScopeActive(entry) : this.activeState.get();
    }
    setScopeActive(active) {
      this.setFocusState(this.activeOwned, active, active ? "scope.activate" : "scope.deactivate");
      if (active) {
        this.setFocusState(this.hasFocusedOwned, true, "scope.activate");
      }
    }
    setDisabled(disabled, reason = "focus.setDisabled") {
      this.focusableConfig = Object.freeze({
        ...this.focusableConfig,
        disabled
      });
      this.setFocusState(this.focusableOwned, this.focusableDeclared && !disabled, reason, {
        defaultOnly: this.sys?.execPhase?.() === "setup"
      });
      if (disabled) {
        this.blur();
      }
      this.syncHostFocusable();
      this.syncCenter();
    }
    setNavParticipation(navParticipation) {
      this.focusableConfig = Object.freeze({
        ...this.focusableConfig,
        navParticipation
      });
      this.syncHostFocusable();
      this.syncCenter();
    }
    setRovingStatus(status) {
      if (typeof status.selected !== "undefined")
        this.rovingSelected = status.selected;
      if (typeof status.active !== "undefined")
        this.rovingActive = status.active;
    }
    setEntryDisabled(disabled) {
      this.entryConfig = Object.freeze({
        ...this.entryConfig,
        disabled
      });
      this.syncHostEntry();
    }
    afterRenderCommit() {
      this.syncCenter();
      this.syncHostFocusable();
      this.syncHostEntry();
      const hadPendingFocus = !!this.pendingFocusRequest;
      if (this.mountPhase !== "mounting")
        this.fulfillPendingFocus();
      if (hadPendingFocus) {
        this.didAutoFocus = true;
        return;
      }
      if (this.didAutoFocus)
        return;
      this.didAutoFocus = true;
      if (this.focusableDeclared && this.focusableConfig.autoFocus && !this.focusableConfig.disabled) {
        this.requestFocus({ reason: "programmatic" });
      }
    }
    getEffectiveScopeKey() {
      return this.focusableConfig.scopeKey ?? this.scopeConfig.key;
    }
    getEffectiveRovingKey() {
      return this.rovingConfig.key;
    }
    getFocusableConfig() {
      return this.focusableConfig;
    }
    getEntryConfig() {
      return this.entryConfig;
    }
    getScopeConfig() {
      return this.scopeConfig;
    }
    getRovingConfig() {
      return this.rovingConfig;
    }
    getFacts() {
      return Object.freeze({
        focused: this.focusedState.get(),
        focusVisible: this.focusVisibleState.get(),
        focusable: this.focusableState.get(),
        active: this.activeState.get(),
        hasFocused: this.hasFocusedState.get(),
        rovingSelected: this.rovingSelected,
        rovingActive: this.rovingActive
      });
    }
    getWarnings() {
      return Object.freeze(this.warnings.slice());
    }
    onInstancePhase(phase) {
      super.onInstancePhase(phase);
      if (phase === "disposing") {
        this.clearPendingFocus();
        this.invalidateHostFocusTarget();
        const self = this.getSelfToken();
        if (self)
          FOCUS_CENTER.remove(self);
      }
      if (phase === "disposed") {
        this.offTargetReady?.();
        this.offTargetReady = undefined;
      }
    }
    onMountPhase(phase, epoch) {
      super.onMountPhase(phase, epoch);
      if (phase === "mounted") {
        this.syncCenter();
        this.syncHostFocusable();
        this.syncHostEntry();
        this.fulfillPendingFocus();
        return;
      }
      if (phase !== "detached")
        return;
      this.invalidateHostFocusTarget();
      const self = this.getSelfToken();
      if (self)
        FOCUS_CENTER.detach(self);
    }
  }
  function createFocusModule(ctx) {
    const { init, caps: caps13, deps } = ctx;
    return createModule({
      name: "focus",
      scope: "instance",
      init,
      caps: caps13,
      deps,
      build: ({ deps: deps2 }) => {
        const eventPort = deps2.requirePort("event");
        const statePort = deps2.requirePort("state");
        const stateFacade = deps2.requireFacade("state");
        const impl3 = new FocusModuleImpl(caps13, init.prototypeName, eventPort, statePort, stateFacade);
        const port = {
          configureFocusable: (patch) => impl3.configureFocusable(patch),
          configureEntry: (patch) => impl3.configureEntry(patch),
          configureRoving: (patch) => impl3.configureRoving(patch),
          configureGroup: (patch) => impl3.configureRoving(patch),
          setRovingLoop: (loop) => impl3.setRovingLoop(loop),
          setRovingOrientation: (orientation) => impl3.setRovingOrientation(orientation),
          configureScope: (patch) => impl3.configureScope(patch),
          setDisabled: (disabled) => impl3.setDisabled(disabled),
          setNavParticipation: (navParticipation) => impl3.setNavParticipation(navParticipation),
          setRovingStatus: (status) => impl3.setRovingStatus(status),
          setEntryDisabled: (disabled) => impl3.setEntryDisabled(disabled),
          requestFocus: (options) => impl3.requestFocus(options),
          requestEntryFocus: (options) => impl3.requestEntryFocus(options),
          blur: () => impl3.blur(),
          focusFirst: (options) => impl3.focusFirst(options),
          focusLast: (options) => impl3.focusLast(options),
          focusNext: () => impl3.focusNext(),
          focusPrev: () => impl3.focusPrev(),
          focusSelected: (options) => impl3.focusSelected(options),
          restoreFocus: () => impl3.restoreFocus(),
          activateScope: (options) => impl3.activateScope(options),
          deactivateScope: (options) => impl3.deactivateScope(options),
          isScopeActive: () => impl3.isScopeActive(),
          getEffectiveRovingKey: () => impl3.getEffectiveRovingKey(),
          getEffectiveGroupKey: () => impl3.getEffectiveRovingKey(),
          getEffectiveScopeKey: () => impl3.getEffectiveScopeKey(),
          getFocusableConfig: () => impl3.getFocusableConfig(),
          getEntryConfig: () => impl3.getEntryConfig(),
          getRovingConfig: () => impl3.getRovingConfig(),
          getGroupConfig: () => impl3.getRovingConfig(),
          getScopeConfig: () => impl3.getScopeConfig(),
          getFacts: () => impl3.getFacts(),
          getWarnings: () => impl3.getWarnings()
        };
        return {
          facade: {
            getFocusable: () => impl3.getFocusable(),
            getEntry: () => impl3.getEntry(),
            getRoving: () => impl3.getRoving(),
            getScope: () => impl3.getScope()
          },
          hooks: {
            onInstancePhase: (p) => impl3.onInstancePhase(p),
            onMountPhase: (p, epoch) => impl3.onMountPhase(p, epoch),
            onProtoPhase: (p) => impl3.onProtoPhase(p),
            afterRenderCommit: () => impl3.afterRenderCommit()
          },
          port
        };
      }
    });
  }
  var FocusModuleDef = defineModule({
    name: "focus",
    resourceOwnership: "mixed",
    deps: ["event", "state"],
    create: createFocusModule
  });
  // ../packages/modules/boundary/src/caps.ts
  var BOUNDARY_HOST_BRIDGE_CAP = cap("@proto.ui/boundary/hostBridge");
  // ../packages/modules/boundary/src/impl.ts
  var DEFAULT_CONFIG = Object.freeze({});
  function mergeMeta2(prev, next) {
    if (!next)
      return prev;
    return Object.freeze({
      ...prev ?? {},
      ...next
    });
  }
  function pushOverrideWarning2(warnings, field, prev, next) {
    if (typeof prev === "undefined" || Object.is(prev, next))
      return;
    warnings.push(`[Boundary] ${field} overridden: ${String(prev)} -> ${String(next)}`);
  }
  var STACK_CENTER = (() => {
    const order = [];
    return {
      activate(id) {
        const existingIndex = order.indexOf(id);
        if (existingIndex >= 0) {
          order.splice(existingIndex, 1);
        }
        order.push(id);
      },
      deactivate(id) {
        const existingIndex = order.indexOf(id);
        if (existingIndex >= 0) {
          order.splice(existingIndex, 1);
        }
      },
      top() {
        return order.length > 0 ? order[order.length - 1] ?? null : null;
      }
    };
  })();

  class BoundaryModuleImpl extends ModuleBase {
    eventPort;
    static nextBoundaryInstanceId = 1;
    config = DEFAULT_CONFIG;
    prototypeName;
    boundaryInstanceId = BoundaryModuleImpl.nextBoundaryInstanceId++;
    warnings = [];
    outsideSubscribers = new Set;
    hostBridge = null;
    hostElement = null;
    nextRegionId = 1;
    regions = [];
    stackActive = false;
    suspended = false;
    observingPointerDown = false;
    constructor(caps14, prototypeName, eventPort) {
      super(caps14);
      this.eventPort = eventPort;
      this.prototypeName = prototypeName;
      this.refreshHostCaps();
    }
    onCapsEpoch(_epoch) {
      this.refreshHostCaps();
    }
    onProtoPhase(phase) {
      super.onProtoPhase(phase);
      if (phase !== "unmounted")
        return;
      this.setStackActive(false);
      this.regions = [];
      this.outsideSubscribers.clear();
    }
    onMountPhase(phase, epoch) {
      super.onMountPhase(phase, epoch);
      if (phase === "unmounting" || phase === "detached") {
        this.suspended = true;
        STACK_CENTER.deactivate(this.boundaryInstanceId);
        return;
      }
      if (phase === "mounted") {
        this.suspended = false;
        if (this.stackActive)
          STACK_CENTER.activate(this.boundaryInstanceId);
      }
    }
    refreshHostCaps() {
      this.hostElement = this.caps.has(HOST_ELEMENT_CAP) ? this.caps.get(HOST_ELEMENT_CAP) : null;
      this.hostBridge = this.caps.has(BOUNDARY_HOST_BRIDGE_CAP) ? this.caps.get(BOUNDARY_HOST_BRIDGE_CAP) : null;
    }
    ensureSetup(op) {
      this.sys?.ensureSetup(op);
      if (!this.sys && this.protoPhase !== "setup") {
        throw illegalPhase(op, this.protoPhase, {
          prototypeName: this.prototypeName
        });
      }
    }
    configure(patch) {
      this.ensureSetup("boundary.configure");
      if (typeof patch.debugLabel !== "undefined") {
        pushOverrideWarning2(this.warnings, "debugLabel", this.config.debugLabel, patch.debugLabel);
        this.config = Object.freeze({
          ...this.config,
          debugLabel: patch.debugLabel
        });
      }
      if (typeof patch.meta !== "undefined") {
        this.config = Object.freeze({
          ...this.config,
          meta: mergeMeta2(this.config.meta, patch.meta)
        });
      }
    }
    setStackActive(active) {
      if (Object.is(this.stackActive, active)) {
        if (active && !this.suspended) {
          STACK_CENTER.activate(this.boundaryInstanceId);
        }
        return;
      }
      this.stackActive = active;
      if (active && !this.suspended) {
        STACK_CENTER.activate(this.boundaryInstanceId);
        return;
      }
      STACK_CENTER.deactivate(this.boundaryInstanceId);
    }
    registerRegion(target, options = {}) {
      const id = this.nextRegionId++;
      this.regions = this.regions.concat([
        Object.freeze({
          id,
          target,
          role: options.role,
          meta: options.meta
        })
      ]);
      return () => {
        this.regions = this.regions.filter((region) => region.id !== id);
      };
    }
    unregisterRegion(target) {
      this.regions = this.regions.filter((region) => region.target !== target);
    }
    getRegions() {
      const explicit = this.regions.map(({ target, role, meta }) => Object.freeze({
        target,
        ...typeof role === "undefined" ? {} : { role },
        ...typeof meta === "undefined" ? {} : { meta }
      }));
      if (this.hostElement && !explicit.some((region) => Object.is(region.target, this.hostElement))) {
        return Object.freeze([{ target: this.hostElement, role: "content" }, ...explicit]);
      }
      return explicit;
    }
    classify(sample) {
      const target = sample?.target;
      if (typeof target !== "undefined" && this.regions.some((region) => Object.is(region.target, target))) {
        return "inside";
      }
      if (this.hostBridge) {
        return this.hostBridge.classify({
          regions: this.getRegions(),
          sample
        });
      }
      return "unknown";
    }
    notify(sample) {
      if (this.suspended)
        return "unknown";
      const classification = this.classify(sample);
      if (classification !== "outside")
        return classification;
      if (this.stackActive) {
        const topBoundaryId = STACK_CENTER.top();
        if (topBoundaryId !== null && topBoundaryId !== this.boundaryInstanceId) {
          return "unknown";
        }
      }
      const event2 = Object.freeze({
        classification,
        sample
      });
      for (const subscriber of this.outsideSubscribers) {
        subscriber(event2);
      }
      return classification;
    }
    observe(observation) {
      this.ensureSetup("boundary.observe");
      if (observation !== "pointer.press")
        return;
      if (this.observingPointerDown)
        return;
      this.observingPointerDown = true;
      this.eventPort.onGlobal("host:pointerdown", (nativeEvent) => {
        const target = nativeEvent && typeof nativeEvent === "object" && "target" in nativeEvent ? nativeEvent.target : undefined;
        this.notify({ type: "pointerdown", target, nativeEvent });
      });
    }
    subscribeOutside(cb) {
      this.outsideSubscribers.add(cb);
      return () => {
        this.outsideSubscribers.delete(cb);
      };
    }
    getConfig() {
      return this.config;
    }
    getWarnings() {
      return Object.freeze(this.warnings.slice());
    }
    handle = {
      configure: (patch) => this.configure(patch),
      observe: (observation) => this.observe(observation),
      setStackActive: (active) => this.setStackActive(active),
      registerRegion: (target, options) => this.registerRegion(target, options),
      unregisterRegion: (target) => this.unregisterRegion(target),
      classify: (sample) => this.classify(sample),
      notify: (sample) => this.notify(sample),
      subscribeOutside: (cb) => this.subscribeOutside(cb)
    };
  }
  // ../packages/modules/boundary/src/create.ts
  function createBoundaryModule(ctx) {
    const { init, caps: caps14, deps } = ctx;
    return createModule({
      name: "boundary",
      scope: "instance",
      init,
      caps: caps14,
      deps,
      build: ({ init: init2, caps: caps15 }) => {
        const impl3 = new BoundaryModuleImpl(caps15, init2.prototypeName, deps.requirePort("event"));
        return {
          facade: {
            getBoundary: () => impl3.handle
          },
          hooks: {
            onMountPhase: (phase, epoch) => impl3.onMountPhase(phase, epoch),
            onProtoPhase: (phase) => impl3.onProtoPhase(phase)
          },
          port: {
            configure: (patch) => impl3.configure(patch),
            setStackActive: (active) => impl3.setStackActive(active),
            registerRegion: (target, options) => impl3.registerRegion(target, options),
            unregisterRegion: (target) => impl3.unregisterRegion(target),
            classify: (sample) => impl3.classify(sample),
            notify: (sample) => impl3.notify(sample),
            observe: (observation) => impl3.observe(observation),
            subscribeOutside: (cb) => impl3.subscribeOutside(cb),
            getConfig: () => impl3.getConfig(),
            getWarnings: () => impl3.getWarnings(),
            getRegions: () => impl3.getRegions()
          }
        };
      }
    });
  }
  var BoundaryModuleDef = defineModule({
    name: "boundary",
    resourceOwnership: "mixed",
    deps: ["event"],
    create: createBoundaryModule
  });
  // ../packages/modules/boundary/src/web/host-bridge.ts
  var PROTO_PARENT_INSTANCE_MARK = Symbol.for("@proto.ui/adapter-base/__proto_parent_instance");
  // ../packages/modules/hit-participation/src/caps.ts
  var HIT_PARTICIPATION_HOST_BRIDGE_CAP = cap("@proto.ui/hitParticipation/hostBridge");
  // ../packages/modules/hit-participation/src/impl.ts
  var DEFAULT_CONFIG2 = Object.freeze({
    mode: "participating"
  });
  function mergeMeta3(prev, next) {
    if (!next)
      return prev;
    return Object.freeze({
      ...prev ?? {},
      ...next
    });
  }
  function pushOverrideWarning3(warnings, field, prev, next) {
    if (typeof prev === "undefined" || Object.is(prev, next))
      return;
    warnings.push(`[HitParticipation] ${field} overridden: ${String(prev)} -> ${String(next)}`);
  }

  class HitParticipationModuleImpl extends ModuleBase {
    config = DEFAULT_CONFIG2;
    prototypeName;
    warnings = [];
    hostBridge = null;
    hostElement = null;
    nextRegionId = 1;
    regions = [];
    suspended = false;
    constructor(caps14, prototypeName) {
      super(caps14);
      this.prototypeName = prototypeName;
      this.refreshHostCaps();
    }
    onCapsEpoch(_epoch) {
      this.refreshHostCaps();
      this.syncHostBridge();
    }
    onProtoPhase(phase) {
      super.onProtoPhase(phase);
      if (phase !== "unmounted")
        return;
      this.clearHostBridge();
      this.regions = [];
    }
    onMountPhase(phase, epoch) {
      super.onMountPhase(phase, epoch);
      if (phase === "unmounting" || phase === "detached") {
        this.suspended = true;
        this.clearHostBridge();
        return;
      }
      if (phase === "mounted") {
        this.suspended = false;
        this.refreshHostCaps();
        this.syncHostBridge();
      }
    }
    refreshHostCaps() {
      this.hostElement = this.caps.has(HOST_ELEMENT_CAP) ? this.caps.get(HOST_ELEMENT_CAP) : null;
      this.hostBridge = this.caps.has(HIT_PARTICIPATION_HOST_BRIDGE_CAP) ? this.caps.get(HIT_PARTICIPATION_HOST_BRIDGE_CAP) : null;
    }
    ensureSetup(op) {
      this.sys?.ensureSetup(op);
      if (!this.sys && this.protoPhase !== "setup") {
        throw illegalPhase(op, this.protoPhase, {
          prototypeName: this.prototypeName
        });
      }
    }
    patchMode(mode) {
      if (typeof mode === "undefined")
        return;
      pushOverrideWarning3(this.warnings, "mode", this.config.mode, mode);
      this.config = Object.freeze({
        ...this.config,
        mode
      });
    }
    configure(patch) {
      this.ensureSetup("hitParticipation.configure");
      this.patchMode(patch.mode);
      if (typeof patch.debugLabel !== "undefined") {
        pushOverrideWarning3(this.warnings, "debugLabel", this.config.debugLabel, patch.debugLabel);
        this.config = Object.freeze({
          ...this.config,
          debugLabel: patch.debugLabel
        });
      }
      if (typeof patch.meta !== "undefined") {
        this.config = Object.freeze({
          ...this.config,
          meta: mergeMeta3(this.config.meta, patch.meta)
        });
      }
      this.syncHostBridge();
    }
    registerRegion(target, options = {}) {
      const id = this.nextRegionId++;
      this.regions = this.regions.concat([
        Object.freeze({
          id,
          target,
          role: options.role,
          mode: options.mode ?? this.config.mode,
          meta: options.meta
        })
      ]);
      this.syncHostBridge();
      return () => {
        this.regions = this.regions.filter((region) => region.id !== id);
        this.syncHostBridge();
      };
    }
    unregisterRegion(target) {
      this.regions = this.regions.filter((region) => region.target !== target);
      this.syncHostBridge();
    }
    getConfig() {
      return this.config;
    }
    getWarnings() {
      return Object.freeze(this.warnings.slice());
    }
    getRegions() {
      return this.buildRegions();
    }
    buildRegions() {
      return this.regions.map(({ target, role, mode, meta }) => Object.freeze({
        target,
        mode,
        ...typeof role === "undefined" ? {} : { role },
        ...typeof meta === "undefined" ? {} : { meta }
      }));
    }
    getEffectiveRegions(includeImplicitHost) {
      const explicit = this.buildRegions();
      if (includeImplicitHost && this.hostElement && !explicit.some((region) => Object.is(region.target, this.hostElement))) {
        return Object.freeze([
          { target: this.hostElement, role: "content", mode: this.config.mode },
          ...explicit
        ]);
      }
      return explicit;
    }
    syncHostBridge() {
      if (!this.hostBridge || this.suspended || this.mountPhase === "detached" || this.mountPhase === "unmounting")
        return;
      this.hostBridge.sync({
        config: this.config,
        regions: this.getEffectiveRegions(true)
      });
    }
    clearHostBridge() {
      if (!this.hostBridge)
        return;
      this.hostBridge.sync({
        config: this.config,
        regions: []
      });
    }
    handle = {
      configure: (patch) => this.configure(patch),
      registerRegion: (target, options) => this.registerRegion(target, options),
      unregisterRegion: (target) => this.unregisterRegion(target)
    };
  }

  // ../packages/modules/hit-participation/src/create.ts
  function createHitParticipationModule(ctx) {
    const { init, caps: caps14, deps } = ctx;
    return createModule({
      name: "hit-participation",
      scope: "instance",
      init,
      caps: caps14,
      deps,
      build: ({ init: init2, caps: caps15 }) => {
        const impl3 = new HitParticipationModuleImpl(caps15, init2.prototypeName);
        return {
          facade: {
            getHitParticipation: () => impl3.handle
          },
          hooks: {
            onMountPhase: (phase, epoch) => impl3.onMountPhase(phase, epoch),
            onProtoPhase: (phase) => impl3.onProtoPhase(phase)
          },
          port: {
            configure: (patch) => impl3.configure(patch),
            registerRegion: (target, options) => impl3.registerRegion(target, options),
            unregisterRegion: (target) => impl3.unregisterRegion(target),
            getConfig: () => impl3.getConfig(),
            getWarnings: () => impl3.getWarnings(),
            getRegions: () => impl3.getRegions()
          }
        };
      }
    });
  }
  var HitParticipationModuleDef = defineModule({
    name: "hit-participation",
    resourceOwnership: "mixed",
    create: createHitParticipationModule
  });
  // ../packages/modules/hit-participation/src/web/host-bridge.ts
  var HIT_PARTICIPATION_MODE_MARK = Symbol.for("@proto.ui/module-hit-participation/__mode");
  var HIT_PARTICIPATION_PREV_POINTER_EVENTS_MARK = Symbol.for("@proto.ui/module-hit-participation/__prev_pointer_events");
  // ../packages/modules/overlay/src/caps.ts
  var OVERLAY_GLOBAL_MOUNT_CAP = cap("@proto.ui/overlay/globalMount");
  var OVERLAY_MODAL_CAP = cap("@proto.ui/overlay/modal");
  var OVERLAY_LAYER_SCHEDULER_CAP = cap("@proto.ui/overlay/layerScheduler");

  // ../packages/modules/overlay/src/impl.ts
  var DEFAULT_CONFIG3 = Object.freeze({
    defaultOpen: false,
    closeOnEscape: false,
    closeOnOutsidePress: false,
    closeOnFocusOutside: false,
    closeOnAnchorPress: false,
    closeOnTriggerPress: false,
    placement: "bottom",
    align: "start",
    sideOffset: 4,
    alignOffset: 0,
    anchored: false,
    strategy: "absolute",
    avoidCollisions: true,
    collisionBoundary: "clippingAncestors",
    collisionPadding: 0,
    excludeAnchorTranslation: false,
    entry: "content",
    restore: "trigger",
    portal: false,
    modal: false,
    layerRole: "overlay",
    layerOffset: 0
  });
  function createObservedHandle(initialValue) {
    let value = initialValue;
    const watchers = new Set;
    const handle = {
      get: () => value,
      watch: (cb) => {
        watchers.add(cb);
        return () => {
          watchers.delete(cb);
        };
      }
    };
    return {
      handle: Object.freeze(handle),
      set(next, reason) {
        if (Object.is(next, value))
          return;
        const prev = value;
        value = next;
        const event2 = { type: "next", next, prev, reason };
        for (const watcher of watchers) {
          watcher(undefined, event2);
        }
      }
    };
  }
  function mergeMeta4(prev, next) {
    if (!next)
      return prev;
    return Object.freeze({
      ...prev ?? {},
      ...next
    });
  }
  function pushOverrideWarning4(warnings, field, prev, next) {
    if (typeof prev === "undefined" || Object.is(prev, next))
      return;
    warnings.push(`[Overlay] ${field} overridden: ${String(prev)} -> ${String(next)}`);
  }

  class OverlayModuleImpl extends ModuleBase {
    boundaryPort;
    eventPort;
    anatomyPort;
    anchoredPosition;
    config = DEFAULT_CONFIG3;
    presenceBound = false;
    prototypeName;
    warnings = [];
    boundary;
    lastReason = undefined;
    viewReconciliationVersion = 0;
    registration = Object.freeze({
      trigger: null,
      anchor: null,
      content: null
    });
    openState = createObservedHandle(false);
    viewActive = false;
    globalMount = null;
    modalLock = null;
    layerScheduler = null;
    mountedHost = null;
    anchorPart = null;
    layerDetach = null;
    layerHost = null;
    modalLocked = false;
    boundaryDisposers = {
      trigger: null,
      anchor: null,
      content: null
    };
    offBoundaryOutside;
    escapeSamplingInstalled = false;
    constructor(caps14, prototypeName, boundary2, boundaryPort, eventPort, anatomyPort, anchoredPosition) {
      super(caps14);
      this.boundaryPort = boundaryPort;
      this.eventPort = eventPort;
      this.anatomyPort = anatomyPort;
      this.anchoredPosition = anchoredPosition;
      this.prototypeName = prototypeName;
      this.boundary = boundary2;
      this.refreshHostCaps();
      this.offBoundaryOutside = this.boundary.subscribeOutside(() => {
        if (!this.isOpen())
          return;
        if (!this.config.closeOnOutsidePress)
          return;
        this.close("outside.press");
      });
    }
    installDismissSampling() {
      if (this.config.closeOnOutsidePress) {
        this.boundaryPort.observe("pointer.press");
      }
      if (this.config.closeOnEscape && !this.escapeSamplingInstalled) {
        this.escapeSamplingInstalled = true;
        this.eventPort.onGlobal("key.down", (event2) => {
          if (!this.isOpen() || !this.config.closeOnEscape)
            return;
          if (event2.key !== "Escape")
            return;
          this.close("escape");
        });
      }
    }
    onCapsEpoch(_epoch) {
      this.refreshHostCaps();
    }
    onProtoPhase(phase) {
      super.onProtoPhase(phase);
      if (phase !== "unmounted")
        return;
      this.teardownMountedViewSideEffects();
      this.clearBoundaryRegistrations();
      this.offBoundaryOutside?.();
    }
    onMountPhase(phase, epoch) {
      super.onMountPhase(phase, epoch);
      if (phase === "unmounting" || phase === "detached") {
        this.teardownMountedViewSideEffects();
        this.boundary.setStackActive(false);
        return;
      }
      if (phase === "mounted") {
        if (this.isOpen())
          this.boundary.setStackActive(true);
        if (this.viewActive)
          this.syncViewSideEffects();
      }
    }
    refreshHostCaps() {
      this.globalMount = this.caps.has(OVERLAY_GLOBAL_MOUNT_CAP) ? this.caps.get(OVERLAY_GLOBAL_MOUNT_CAP) : null;
      this.modalLock = this.caps.has(OVERLAY_MODAL_CAP) ? this.caps.get(OVERLAY_MODAL_CAP) : null;
      this.layerScheduler = this.caps.has(OVERLAY_LAYER_SCHEDULER_CAP) ? this.caps.get(OVERLAY_LAYER_SCHEDULER_CAP) : null;
    }
    ensureSetup(op) {
      this.sys?.ensureSetup(op);
      if (!this.sys && this.protoPhase !== "setup") {
        throw illegalPhase(op, this.protoPhase, {
          prototypeName: this.prototypeName
        });
      }
    }
    resolveHostElement() {
      let hostEl = this.registration.content instanceof HTMLElement ? this.registration.content : null;
      if (hostEl)
        return hostEl;
      if (!this.caps.has(HOST_ELEMENT_CAP))
        return null;
      const capHost = this.caps.get(HOST_ELEMENT_CAP);
      return capHost instanceof HTMLElement ? capHost : null;
    }
    mountGlobalIfNeeded(hostEl) {
      if (!this.config.portal || !this.globalMount)
        return;
      if (this.mountedHost === hostEl)
        return;
      if (this.mountedHost && this.mountedHost !== hostEl) {
        this.globalMount.unmount(this.mountedHost);
        this.mountedHost = null;
      }
      this.globalMount.mount(hostEl);
      this.mountedHost = hostEl;
    }
    unmountGlobalIfNeeded() {
      if (!this.mountedHost || !this.globalMount)
        return;
      this.globalMount.unmount(this.mountedHost);
      this.mountedHost = null;
    }
    applyLayerIfNeeded(hostEl) {
      if (!this.layerScheduler)
        return;
      if (this.layerDetach && this.layerHost === hostEl)
        return;
      this.clearLayer();
      this.layerDetach = this.layerScheduler.attach(hostEl, {
        role: this.config.layerRole,
        offset: this.config.layerOffset,
        modal: this.config.modal,
        portal: this.config.portal,
        meta: this.config.meta
      });
      this.layerHost = hostEl;
    }
    clearLayer() {
      this.layerHost = null;
      if (!this.layerDetach)
        return;
      try {
        this.layerDetach();
      } finally {
        this.layerDetach = null;
      }
    }
    lockModalIfNeeded() {
      if (!this.config.modal || !this.modalLock || this.modalLocked)
        return;
      this.modalLock.lock();
      this.modalLocked = true;
    }
    unlockModalIfNeeded() {
      if (!this.modalLock || !this.modalLocked)
        return;
      this.modalLock.unlock();
      this.modalLocked = false;
    }
    syncViewSideEffects() {
      if (this.mountPhase !== "mounted")
        return;
      this.syncAnchorPartRegistration();
      const hostEl = this.resolveHostElement();
      if (hostEl) {
        this.mountGlobalIfNeeded(hostEl);
        this.applyLayerIfNeeded(hostEl);
      }
      this.syncAnchoredPosition();
      this.lockModalIfNeeded();
    }
    deactivateViewSideEffects() {
      this.anchoredPosition.disconnect();
      this.unlockModalIfNeeded();
    }
    teardownMountedViewSideEffects() {
      this.clearLayer();
      this.anchoredPosition.disconnect();
      this.unmountGlobalIfNeeded();
      this.unlockModalIfNeeded();
    }
    setOpen(next, reason) {
      this.lastReason = reason;
      const wasOpen = this.openState.handle.get();
      if (Object.is(wasOpen, next)) {
        if (next) {
          this.boundary.setStackActive(true);
          if (this.viewActive)
            this.syncViewSideEffects();
        } else {
          this.boundary.setStackActive(false);
        }
        return;
      }
      this.openState.set(next, reason);
      if (next) {
        this.boundary.setStackActive(true);
        return;
      }
      this.boundary.setStackActive(false);
    }
    markPresenceBound() {
      this.presenceBound = true;
    }
    hasPresenceBinding() {
      return this.presenceBound;
    }
    setViewActive(active) {
      if (Object.is(this.viewActive, active)) {
        return;
      }
      this.viewActive = active;
      this.viewReconciliationVersion += 1;
      if (active) {
        if (this.mountPhase === "mounted")
          this.lockModalIfNeeded();
        return;
      }
      this.deactivateViewSideEffects();
    }
    reconcileViewResourcesAfterCallback() {
      if (!this.viewActive)
        return;
      const version = this.viewReconciliationVersion;
      const reconcile = () => {
        if (!this.viewActive || version !== this.viewReconciliationVersion)
          return;
        this.syncViewSideEffects();
      };
      if (this.sys.deferAfterCallback) {
        this.sys.deferAfterCallback(reconcile);
        return;
      }
      reconcile();
    }
    replaceRegistration(next) {
      const prev = this.registration;
      this.registration = Object.freeze({
        trigger: typeof next.trigger === "undefined" ? this.registration.trigger : next.trigger,
        anchor: typeof next.anchor === "undefined" ? this.registration.anchor : next.anchor,
        content: typeof next.content === "undefined" ? this.registration.content : next.content
      });
      this.syncBoundaryRegistration("trigger", prev.trigger, this.registration.trigger);
      this.syncBoundaryRegistration("anchor", prev.anchor, this.registration.anchor);
      this.syncBoundaryRegistration("content", prev.content, this.registration.content);
    }
    syncBoundaryRegistration(role, prevTarget, nextTarget) {
      if (Object.is(prevTarget, nextTarget))
        return;
      this.boundaryDisposers[role]?.();
      this.boundaryDisposers[role] = null;
      if (typeof nextTarget === "undefined" || nextTarget === null)
        return;
      this.boundaryDisposers[role] = this.boundary.registerRegion(nextTarget, { role });
    }
    clearBoundaryRegistrations() {
      this.boundaryDisposers.trigger?.();
      this.boundaryDisposers.anchor?.();
      this.boundaryDisposers.content?.();
      this.boundaryDisposers.trigger = null;
      this.boundaryDisposers.anchor = null;
      this.boundaryDisposers.content = null;
    }
    patchValue(field, value) {
      if (typeof value === "undefined")
        return;
      pushOverrideWarning4(this.warnings, String(field), this.config[field], value);
      this.config = Object.freeze({
        ...this.config,
        [field]: value
      });
    }
    configure(patch) {
      this.ensureSetup("overlay.configure");
      this.patchValue("defaultOpen", patch.defaultOpen);
      this.patchValue("closeOnEscape", patch.closeOnEscape);
      this.patchValue("closeOnOutsidePress", patch.closeOnOutsidePress);
      this.patchValue("closeOnFocusOutside", patch.closeOnFocusOutside);
      this.patchValue("closeOnAnchorPress", patch.closeOnAnchorPress);
      this.patchValue("closeOnTriggerPress", patch.closeOnTriggerPress);
      this.patchValue("placement", patch.placement);
      this.patchValue("align", patch.align);
      this.patchValue("sideOffset", patch.sideOffset);
      this.patchValue("alignOffset", patch.alignOffset);
      this.patchValue("anchored", patch.anchored);
      this.patchValue("strategy", patch.strategy);
      this.patchValue("avoidCollisions", patch.avoidCollisions);
      this.patchValue("collisionBoundary", patch.collisionBoundary);
      this.patchValue("collisionPadding", patch.collisionPadding);
      this.patchValue("excludeAnchorTranslation", patch.excludeAnchorTranslation);
      this.patchValue("entry", patch.entry);
      this.patchValue("restore", patch.restore);
      this.patchValue("portal", patch.portal);
      this.patchValue("modal", patch.modal);
      this.patchValue("layerRole", patch.layerRole);
      this.patchValue("layerOffset", patch.layerOffset);
      if (typeof patch.meta !== "undefined") {
        this.config = Object.freeze({
          ...this.config,
          meta: mergeMeta4(this.config.meta, patch.meta)
        });
      }
      this.installDismissSampling();
      if (this.config.defaultOpen) {
        this.setOpen(true, "programmatic");
      }
    }
    updatePosition(patch) {
      const assign = (field, value) => {
        if (typeof value === "undefined")
          return;
        this.config = Object.freeze({ ...this.config, [field]: value });
      };
      assign("placement", patch.placement);
      assign("align", patch.align);
      assign("sideOffset", patch.sideOffset);
      assign("alignOffset", patch.alignOffset);
      assign("strategy", patch.strategy);
      assign("avoidCollisions", patch.avoidCollisions);
      assign("collisionBoundary", patch.collisionBoundary);
      assign("collisionPadding", patch.collisionPadding);
      assign("excludeAnchorTranslation", patch.excludeAnchorTranslation);
      if (this.viewActive)
        this.syncAnchoredPosition();
    }
    open(reason) {
      this.setOpen(true, reason);
    }
    close(reason) {
      this.setOpen(false, reason);
    }
    toggle(reason) {
      this.setOpen(!this.openState.handle.get(), reason);
    }
    isOpen() {
      return this.openState.handle.get();
    }
    getConfig() {
      return this.config;
    }
    getWarnings() {
      return Object.freeze(this.warnings.slice());
    }
    getLastReason() {
      return this.lastReason;
    }
    getRegistration() {
      return this.registration;
    }
    getPositionSnapshot() {
      return this.anchoredPosition.getSnapshot();
    }
    resolveAnchorTarget() {
      if (this.anchorPart) {
        const target = this.anatomyPort.resolvePartTarget(this.anchorPart);
        if (target)
          return target;
      }
      return this.registration.anchor ?? this.registration.trigger;
    }
    syncAnchorPartRegistration() {
      if (!this.anchorPart)
        return;
      const target = this.anatomyPort.resolvePartTarget(this.anchorPart);
      this.replaceRegistration({ anchor: target ?? null });
    }
    syncAnchoredPosition() {
      if (!this.config.anchored || !this.viewActive || this.mountPhase !== "mounted") {
        this.anchoredPosition.disconnect();
        return;
      }
      const anchor = this.resolveAnchorTarget();
      const floating = this.registration.content ?? this.resolveHostElement();
      if (!anchor || !floating) {
        this.anchoredPosition.disconnect();
        return;
      }
      this.anchoredPosition.connect({
        anchor,
        floating,
        config: {
          side: this.config.placement,
          align: this.config.align,
          sideOffset: this.config.sideOffset,
          alignOffset: this.config.alignOffset,
          strategy: this.config.strategy,
          avoidCollisions: this.config.avoidCollisions,
          collisionBoundary: this.config.collisionBoundary,
          collisionPadding: this.config.collisionPadding,
          excludeAnchorTranslation: this.config.excludeAnchorTranslation
        }
      });
    }
    registerTrigger(target) {
      this.replaceRegistration({ trigger: target });
      if (this.viewActive)
        this.reconcileViewResourcesAfterCallback();
    }
    registerAnchor(target) {
      this.anchorPart = null;
      this.replaceRegistration({ anchor: target });
      if (this.viewActive)
        this.reconcileViewResourcesAfterCallback();
    }
    registerAnchorPart(part) {
      this.anchorPart = part;
      this.syncAnchorPartRegistration();
      if (this.viewActive)
        this.reconcileViewResourcesAfterCallback();
    }
    registerContent(target) {
      this.replaceRegistration({ content: target });
      if (!this.viewActive)
        return;
      const hostEl = this.resolveHostElement();
      if (!hostEl)
        return;
      this.reconcileViewResourcesAfterCallback();
    }
    handle = {
      open: this.openState.handle,
      isOpen: () => this.isOpen(),
      openOverlay: (reason) => this.open(reason),
      close: (reason) => this.close(reason),
      toggle: (reason) => this.toggle(reason),
      configure: (patch) => this.configure(patch),
      updatePosition: (patch) => this.updatePosition(patch),
      registerTrigger: (target) => this.registerTrigger(target),
      registerAnchor: (target) => this.registerAnchor(target),
      registerAnchorPart: (part) => this.registerAnchorPart(part),
      registerContent: (target) => this.registerContent(target),
      getPositionSnapshot: () => this.getPositionSnapshot()
    };
  }

  // ../packages/modules/overlay/src/create.ts
  function createOverlayModule(ctx) {
    const { init, caps: caps14, deps } = ctx;
    return createModule({
      name: "overlay",
      scope: "instance",
      init,
      caps: caps14,
      deps,
      build: ({ init: init2, caps: caps15 }) => {
        const boundaryFacade = deps.requireFacade("boundary");
        const impl3 = new OverlayModuleImpl(caps15, init2.prototypeName, boundaryFacade.getBoundary(), deps.requirePort("boundary"), deps.requirePort("event"), deps.requirePort("anatomy"), deps.requireFacade("positioning").getAnchoredPosition());
        return {
          facade: {
            getOverlay: () => impl3.handle
          },
          hooks: {
            onMountPhase: (p, epoch) => impl3.onMountPhase(p, epoch),
            onProtoPhase: (p) => impl3.onProtoPhase(p)
          },
          port: {
            configure: (patch) => impl3.configure(patch),
            open: (reason) => impl3.open(reason),
            close: (reason) => impl3.close(reason),
            toggle: (reason) => impl3.toggle(reason),
            isOpen: () => impl3.isOpen(),
            getConfig: () => impl3.getConfig(),
            getWarnings: () => impl3.getWarnings(),
            getLastReason: () => impl3.getLastReason(),
            getRegistration: () => impl3.getRegistration(),
            getPositionSnapshot: () => impl3.getPositionSnapshot(),
            registerTrigger: (target) => impl3.registerTrigger(target),
            registerAnchor: (target) => impl3.registerAnchor(target),
            registerAnchorPart: (part) => impl3.registerAnchorPart(part),
            registerContent: (target) => impl3.registerContent(target),
            updatePosition: (patch) => impl3.updatePosition(patch),
            setViewActive: (active) => impl3.setViewActive(active),
            markPresenceBound: () => impl3.markPresenceBound(),
            hasPresenceBinding: () => impl3.hasPresenceBinding(),
            reconcileViewResourcesAfterCallback: () => impl3.reconcileViewResourcesAfterCallback()
          }
        };
      }
    });
  }
  var OverlayModuleDef = defineModule({
    name: "overlay",
    resourceOwnership: "mixed",
    deps: ["boundary", "event", "anatomy", "positioning"],
    create: createOverlayModule
  });
  // ../packages/modules/overlay/src/web/z-index-layer-scheduler.ts
  var DEFAULT_ROLE_OFFSETS = Object.freeze({
    overlay: 0,
    "dialog-mask": 1000,
    "dialog-content": 1010
  });
  // ../packages/modules/positioning/src/caps.ts
  var ANCHORED_POSITION_HOST_CAP = cap("@proto.ui/positioning/anchoredHost");

  // ../packages/modules/positioning/src/impl.ts
  class PositioningModuleImpl extends ModuleBase {
    connection = null;
    lease = null;
    snapshot = null;
    onCapsEpoch() {
      if (!this.connection)
        return;
      this.attach(this.connection);
    }
    onProtoPhase(phase) {
      super.onProtoPhase(phase);
      if (phase === "unmounted")
        this.disconnect();
    }
    getHost() {
      return this.caps.has(ANCHORED_POSITION_HOST_CAP) ? this.caps.get(ANCHORED_POSITION_HOST_CAP) : null;
    }
    bridge(connection) {
      const authoredResolved = connection.onResolved;
      return {
        ...connection,
        onResolved: (snapshot) => {
          this.snapshot = Object.freeze({ ...snapshot });
          authoredResolved?.(this.snapshot);
        }
      };
    }
    attach(connection) {
      this.lease?.dispose();
      this.lease = null;
      const host = this.getHost();
      if (!host)
        return;
      this.lease = host.attach(this.bridge(connection));
    }
    handle = {
      connect: (connection) => {
        const sameTargets = this.connection && Object.is(this.connection.anchor, connection.anchor) && Object.is(this.connection.floating, connection.floating);
        this.connection = connection;
        if (sameTargets && this.lease) {
          this.lease.update(this.bridge(connection));
          return;
        }
        this.snapshot = null;
        this.attach(connection);
      },
      update: (config) => {
        if (!this.connection)
          return;
        this.connection = { ...this.connection, config };
        if (this.lease) {
          this.lease.update(this.bridge(this.connection));
          return;
        }
        this.attach(this.connection);
      },
      requestUpdate: () => this.lease?.requestUpdate(),
      disconnect: () => this.disconnect(),
      getSnapshot: () => this.snapshot
    };
    disconnect() {
      this.lease?.dispose();
      this.lease = null;
      this.connection = null;
      this.snapshot = null;
    }
  }

  // ../packages/modules/positioning/src/create.ts
  function createPositioningModule(ctx) {
    const { init, caps: caps14, deps } = ctx;
    const impl3 = new PositioningModuleImpl(caps14);
    return createModule({
      name: "positioning",
      scope: "instance",
      init,
      caps: caps14,
      deps,
      build: () => ({
        facade: { getAnchoredPosition: () => impl3.handle },
        port: { getAnchoredPosition: () => impl3.handle },
        hooks: {
          onProtoPhase: (phase) => impl3.onProtoPhase(phase)
        }
      })
    });
  }
  var PositioningModuleDef = defineModule({
    name: "positioning",
    resourceOwnership: "mixed",
    deps: [],
    create: createPositioningModule
  });
  // ../packages/modules/scroll/src/caps.ts
  var SCROLL_SURFACE_HOST_CAP = cap("@proto.ui/scroll/surfaceHost");

  // ../packages/modules/scroll/src/projection.ts
  class ScrollProjectionResolutionError extends Error {
    requested;
    support;
    code = "PUI_SCROLL_PROJECTION_UNSUPPORTED";
    constructor(requested, support) {
      super(`[Scroll] requested ${requested} projection is unsupported by the host.`);
      this.requested = requested;
      this.support = support;
      this.name = "ScrollProjectionResolutionError";
    }
  }
  function resolveScrollProjection(config, support, hostPreference = "auto") {
    const required = config.requireProjection;
    if (required) {
      if (!support[required])
        throw new ScrollProjectionResolutionError(required, support);
      return required;
    }
    const preference = config.projection === "auto" ? hostPreference : config.projection;
    if (preference !== "auto" && support[preference])
      return preference;
    if (support.system)
      return "system";
    if (support.composed)
      return "composed";
    throw new ScrollProjectionResolutionError(preference, support);
  }

  // ../packages/modules/scroll/src/impl.ts
  var EMPTY_AXIS = Object.freeze({
    position: 0,
    visibleRatio: 1,
    canScrollBefore: false,
    canScrollAfter: false
  });
  var DEFAULT_CONFIG4 = Object.freeze({
    axes: "both",
    projection: "auto"
  });
  var clampRatio = (value) => Number.isFinite(value) ? Math.min(1, Math.max(0, value)) : 0;

  class ScrollModuleImpl extends ModuleBase {
    prototypeName;
    statePort;
    anatomyPort;
    contextPort;
    config = DEFAULT_CONFIG4;
    declared = false;
    composedChromeBinding = null;
    offAnatomyOrder = null;
    offAnatomyTargets = null;
    lease = null;
    mounted = false;
    axesOwned;
    scrollingOwned;
    projectionOwned;
    horizontalPositionOwned;
    horizontalVisibleOwned;
    horizontalBeforeOwned;
    horizontalAfterOwned;
    verticalPositionOwned;
    verticalVisibleOwned;
    verticalBeforeOwned;
    verticalAfterOwned;
    handle;
    constructor(caps14, prototypeName, statePort, stateFacade, anatomyPort, contextPort) {
      super(caps14);
      this.prototypeName = prototypeName;
      this.statePort = statePort;
      this.anatomyPort = anatomyPort;
      this.contextPort = contextPort;
      this.axesOwned = stateFacade.enum("@scroll/axes", "both", {
        options: ["horizontal", "vertical", "both"]
      });
      this.scrollingOwned = stateFacade.bool("@scroll/scrolling", false);
      this.projectionOwned = stateFacade.enum("@scroll/projection", "unresolved", {
        options: ["unresolved", "system", "composed"]
      });
      this.horizontalPositionOwned = this.createRatio(stateFacade, "@scroll/horizontalPosition", 0);
      this.horizontalVisibleOwned = this.createRatio(stateFacade, "@scroll/horizontalVisibleRatio", 1);
      this.horizontalBeforeOwned = stateFacade.bool("@scroll/horizontalCanScrollBefore", false);
      this.horizontalAfterOwned = stateFacade.bool("@scroll/horizontalCanScrollAfter", false);
      this.verticalPositionOwned = this.createRatio(stateFacade, "@scroll/verticalPosition", 0);
      this.verticalVisibleOwned = this.createRatio(stateFacade, "@scroll/verticalVisibleRatio", 1);
      this.verticalBeforeOwned = stateFacade.bool("@scroll/verticalCanScrollBefore", false);
      this.verticalAfterOwned = stateFacade.bool("@scroll/verticalCanScrollAfter", false);
      this.handle = {
        axes: this.observed(this.axesOwned),
        horizontal: {
          position: this.observed(this.horizontalPositionOwned),
          visibleRatio: this.observed(this.horizontalVisibleOwned),
          canScrollBefore: this.observed(this.horizontalBeforeOwned),
          canScrollAfter: this.observed(this.horizontalAfterOwned)
        },
        vertical: {
          position: this.observed(this.verticalPositionOwned),
          visibleRatio: this.observed(this.verticalVisibleOwned),
          canScrollBefore: this.observed(this.verticalBeforeOwned),
          canScrollAfter: this.observed(this.verticalAfterOwned)
        },
        scrolling: this.observed(this.scrollingOwned),
        projection: this.observed(this.projectionOwned),
        configure: (patch) => this.configure(patch),
        bindComposedChrome: (binding) => this.bindComposedChrome(binding),
        request: (request) => this.request(request),
        getSnapshot: () => this.getSnapshot()
      };
    }
    createRatio(stateFacade, semantic, value) {
      return stateFacade.numberRange(semantic, value, { min: 0, max: 1, clamp: true });
    }
    observed(handle) {
      return this.statePort.createObservedHandle(handle);
    }
    ensureSetup(operation) {
      this.sys?.ensureSetup(operation);
      if (!this.sys && this.protoPhase !== "setup") {
        throw illegalPhase(operation, this.protoPhase, { prototypeName: this.prototypeName });
      }
    }
    getSurface() {
      this.declared = true;
      if (this.mounted)
        this.attach();
      return this.handle;
    }
    configure(patch) {
      this.ensureSetup("asScrollSurface().configure");
      this.config = Object.freeze({
        ...this.config,
        ...patch,
        requireProjection: typeof patch.requireProjection === "undefined" ? this.config.requireProjection : patch.requireProjection
      });
      this.set(this.axesOwned, this.config.axes);
    }
    bindComposedChrome(binding) {
      this.ensureSetup("asScrollSurface().bindComposedChrome");
      if (this.composedChromeBinding && this.composedChromeBinding !== binding) {
        throw new Error("[Scroll] composed chrome may be bound only once per logical surface.");
      }
      this.composedChromeBinding = binding;
      if (!this.offAnatomyOrder) {
        const refresh = () => {
          if (!this.mounted || !this.lease)
            return;
          this.lease.update(this.createHostAttachment());
        };
        this.offAnatomyOrder = this.anatomyPort.subscribeOrder(binding.anatomy, refresh);
        this.offAnatomyTargets = this.anatomyPort.subscribeTargets(binding.anatomy, refresh);
      }
    }
    request(request) {
      if (!this.declared)
        return;
      const normalized = request.kind === "to" || request.kind === "control-drag" ? { ...request, position: clampRatio(request.position) } : request;
      this.lease?.request(normalized);
    }
    getConfig() {
      return this.config;
    }
    getSnapshot() {
      return Object.freeze({
        axes: this.axesOwned.get(),
        horizontal: Object.freeze({
          position: this.horizontalPositionOwned.get(),
          visibleRatio: this.horizontalVisibleOwned.get(),
          canScrollBefore: this.horizontalBeforeOwned.get(),
          canScrollAfter: this.horizontalAfterOwned.get()
        }),
        vertical: Object.freeze({
          position: this.verticalPositionOwned.get(),
          visibleRatio: this.verticalVisibleOwned.get(),
          canScrollBefore: this.verticalBeforeOwned.get(),
          canScrollAfter: this.verticalAfterOwned.get()
        }),
        scrolling: this.scrollingOwned.get(),
        projection: this.projectionOwned.get()
      });
    }
    onCapsEpoch() {
      if (this.mounted && this.declared)
        this.attach();
    }
    onMountPhase(phase, epoch) {
      super.onMountPhase(phase, epoch);
      this.mounted = phase === "mounted";
      if (phase === "mounted") {
        if (this.declared)
          this.attach();
        return;
      }
      if (phase === "detached")
        this.disconnect();
    }
    onProtoPhase(phase) {
      super.onProtoPhase(phase);
      if (phase === "unmounted")
        this.disconnect();
      if (phase === "unmounted") {
        this.offAnatomyOrder?.();
        this.offAnatomyOrder = null;
        this.offAnatomyTargets?.();
        this.offAnatomyTargets = null;
      }
    }
    getHost() {
      return this.caps.has(SCROLL_SURFACE_HOST_CAP) ? this.caps.get(SCROLL_SURFACE_HOST_CAP) : null;
    }
    attach() {
      this.lease?.dispose();
      this.lease = null;
      const host = this.getHost();
      if (!host) {
        this.set(this.projectionOwned, "unresolved");
        return;
      }
      const projection = resolveScrollProjection(this.config, host.support, host.preference);
      this.set(this.projectionOwned, projection);
      this.lease = host.attach(this.createHostAttachment());
    }
    createHostAttachment() {
      const composedChrome = this.resolveComposedChrome();
      const attachment = {
        config: this.config,
        projection: this.projectionOwned.get(),
        onFacts: (snapshot) => this.applySnapshot(snapshot),
        ...composedChrome ? { composedChrome } : {}
      };
      return Object.freeze(attachment);
    }
    resolveComposedChrome() {
      const binding = this.composedChromeBinding;
      if (!binding)
        return null;
      const anatomyScope = this.anatomyPort.resolveDomainScope(binding.anatomy);
      if (!anatomyScope)
        return null;
      const scope = this.contextPort.resolveScope(binding.scope, anatomyScope);
      if (!scope || scope !== anatomyScope)
        return null;
      const scrollbars = this.anatomyPort.order.partsOf(binding.anatomy, binding.scrollbarRole, {
        missing: "empty"
      });
      const controls = scrollbars.flatMap((scrollbar) => {
        const thumb = this.anatomyPort.descendantsOf(binding.anatomy, scrollbar, binding.thumbRole)[0];
        if (!thumb)
          return [];
        const trackTarget = this.anatomyPort.resolvePartTarget(scrollbar);
        const thumbTarget = this.anatomyPort.resolvePartTarget(thumb);
        if (!trackTarget || !thumbTarget)
          return [];
        return [
          Object.freeze({
            getAxis: () => this.readControlAxis(scrollbar, binding.orientationExpose),
            trackTarget,
            thumbTarget
          })
        ];
      });
      return Object.freeze({ scope, controls: Object.freeze(controls) });
    }
    readControlAxis(part, exposeKey) {
      const exposed = part.getExpose(exposeKey);
      let state2 = exposed;
      if (exposed && "kind" in exposed && exposed.kind === "state")
        state2 = exposed.state;
      return state2?.get?.() === "horizontal" ? "horizontal" : "vertical";
    }
    applySnapshot(snapshot) {
      this.set(this.axesOwned, snapshot.axes);
      this.applyAxis("horizontal", snapshot.horizontal);
      this.applyAxis("vertical", snapshot.vertical);
      this.set(this.scrollingOwned, snapshot.scrolling);
      this.set(this.projectionOwned, snapshot.projection);
    }
    applyAxis(axis, snapshot) {
      const value = snapshot ?? EMPTY_AXIS;
      const handles3 = axis === "horizontal" ? [
        this.horizontalPositionOwned,
        this.horizontalVisibleOwned,
        this.horizontalBeforeOwned,
        this.horizontalAfterOwned
      ] : [
        this.verticalPositionOwned,
        this.verticalVisibleOwned,
        this.verticalBeforeOwned,
        this.verticalAfterOwned
      ];
      this.set(handles3[0], clampRatio(value.position));
      this.set(handles3[1], clampRatio(value.visibleRatio));
      this.set(handles3[2], value.canScrollBefore);
      this.set(handles3[3], value.canScrollAfter);
    }
    set(handle, value) {
      if (Object.is(handle.get(), value))
        return;
      this.statePort.set(handle, value, "reason: scroll host fact");
    }
    disconnect() {
      this.lease?.dispose();
      this.lease = null;
      this.set(this.scrollingOwned, false);
      this.set(this.projectionOwned, "unresolved");
    }
  }

  // ../packages/modules/scroll/src/create.ts
  function createScrollModule(ctx) {
    const { init, caps: caps14, deps } = ctx;
    return createModule({
      name: "scroll",
      scope: "instance",
      init,
      caps: caps14,
      deps,
      build: ({ deps: deps2 }) => {
        const statePort = deps2.requirePort("state");
        const stateFacade = deps2.requireFacade("state");
        const impl3 = new ScrollModuleImpl(caps14, init.prototypeName, statePort, stateFacade, deps2.requirePort("anatomy"), deps2.requirePort("context"));
        return {
          facade: { getSurface: () => impl3.getSurface() },
          port: {
            configureSurface: (patch) => impl3.configure(patch),
            bindComposedChrome: (binding) => impl3.bindComposedChrome(binding),
            request: (request) => impl3.request(request),
            getConfig: () => impl3.getConfig(),
            getSnapshot: () => impl3.getSnapshot()
          },
          hooks: {
            onMountPhase: (phase, epoch) => impl3.onMountPhase(phase, epoch),
            onProtoPhase: (phase) => impl3.onProtoPhase(phase)
          }
        };
      }
    });
  }
  var ScrollModuleDef = defineModule({
    name: "scroll",
    resourceOwnership: "mixed",
    deps: ["state", "anatomy", "context"],
    create: createScrollModule
  });
  // ../packages/modules/presence/src/caps.ts
  var PRESENCE_HOST_BRIDGE_CAP = cap("@proto.ui/presence/hostBridge");
  // ../packages/modules/presence/src/impl.ts
  function isPromiseLike(value) {
    return value !== null && typeof value === "object" && typeof value.then === "function";
  }

  class PresenceModuleImpl extends ModuleBase {
    phase = "absent";
    hasHandle = false;
    mountResolvers = [];
    unmountResolvers = [];
    beforeMounts = [];
    beforeUnmounts = [];
    mountResolved = false;
    lifecycleDriver = null;
    policy = { mode: "transition" };
    immediateAbsentNotified = false;
    pendingMount = null;
    pendingUnmount = null;
    getBridge() {
      return this.caps.has(PRESENCE_HOST_BRIDGE_CAP) ? this.caps.get(PRESENCE_HOST_BRIDGE_CAP) : {
        mount: () => {},
        unmount: () => {}
      };
    }
    createHandle(policy) {
      this.hasHandle = true;
      this.policy = { mode: policy?.mode ?? "transition" };
      return {
        setIntent: (intent) => this.setIntent(intent),
        getPhase: () => this.phase,
        onBeforeMount: (cb) => {
          this.beforeMounts.push(cb);
          return () => {
            const idx = this.beforeMounts.indexOf(cb);
            if (idx >= 0)
              this.beforeMounts.splice(idx, 1);
          };
        },
        onBeforeUnmount: (cb) => {
          this.beforeUnmounts.push(cb);
          return () => {
            const idx = this.beforeUnmounts.indexOf(cb);
            if (idx >= 0)
              this.beforeUnmounts.splice(idx, 1);
          };
        }
      };
    }
    setIntent(intent) {
      if (intent === "enter") {
        this.immediateAbsentNotified = false;
        if (this.phase === "absent") {
          this.phase = "mounting";
          this.pendingUnmount = null;
          this.lifecycleDriver?.requestMount();
          this.runCbsSync(this.beforeMounts);
          const mountResult = this.getBridge().mount();
          this.pendingMount = mountResult ?? null;
          if (isPromiseLike(mountResult)) {
            const activeMount = mountResult;
            mountResult.then(() => {
              if (this.pendingMount !== activeMount)
                return;
              this.pendingMount = null;
              this.resolveMounts();
              this.phase = "present";
            }, () => {
              if (this.pendingMount !== activeMount)
                return;
              this.pendingMount = null;
              this.resolveMounts();
              this.phase = "present";
            });
          } else {
            this.pendingMount = null;
            this.resolveMounts();
            this.phase = "present";
          }
        } else if (this.phase === "unmounting") {
          const settle = () => {
            this.pendingUnmount = null;
            this.resolveUnmounts();
            this.phase = "present";
          };
          this.lifecycleDriver?.requestMount();
          if (this.pendingUnmount != null) {
            this.pendingUnmount = null;
            const mountResult = this.getBridge().mount();
            this.pendingMount = mountResult ?? null;
            if (isPromiseLike(mountResult)) {
              const activeMount = mountResult;
              mountResult.then(() => {
                if (this.pendingMount !== activeMount)
                  return;
                this.pendingMount = null;
                settle();
              }, () => {
                if (this.pendingMount !== activeMount)
                  return;
                this.pendingMount = null;
                settle();
              });
            } else {
              this.pendingMount = null;
              settle();
            }
          } else {
            settle();
          }
        }
      } else {
        if (this.phase === "present") {
          this.phase = "unmounting";
          if (this.policy.mode !== "immediate")
            return;
        }
        if (this.phase === "unmounting") {
          this.runCbsSync(this.beforeUnmounts);
          this.lifecycleDriver?.requestUnmount();
          this.pendingMount = null;
          const unmountResult = this.getBridge().unmount(this.policy.mode === "immediate" ? { immediate: true } : undefined);
          this.pendingUnmount = unmountResult ?? null;
          if (isPromiseLike(unmountResult)) {
            const activeUnmount = unmountResult;
            unmountResult.then(() => {
              if (this.pendingUnmount !== activeUnmount)
                return;
              this.pendingUnmount = null;
              this.resolveUnmounts();
              this.phase = "absent";
              this.mountResolved = false;
            }, () => {
              if (this.pendingUnmount !== activeUnmount)
                return;
              this.pendingUnmount = null;
              this.resolveUnmounts();
              this.phase = "absent";
              this.mountResolved = false;
            });
          } else {
            this.pendingUnmount = null;
            this.resolveUnmounts();
            this.phase = "absent";
            this.mountResolved = false;
          }
        } else if (this.phase === "mounting") {
          const settle = () => {
            this.pendingMount = null;
            this.resolveMounts();
            this.phase = "absent";
            this.mountResolved = false;
          };
          if (this.pendingMount != null) {
            this.pendingMount = null;
            const unmountResult = this.getBridge().unmount(this.policy.mode === "immediate" ? { immediate: true } : undefined);
            this.pendingUnmount = unmountResult ?? null;
            if (isPromiseLike(unmountResult)) {
              const activeUnmount = unmountResult;
              unmountResult.then(() => {
                if (this.pendingUnmount !== activeUnmount)
                  return;
                this.pendingUnmount = null;
                settle();
              }, () => {
                if (this.pendingUnmount !== activeUnmount)
                  return;
                this.pendingUnmount = null;
                settle();
              });
            } else {
              this.pendingUnmount = null;
              settle();
            }
          } else {
            settle();
          }
        } else if (this.phase === "absent") {
          if (this.policy.mode === "immediate" && !this.immediateAbsentNotified) {
            this.immediateAbsentNotified = true;
            this.runCbsSync(this.beforeUnmounts);
            this.lifecycleDriver?.requestUnmount();
            const unmountResult = this.getBridge().unmount({ immediate: true });
            if (isPromiseLike(unmountResult))
              Promise.resolve(unmountResult).catch(() => {});
          }
          this.resolveMounts();
        }
      }
    }
    runCbsSync(cbs) {
      for (const cb of cbs) {
        const result = cb();
        if (isPromiseLike(result)) {
          result.catch(() => {});
        }
      }
    }
    resolveMounts() {
      for (const r of this.mountResolvers)
        r();
      this.mountResolvers = [];
      this.mountResolved = true;
    }
    resolveUnmounts() {
      for (const r of this.unmountResolvers)
        r();
      this.unmountResolvers = [];
    }
    awaitMount() {
      if (!this.hasHandle || this.phase === "present" || this.phase === "unmounting") {
        return;
      }
      if (this.mountResolved)
        return;
      return new Promise((resolve) => {
        this.mountResolvers.push(resolve);
      });
    }
    awaitUnmount() {
      if (!this.hasHandle || this.phase === "absent")
        return;
      return new Promise((resolve) => {
        this.unmountResolvers.push(resolve);
      });
    }
    forceUnmount() {
      if (!this.hasHandle || this.phase === "absent")
        return;
      this.runCbsSync(this.beforeUnmounts);
      this.pendingMount = null;
      this.pendingUnmount = null;
      try {
        const result = this.getBridge().unmount({ immediate: true });
        if (isPromiseLike(result))
          Promise.resolve(result).catch(() => {});
      } finally {
        this.phase = "absent";
        this.mountResolved = false;
        this.resolveMounts();
        this.resolveUnmounts();
      }
    }
    setLifecycleDriver(driver) {
      this.lifecycleDriver = driver;
    }
  }

  // ../packages/modules/presence/src/create.ts
  function createPresenceModule(ctx) {
    const { init, caps: caps14 } = ctx;
    const impl3 = new PresenceModuleImpl(caps14);
    return createModule({
      name: "presence",
      scope: "instance",
      init,
      caps: caps14,
      deps: ctx.deps,
      build: () => ({
        facade: {
          createHandle: (policy) => impl3.createHandle(policy)
        },
        hooks: {},
        port: {
          awaitMount: () => impl3.awaitMount(),
          awaitUnmount: () => impl3.awaitUnmount(),
          forceUnmount: () => impl3.forceUnmount(),
          setLifecycleDriver: (driver) => impl3.setLifecycleDriver(driver)
        }
      })
    });
  }
  var PresenceModuleDef = defineModule({
    name: "presence",
    resourceOwnership: "mixed",
    deps: [],
    create: createPresenceModule
  });
  // ../packages/modules/test-sys/src/impl.ts
  class TestSysImpl {
    sys;
    prototypeName;
    traceLog = [];
    constructor(caps14, prototypeName) {
      this.prototypeName = prototypeName;
      this.sys = caps14.get(SYS_CAP);
    }
    snapshot(label) {
      return {
        label,
        t: Date.now(),
        execPhase: String(this.sys.execPhase?.() ?? "unknown"),
        domain: this.sys.domain(),
        protoPhase: String(this.sys.protoPhase()),
        disposed: this.sys.isDisposed(),
        prototypeName: this.prototypeName
      };
    }
    trace(label) {
      const s = this.snapshot(label);
      this.traceLog.push(s);
      return s;
    }
    getTrace() {
      return this.traceLog;
    }
    clearTrace() {
      this.traceLog = [];
    }
    port() {
      return {
        snapshot: (label) => this.snapshot(label),
        trace: (label) => this.trace(label),
        getTrace: () => this.getTrace(),
        clearTrace: () => this.clearTrace()
      };
    }
  }

  // ../packages/modules/test-sys/src/create.ts
  function createTestSysModule(ctx) {
    const { init, caps: caps14, deps } = ctx;
    return createModule({
      name: "test-sys",
      scope: "instance",
      init,
      caps: caps14,
      deps,
      build: ({ init: init2, caps: caps15 }) => {
        const impl3 = new TestSysImpl(caps15, init2.prototypeName);
        return {
          facade: {},
          hooks: {},
          port: impl3.port()
        };
      }
    });
  }
  var TestSysModuleDef = defineModule({
    name: "test-sys",
    resourceOwnership: "instance",
    deps: [],
    create: createTestSysModule
  });

  // ../packages/modules/test-sys/src/index.ts
  var __RUN_TEST_SYS = "__testSys";

  // ../packages/modules/text-control/src/caps.ts
  var TEXT_CONTROL_HOST_CAP = cap("@proto.ui/text-control/host");
  var TEXT_CONTROL_RUN_IN_CALLBACK_CAP = cap("@proto.ui/text-control/run-in-callback");

  // ../packages/modules/text-control/src/declaration.ts
  var TEXT_CONTROL_DECLARATION = moduleDeclaration("@proto.ui/text-control/declaration");
  function declareTextControl(config) {
    return declareModule(TEXT_CONTROL_DECLARATION, config);
  }

  // ../packages/modules/text-control/src/impl.ts
  var EMPTY_PATCH = Object.freeze({});

  class TextControlModuleImpl extends ModuleBase {
    prototypeName;
    supported;
    declaration;
    declared = false;
    initialized = false;
    valueMode = null;
    patch = EMPTY_PATCH;
    value = "";
    composing = false;
    listeners = [];
    host = null;
    lease = null;
    constructor(caps14, prototypeName, declarations) {
      super(caps14);
      this.prototypeName = prototypeName;
      const declaration = getModuleDeclaration({ modules: declarations }, TEXT_CONTROL_DECLARATION);
      this.declaration = declaration?.config ?? null;
      this.supported = this.declaration !== null;
      if (this.supported)
        this.refreshHost();
    }
    declare() {
      this.sys.ensureSetup("textControl.declare");
      if (!this.supported) {
        throw new Error(`[TextControl] ${this.prototypeName} requires a static text-control declaration.`);
      }
      if (this.declared) {
        throw new Error(`[TextControl] ${this.prototypeName} may declare one text control.`);
      }
      this.declared = true;
      return {
        on: (type, callback) => this.on(type, callback),
        sync: (patch) => this.sync(patch),
        snapshot: () => this.snapshot()
      };
    }
    on(type, callback) {
      this.sys.ensureSetup("textControl.on");
      const listener = {
        type,
        callback
      };
      this.listeners = this.listeners.concat(listener);
      return () => {
        this.listeners = this.listeners.filter((candidate) => candidate !== listener);
      };
    }
    sync(next) {
      this.sys.ensureCallback("textControl.sync");
      if (this.declaration?.lineMode === "single" && (typeof next.rows === "number" || next.wrap !== undefined)) {
        throw new Error("[TextControl] rows/wrap are not compatible with single-line mode");
      }
      if (!this.initialized) {
        this.valueMode = next.valueMode ?? "uncontrolled";
        this.value = this.valueMode === "controlled" ? this.canonicalize(next.value ?? "") : this.canonicalize(next.defaultValue ?? "");
        this.initialized = true;
      }
      this.patch = Object.freeze({
        ...this.patch,
        ...next,
        valueMode: this.valueMode ?? "uncontrolled",
        value: typeof next.value === "string" ? this.canonicalize(next.value) : this.patch.value,
        defaultValue: typeof next.defaultValue === "string" ? this.canonicalize(next.defaultValue) : this.patch.defaultValue
      });
      if (this.valueMode === "controlled")
        this.value = this.canonicalize(this.patch.value ?? "");
      this.syncLease();
    }
    snapshot() {
      return this.declared ? Object.freeze({ value: this.value, composing: this.composing }) : null;
    }
    onCapsEpoch() {
      this.refreshHost();
      this.attachLease();
    }
    onMountPhase(phase, epoch) {
      super.onMountPhase(phase, epoch);
      if (phase === "mounted") {
        this.refreshHost();
        this.attachLease();
        return;
      }
      if (phase === "unmounting" || phase === "detached")
        this.disposeLease();
    }
    dispose() {
      this.disposeLease();
      this.listeners = [];
      this.declared = false;
    }
    refreshHost() {
      this.host = this.caps.has(TEXT_CONTROL_HOST_CAP) ? this.caps.get(TEXT_CONTROL_HOST_CAP) : null;
    }
    attachLease() {
      this.disposeLease();
      if (!this.declared || !this.host || this.mountPhase !== "mounted")
        return;
      this.lease = this.host.attach({
        patch: this.effectivePatch(),
        onEvent: (event2) => this.receive(event2)
      });
    }
    disposeLease() {
      this.lease?.dispose();
      this.lease = null;
    }
    effectivePatch() {
      const { value: _declaredValue, ...patchWithoutValue } = this.patch;
      const shouldProjectValue = !(this.valueMode === "controlled" && this.composing);
      return Object.freeze({
        ...patchWithoutValue,
        valueMode: this.valueMode ?? "uncontrolled",
        ...shouldProjectValue ? { value: this.value } : {}
      });
    }
    syncLease() {
      this.lease?.update(this.effectivePatch());
    }
    receive(event2) {
      const canonicalEvent = Object.freeze({
        ...event2,
        value: this.canonicalize(event2.value),
        data: typeof event2.data === "string" ? this.canonicalize(event2.data) : event2.data
      });
      this.composing = canonicalEvent.composing;
      if (this.valueMode === "uncontrolled" && canonicalEvent.type === "input") {
        this.value = canonicalEvent.value;
      }
      const runInCallback = this.caps.has(TEXT_CONTROL_RUN_IN_CALLBACK_CAP) ? this.caps.get(TEXT_CONTROL_RUN_IN_CALLBACK_CAP) : (callback) => callback();
      runInCallback(() => {
        const run2 = this.sys.getCallbackCtx();
        if (!run2)
          return;
        for (const listener of this.listeners) {
          if (listener.type === canonicalEvent.type)
            listener.callback(run2, canonicalEvent);
        }
      });
      const mustRestoreControlledValue = this.valueMode === "controlled" && (event2.type === "input" && !event2.composing || event2.type === "compositionend");
      if (!mustRestoreControlledValue)
        return;
      queueMicrotask(() => this.syncLease());
    }
    canonicalize(value) {
      const lineMode = this.declaration?.lineMode ?? "multiline";
      return canonicalizeTextControlValue(value, lineMode);
    }
  }

  // ../packages/modules/text-control/src/create.ts
  function createTextControlModule(ctx) {
    return createModule({
      name: "text-control",
      scope: "instance",
      init: ctx.init,
      caps: ctx.caps,
      deps: ctx.deps,
      build: ({ init, caps: caps14 }) => {
        const impl3 = new TextControlModuleImpl(caps14, init.prototypeName, init.declarations);
        return {
          facade: {
            declare: () => impl3.declare()
          },
          hooks: {
            onMountPhase: (phase, epoch) => impl3.onMountPhase(phase, epoch),
            dispose: () => impl3.dispose()
          },
          port: {
            isDeclared: () => impl3.snapshot() !== null,
            getSnapshot: () => impl3.snapshot()
          }
        };
      }
    });
  }
  var TextControlModuleDef = defineModule({
    name: "text-control",
    resourceOwnership: "mixed",
    create: createTextControlModule
  });
  // ../packages/modules/image-view/src/caps.ts
  var IMAGE_VIEW_HOST_CAP = cap("@proto.ui/image-view/host");
  var IMAGE_VIEW_RUN_IN_CALLBACK_CAP = cap("@proto.ui/image-view/run-in-callback");

  // ../packages/modules/image-view/src/declaration.ts
  var IMAGE_VIEW_DECLARATION = moduleDeclaration("@proto.ui/image-view/declaration");

  // ../packages/modules/image-view/src/create.ts
  var EMPTY_PATCH2 = Object.freeze({});

  class ImageViewModuleImpl extends ModuleBase {
    prototypeName;
    declaration;
    declared = false;
    initialized = false;
    requestedSource = "";
    source = "";
    alternativeText = "";
    a11yMode = "informative";
    fit = "contain";
    loadingStatus = "idle";
    generation = 0;
    patch = EMPTY_PATCH2;
    listeners = [];
    host = null;
    lease = null;
    attachmentEpoch = 0;
    lastHostedGeneration = null;
    lastDiagnostic = "";
    constructor(caps14, prototypeName, declarations) {
      super(caps14);
      this.prototypeName = prototypeName;
      this.declaration = getModuleDeclaration({ modules: declarations }, IMAGE_VIEW_DECLARATION)?.config ?? null;
      if (this.declaration) {
        this.requestedSource = this.declaration.source;
        this.alternativeText = this.declaration.alternativeText;
        this.a11yMode = this.declaration.a11yMode;
        this.fit = this.declaration.fit;
        this.source = this.validatedSource();
        if (this.source) {
          this.generation = 1;
          this.loadingStatus = "loading";
        }
        this.refreshHost();
      }
    }
    declare() {
      this.sys.ensureSetup("imageView.declare");
      if (!this.declaration) {
        throw new Error(`[ImageView] ${this.prototypeName} requires a static image-view declaration.`);
      }
      if (this.declared) {
        throw new Error(`[ImageView] ${this.prototypeName} may declare one image view.`);
      }
      this.declared = true;
      return {
        on: (type, callback) => this.on(type, callback),
        sync: (patch) => this.sync(patch),
        snapshot: () => this.snapshot()
      };
    }
    on(type, callback) {
      this.sys.ensureSetup("imageView.on");
      const listener = {
        callback
      };
      this.listeners = this.listeners.concat(listener);
      return () => {
        this.listeners = this.listeners.filter((candidate) => candidate !== listener);
      };
    }
    sync(next) {
      this.sys.ensureCallback("imageView.sync");
      const { loadingStatus: _moduleOwnedStatus, ...portableNext } = next;
      this.patch = Object.freeze({ ...this.patch, ...portableNext });
      if (!this.initialized) {
        this.requestedSource = portableNext.source ?? this.requestedSource;
        this.alternativeText = portableNext.alternativeText ?? this.alternativeText;
        this.a11yMode = portableNext.a11yMode ?? this.a11yMode;
        this.fit = portableNext.fit ?? this.fit;
        this.initialized = true;
      } else {
        if (typeof portableNext.source === "string")
          this.requestedSource = portableNext.source;
        if (typeof portableNext.alternativeText === "string") {
          this.alternativeText = portableNext.alternativeText;
        }
        if (portableNext.a11yMode)
          this.a11yMode = portableNext.a11yMode;
        if (portableNext.fit)
          this.fit = portableNext.fit;
      }
      const source = this.validatedSource();
      if (source !== this.source) {
        this.source = source;
        this.generation += 1;
        this.transition(source ? "loading" : "idle");
      }
      this.syncLease();
    }
    snapshot() {
      return this.declared ? Object.freeze({
        source: this.source,
        loadingStatus: this.loadingStatus,
        fit: this.fit
      }) : null;
    }
    onCapsEpoch() {
      this.disposeLease();
      this.refreshHost();
      this.attachLease();
    }
    onMountPhase(phase, epoch) {
      super.onMountPhase(phase, epoch);
      if (phase === "mounted") {
        this.refreshHost();
        this.attachLease();
        return;
      }
      if (phase === "unmounting" || phase === "detached")
        this.disposeLease();
    }
    dispose() {
      this.disposeLease();
      this.listeners = [];
      this.declared = false;
    }
    refreshHost() {
      this.host = this.caps.has(IMAGE_VIEW_HOST_CAP) ? this.caps.get(IMAGE_VIEW_HOST_CAP) : null;
    }
    attachLease() {
      this.disposeLease();
      if (!this.host || !this.declared || this.mountPhase !== "mounted")
        return;
      if (this.source && this.lastHostedGeneration === this.generation) {
        this.generation += 1;
        this.transition("loading");
      }
      const host = this.host;
      const generation = this.generation;
      const attachmentEpoch = ++this.attachmentEpoch;
      const initialPatch = this.effectivePatch();
      this.lastHostedGeneration = generation;
      const lease = host.attach({
        generation,
        patch: initialPatch,
        onStatusChange: (change) => this.receive(change, attachmentEpoch)
      });
      if (attachmentEpoch !== this.attachmentEpoch || this.host !== host || this.mountPhase !== "mounted") {
        lease.dispose();
        return;
      }
      this.lease = lease;
      if (generation !== this.generation || initialPatch.loadingStatus !== this.loadingStatus) {
        this.syncLease();
      }
    }
    disposeLease() {
      this.attachmentEpoch += 1;
      this.lease?.dispose();
      this.lease = null;
    }
    effectivePatch() {
      return Object.freeze({
        ...this.patch,
        source: this.source,
        alternativeText: this.a11yMode === "decorative" ? "" : this.alternativeText,
        a11yMode: this.a11yMode,
        fit: this.fit,
        loadingStatus: this.loadingStatus
      });
    }
    syncLease() {
      if (!this.lease)
        return;
      this.lastHostedGeneration = this.generation;
      this.lease.update({
        generation: this.generation,
        patch: this.effectivePatch()
      });
    }
    receive(change, attachmentEpoch) {
      if (attachmentEpoch !== this.attachmentEpoch)
        return;
      if (change.generation !== this.generation)
        return;
      if (!this.source)
        return;
      if (this.loadingStatus === "loaded" || this.loadingStatus === "error")
        return;
      this.transition(change.status);
      this.syncLease();
    }
    transition(status) {
      if (status === this.loadingStatus)
        return;
      const currentRun = this.sys.getCallbackCtx();
      if (this.listeners.length > 0 && !currentRun && !this.caps.has(IMAGE_VIEW_RUN_IN_CALLBACK_CAP)) {
        throw new Error(`[ImageView] ${this.prototypeName} requires IMAGE_VIEW_RUN_IN_CALLBACK_CAP to dispatch loadingStatusChange outside callback scope.`);
      }
      const event2 = Object.freeze({
        source: this.source,
        previousStatus: this.loadingStatus,
        status
      });
      this.loadingStatus = status;
      const dispatch = (run2) => {
        for (const listener of [...this.listeners])
          listener.callback(run2, event2);
      };
      if (currentRun) {
        dispatch(currentRun);
        return;
      }
      if (this.listeners.length === 0)
        return;
      this.caps.get(IMAGE_VIEW_RUN_IN_CALLBACK_CAP)(() => {
        const run2 = this.sys.getCallbackCtx();
        if (!run2) {
          throw new Error(`[ImageView] ${this.prototypeName} received an invalid IMAGE_VIEW_RUN_IN_CALLBACK_CAP implementation that did not enter callback scope.`);
        }
        dispatch(run2);
      });
    }
    validatedSource() {
      if (!this.requestedSource) {
        this.lastDiagnostic = "";
        return "";
      }
      const hasAlternative = this.alternativeText.trim().length > 0;
      const valid = this.a11yMode === "informative" && hasAlternative || this.a11yMode === "decorative" && !hasAlternative;
      if (valid) {
        this.lastDiagnostic = "";
        return this.requestedSource;
      }
      const diagnostic = `${this.a11yMode}:${this.requestedSource}:${this.alternativeText}`;
      if (diagnostic !== this.lastDiagnostic) {
        this.lastDiagnostic = diagnostic;
        console.warn(`[ImageView] ${this.prototypeName} rejected contradictory or missing accessibility input; source failed closed to idle.`);
      }
      return "";
    }
  }
  function createImageViewModule(ctx) {
    const { init, caps: caps14, deps } = ctx;
    return createModule({
      name: "image-view",
      scope: "instance",
      init,
      caps: caps14,
      deps,
      build: ({ init: init2, caps: caps15 }) => {
        const impl3 = new ImageViewModuleImpl(caps15, init2.prototypeName, init2.declarations);
        return {
          facade: {
            declare: () => impl3.declare()
          },
          hooks: {
            onMountPhase: (phase, epoch) => impl3.onMountPhase(phase, epoch),
            dispose: () => impl3.dispose()
          },
          port: {
            isDeclared: () => impl3.snapshot() !== null,
            getSnapshot: () => impl3.snapshot()
          }
        };
      }
    });
  }
  var ImageViewModuleDef = defineModule({
    name: "image-view",
    resourceOwnership: "mixed",
    create: createImageViewModule
  });
  // ../packages/runtime/src/orchestrator/module-orchestrator/graph.ts
  function buildModuleGraph(prototypeName, modules) {
    const seen = new Set;
    for (const m of modules) {
      if (seen.has(m.name)) {
        throw new Error(`[Runtime] duplicate module name: ${prototypeName}/${m.name}`);
      }
      seen.add(m.name);
    }
    const byName = new Map;
    const depsByName = new Map;
    for (const m of modules) {
      byName.set(m.name, m);
      depsByName.set(m.name, {
        hard: new Set(m.deps ?? []),
        optional: new Set(m.optionalDeps ?? [])
      });
    }
    const hardDeps = (m) => m.deps ?? [];
    const optDeps = (m) => m.optionalDeps ?? [];
    for (const m of modules) {
      for (const d of hardDeps(m)) {
        if (!byName.has(d)) {
          throw new Error(`[Runtime] missing module dependency: ${prototypeName}/${m.name} deps -> ${d}`);
        }
      }
    }
    const indeg = new Map;
    const out = new Map;
    for (const m of modules) {
      indeg.set(m.name, 0);
      out.set(m.name, []);
    }
    const addEdge = (from, to) => {
      out.get(from).push(to);
      indeg.set(to, (indeg.get(to) ?? 0) + 1);
    };
    for (const m of modules) {
      for (const d of hardDeps(m))
        addEdge(d, m.name);
      for (const d of optDeps(m)) {
        if (byName.has(d))
          addEdge(d, m.name);
      }
    }
    const q = [];
    for (const [name, v] of indeg)
      if (v === 0)
        q.push(name);
    const order = [];
    while (q.length) {
      const cur = q.shift();
      order.push(cur);
      for (const nxt of out.get(cur)) {
        const v = (indeg.get(nxt) ?? 0) - 1;
        indeg.set(nxt, v);
        if (v === 0)
          q.push(nxt);
      }
    }
    if (order.length !== modules.length) {
      const remains = [...indeg.entries()].filter(([, v]) => v > 0).map(([k]) => k).join(", ");
      throw new Error(`[Runtime] module dependency cycle: ${prototypeName} remains=[${remains}]`);
    }
    return { order: order.map((n) => byName.get(n)), byName, depsByName };
  }

  // ../packages/runtime/src/orchestrator/module-orchestrator/runtime-module-orchestrator.ts
  class RuntimeModuleOrchestrator {
    prototypeName;
    getExecPhase;
    protoPhase = "setup";
    instancePhase = "setup";
    mountPhase = "detached";
    disposed = false;
    records = [];
    recordByName = new Map;
    facades = {};
    ports = {};
    wiring;
    callbackCtx = undefined;
    afterCallbackTasks = [];
    constructor(init, modules) {
      this.prototypeName = init.prototypeName;
      this.getExecPhase = init.getPhase;
      const declarations = Object.freeze((init.declarations ?? []).slice());
      const fail = (msg) => {
        throw new Error(`[Runtime] ${msg}`);
      };
      const sys = {
        execPhase: () => this.getExecPhase(),
        domain: () => this.getExecPhase() === "setup" ? "setup" : "runtime",
        protoPhase: () => this.protoPhase,
        instancePhase: () => this.instancePhase,
        mountPhase: () => this.mountPhase,
        isDisposed: () => this.disposed,
        ensureNotDisposed: (op) => {
          if (this.disposed)
            fail(`${this.prototypeName} is disposed. op=${op}`);
        },
        ensureExecPhase: (op, expected) => {
          if (this.disposed)
            fail(`${this.prototypeName} is disposed. op=${op}`);
          const actual = this.getExecPhase();
          const ex = Array.isArray(expected) ? expected : [expected];
          if (!ex.includes(actual)) {
            fail(`exec-phase violation: ${this.prototypeName} op=${op} ` + `expected=${ex.join("|")} actual=${actual} protoPhase=${this.protoPhase}`);
          }
        },
        ensureSetup: (op) => {
          sys.ensureExecPhase(op, "setup");
        },
        ensureRuntime: (op) => {
          if (this.disposed)
            fail(`${this.prototypeName} is disposed. op=${op}`);
          if (this.getExecPhase() === "setup") {
            fail(`runtime-only violation: ${this.prototypeName} op=${op} ` + `actual=setup protoPhase=${this.protoPhase}`);
          }
        },
        ensureCallback: (op) => {
          sys.ensureExecPhase(op, "callback");
        },
        getCallbackCtx: () => {
          return this.getExecPhase() === "callback" ? this.callbackCtx : undefined;
        },
        deferAfterCallback: (task) => {
          this.afterCallbackTasks.push(task);
        }
      };
      const graph = buildModuleGraph(this.prototypeName, modules);
      for (const m of graph.order) {
        const vault2 = new CapsVault;
        vault2.attachBase([[SYS_CAP, sys]]);
        const controller = this.createController(m.name, vault2);
        const deps = this.createDepsAccess(m.name, graph.depsByName.get(m.name));
        const module2 = m.create({
          init: { prototypeName: this.prototypeName, declarations },
          caps: vault2,
          deps
        });
        const rec = { name: m.name, vault: vault2, controller, module: module2 };
        this.records.push(rec);
        this.recordByName.set(m.name, rec);
        this.facades[m.name] = module2.facade;
        if (module2.port !== undefined) {
          this.ports[m.name] = module2.port;
        }
      }
      this.wiring = {
        attach: (moduleName, entries) => {
          const rec = this.recordByName.get(moduleName);
          if (!rec)
            return false;
          rec.controller.attach(entries);
          return true;
        },
        reset: (moduleName) => {
          if (!moduleName) {
            for (const r of this.records) {
              try {
                r.controller.reset();
              } catch {}
            }
            return;
          }
          const rec = this.recordByName.get(moduleName);
          if (!rec)
            return;
          rec.controller.reset();
        }
      };
    }
    createDepsAccess(moduleName, spec2) {
      const allow = new Set([...spec2.hard, ...spec2.optional]);
      const require2 = (name) => {
        if (!allow.has(name)) {
          throw new Error(`[Runtime] ${this.prototypeName}/${moduleName} tried to access undeclared dep: ${name}`);
        }
      };
      const requireFacade = (name) => {
        require2(name);
        const f = this.facades[name];
        if (!f) {
          throw new Error(`[Runtime] ${this.prototypeName}/${moduleName} missing dep facade: ${name}`);
        }
        return f;
      };
      const requirePort = (name) => {
        require2(name);
        const p = this.ports[name];
        if (!p) {
          throw new Error(`[Runtime] ${this.prototypeName}/${moduleName} missing dep port: ${name}`);
        }
        return p;
      };
      const tryFacade = (name) => {
        require2(name);
        return this.facades[name];
      };
      const tryPort = (name) => {
        require2(name);
        return this.ports[name];
      };
      return { requireFacade, requirePort, tryFacade, tryPort };
    }
    createController(moduleName, vault2) {
      const prototypeName = this.prototypeName;
      const findReserved = (entries) => {
        for (const [token] of entries) {
          if (token.id === SYS_CAP.id)
            return token.id;
        }
        return null;
      };
      return {
        attach: (entries) => {
          const reserved = findReserved(entries);
          if (reserved) {
            throw new Error(`[Wiring] ${prototypeName}/${moduleName} attempted to provide reserved cap: ${reserved}`);
          }
          vault2.attach(entries);
        },
        reset: () => {
          vault2.resetAttached();
        }
      };
    }
    setProtoPhase(phase) {
      this.protoPhase = phase;
      for (const r of this.records) {
        r.module.hooks.onProtoPhase?.(phase);
      }
    }
    setInstancePhase(phase) {
      this.instancePhase = phase;
      for (const r of this.records) {
        r.module.hooks.onInstancePhase?.(phase);
      }
    }
    setMountPhase(phase, epoch) {
      this.mountPhase = phase;
      for (const r of this.records) {
        r.module.hooks.onMountPhase?.(phase, epoch);
      }
    }
    afterRenderCommit() {
      for (const r of this.records) {
        r.module.hooks.afterRenderCommit?.();
      }
    }
    getFacades() {
      return this.facades;
    }
    getPorts() {
      return this.ports;
    }
    getPort(moduleName) {
      return this.ports[moduleName];
    }
    getCapsController(moduleName) {
      return this.recordByName.get(moduleName)?.controller;
    }
    getWiring() {
      return this.wiring;
    }
    __setCallbackCtx(ctx) {
      this.callbackCtx = ctx;
    }
    __getCallbackCtx() {
      return this.callbackCtx;
    }
    __flushAfterCallbackTasks() {
      while (this.afterCallbackTasks.length > 0) {
        const tasks = this.afterCallbackTasks;
        this.afterCallbackTasks = [];
        for (const task of tasks)
          task();
      }
    }
    dispose() {
      if (this.disposed)
        return;
      this.disposed = true;
      this.callbackCtx = undefined;
      this.afterCallbackTasks = [];
      for (const r of this.records) {
        r.module.hooks.dispose?.();
      }
      for (const r of this.records) {
        try {
          r.vault.resetAttached();
        } catch {}
      }
    }
  }
  // ../packages/runtime/src/instance/execute/callback-scope.ts
  class CallbackScope {
    getPhase;
    setPhase;
    moduleHub;
    delayContext;
    depth = 0;
    constructor(getPhase, setPhase, moduleHub) {
      this.getPhase = getPhase;
      this.setPhase = setPhase;
      this.moduleHub = moduleHub;
    }
    setDelayContext(ctx) {
      this.delayContext = ctx;
    }
    syncPropsFromHost() {
      const propsPort = this.moduleHub.getPort("props");
      propsPort?.syncFromHost?.();
    }
    dispatchPropsTasks(ctx) {
      const propsPort = this.moduleHub.getPort("props");
      const tasks = propsPort?.consumeTasks?.() ?? [];
      for (const t of tasks) {
        t.cb(ctx, t.next, t.prev, t.info);
      }
    }
    run(ctx, fn) {
      const prevPhase = this.getPhase();
      const prevCtx = this.moduleHub.__getCallbackCtx?.();
      this.setPhase("callback");
      this.depth += 1;
      this.moduleHub.__setCallbackCtx?.(ctx);
      if (this.delayContext)
        enterActiveRuntimeDelayContext(this.delayContext);
      try {
        this.syncPropsFromHost();
        this.dispatchPropsTasks(ctx);
        return fn();
      } finally {
        if (this.delayContext)
          exitActiveRuntimeDelayContext();
        this.moduleHub.__setCallbackCtx?.(prevCtx);
        this.setPhase(prevPhase);
        this.depth -= 1;
        if (this.depth === 0)
          this.moduleHub.__flushAfterCallbackTasks?.();
      }
    }
    runNoSync(ctx, fn) {
      const prevPhase = this.getPhase();
      const prevCtx = this.moduleHub.__getCallbackCtx?.();
      this.setPhase("callback");
      this.depth += 1;
      this.moduleHub.__setCallbackCtx?.(ctx);
      if (this.delayContext)
        enterActiveRuntimeDelayContext(this.delayContext);
      try {
        this.dispatchPropsTasks(ctx);
        return fn();
      } finally {
        if (this.delayContext)
          exitActiveRuntimeDelayContext();
        this.moduleHub.__setCallbackCtx?.(prevCtx);
        this.setPhase(prevPhase);
        this.depth -= 1;
        if (this.depth === 0)
          this.moduleHub.__flushAfterCallbackTasks?.();
      }
    }
    syncAndDispatch(ctx) {
      this.syncPropsFromHost();
      this.dispatchPropsTasks(ctx);
    }
  }

  // ../packages/runtime/src/instance/instance.ts
  function createRuntimeInstance(proto, opt) {
    let phaseRef = "unknown";
    const getPhase = () => phaseRef;
    const moduleHub = new RuntimeModuleOrchestrator({ prototypeName: proto.name, declarations: proto.modules, getPhase }, [
      AsTriggerModuleDef,
      RuleModuleDef,
      RuleMetaModuleDef,
      FeedbackModuleDef,
      PropsModuleDef,
      EventModuleDef,
      ExposeModuleDef,
      ExposeEventModuleDef,
      AnatomyModuleDef,
      ExposeStateModuleDef,
      ExposeStateWebModuleDef,
      RuleExposeStateWebModuleDef,
      StateModuleDef,
      StateInteractionModuleDef,
      StateAccessibilityModuleDef,
      A11yModuleDef,
      CollectionModuleDef,
      ContextModuleDef,
      FocusModuleDef,
      TextControlModuleDef,
      ImageViewModuleDef,
      BoundaryModuleDef,
      HitParticipationModuleDef,
      PositioningModuleDef,
      ScrollModuleDef,
      OverlayModuleDef,
      PresenceModuleDef,
      TestSysModuleDef
    ]);
    opt?.onModulesReady?.(moduleHub);
    const kernel3 = createKernel(proto, moduleHub, {
      allowRunUpdate: opt?.allowRunUpdate,
      onPhaseChange: (p) => {
        phaseRef = p;
      },
      asHook: {
        projectState: createAsHookStateProjector(moduleHub.getPort("state")),
        enterSetup: ({ def: def2, rt }) => {
          enterActiveAsHookContext({
            def: def2,
            rt,
            facades: moduleHub.getFacades(),
            ports: moduleHub.getPorts()
          });
        },
        exitSetup: () => {
          exitActiveAsHookContext();
        }
      },
      eventSink: {
        setEventCallbacks: (callbacks) => {
          moduleHub[__RT_EVENT_CALLBACKS] = callbacks;
        }
      }
    });
    phaseRef = kernel3.getPhase();
    const callbackScope = new CallbackScope(() => kernel3.getPhase(), (p) => kernel3.setPhase(p), moduleHub);
    const statePort = moduleHub.getPort("state");
    statePort?.setCallbackDispatcher?.((fn) => {
      callbackScope.runNoSync(kernel3.run, () => fn(kernel3.run));
    });
    const contextPort = moduleHub.getPort("context");
    contextPort?.setCallbackDispatcher?.((fn) => {
      callbackScope.runNoSync(kernel3.run, () => fn(kernel3.run));
    });
    const anatomyPort = moduleHub.getPort("anatomy");
    anatomyPort?.setOrderCallbackDispatcher?.((fn) => {
      callbackScope.runNoSync(kernel3.run, () => fn(kernel3.run));
    });
    const testSys = moduleHub.getPort("test-sys");
    if (testSys) {
      Object.defineProperty(kernel3.run, __RUN_TEST_SYS, {
        value: testSys,
        enumerable: false,
        configurable: false,
        writable: false
      });
    }
    const renderOnce = () => kernel3.renderOnce();
    const runLifecycle = (kind) => {
      callbackScope.run(kernel3.run, () => {
        for (const cb of kernel3.lifecycle[kind]) {
          cb(kernel3.run);
        }
      });
    };
    const dispose = () => {
      moduleHub.dispose();
    };
    return {
      kernel: kernel3,
      moduleHub,
      callbackScope,
      renderOnce,
      runLifecycle,
      dispose
    };
  }
  // ../packages/runtime/src/instance/session.ts
  function createRuntimeSession(proto, host) {
    const emit = (event2) => {
      host.onLifecycleEvent?.(event2);
      const legacy = projectLegacyCheckpoint(event2);
      if (legacy)
        host.onLifecycleCheckpoint?.(legacy);
    };
    const inst = createRuntimeInstance(proto, {
      allowRunUpdate: true,
      onModulesReady: (hub) => {
        host.onRuntimeReady?.(hub.getWiring());
      }
    });
    const { kernel: kernel3, moduleHub, callbackScope } = inst;
    const { lifecycle, run: run2 } = kernel3;
    const propsFacade = moduleHub.getFacades()["props"];
    const propsPort = moduleHub.getPort("props");
    const rulePort = moduleHub.getPort("rule");
    if (!propsPort)
      throw new Error("props port not found");
    let instancePhase = "setup";
    let mountPhase = "detached";
    let mountEpoch = 0;
    let transitionVersion = 0;
    let revision = 0;
    let dirty = false;
    let children = [];
    let updateInFlight = false;
    let updateQueued = false;
    let mountPending;
    let unmountPending;
    let unmountVersion = 0;
    let disposePending;
    const pendingDelayTasks = new Set;
    const cancelPendingDelayTasks = () => {
      for (const task of [...pendingDelayTasks])
        task.cancel();
      pendingDelayTasks.clear();
    };
    callbackScope.setDelayContext({
      prototypeName: host.prototypeName,
      scheduleDelay(durationMs, callback) {
        if (instancePhase !== "alive") {
          throw new Error(`[Delay] cannot schedule delayed work in instance phase=${instancePhase}: ${host.prototypeName}`);
        }
        if (!host.scheduleDelay) {
          throw new Error(`[Delay] host scheduler is not available for ${host.prototypeName}. Provide RuntimeHost.scheduleDelay.`);
        }
        let active = true;
        let hostTask;
        const task = {
          cancel() {
            if (!active)
              return;
            active = false;
            pendingDelayTasks.delete(task);
            hostTask?.cancel();
          }
        };
        const invoke = () => {
          if (!active)
            return;
          active = false;
          pendingDelayTasks.delete(task);
          if (instancePhase !== "alive")
            return;
          callbackScope.run(run2, callback);
        };
        pendingDelayTasks.add(task);
        try {
          hostTask = host.scheduleDelay(durationMs, invoke);
        } catch (error5) {
          active = false;
          pendingDelayTasks.delete(task);
          throw error5;
        }
        return task;
      }
    });
    const setInstancePhase = (phase) => {
      instancePhase = phase;
      moduleHub.setInstancePhase(phase);
      emit({ type: "instance.phase", phase });
    };
    const setMountPhase = (phase, epoch = mountEpoch) => {
      mountPhase = phase;
      moduleHub.setMountPhase(phase, epoch);
      emit({ type: "mount.phase", phase, epoch });
    };
    const bindEvents = () => {
      const eventPort = moduleHub.getPort("event");
      const eventRegistry = moduleHub[__RT_EVENT_CALLBACKS];
      if (!eventPort?.bind)
        return;
      eventPort.bind((id, ev) => {
        if (instancePhase !== "alive" || mountPhase !== "mounted")
          return;
        callbackScope.run(run2, () => {
          eventPort.dispatchInternal?.(id, ev);
          eventRegistry?.dispatch(run2, id, ev);
        });
      });
    };
    const renderCommit = (kind, epoch, updateRevision, onCommitted) => {
      propsPort.syncFromHost();
      children = inst.renderOnce();
      if (kind === "mount") {
        emit({ type: "mount.render", epoch });
        emit({ type: "mount.commit.start", epoch });
      } else {
        emit({ type: "update.render", epoch, revision: updateRevision });
      }
      let commitDone = false;
      host.commit(children, {
        done() {
          if (commitDone)
            return;
          commitDone = true;
          const activeCommit = instancePhase === "alive" && epoch === mountEpoch && (kind === "mount" ? mountPhase === "mounting" || mountPhase === "mounted" : mountPhase === "mounted");
          if (activeCommit) {
            if (kind === "mount") {
              emit({ type: "mount.commit.done", epoch });
            } else {
              emit({ type: "update.commit.done", epoch, revision: updateRevision });
            }
            bindEvents();
            moduleHub.afterRenderCommit();
          }
          onCommitted();
        }
      });
    };
    const evaluateRuleStyle = () => {
      propsPort.syncFromHost();
      const current = propsFacade.get();
      if (!rulePort)
        return [];
      const result = rulePort.evaluate({ props: current });
      return result.kind === "plan" && result.plan.kind === "style.tokens" ? result.plan.tokens : [];
    };
    const startUpdate = () => {
      if (instancePhase !== "alive")
        return;
      if (mountPhase !== "mounted") {
        callbackScope.run(run2, () => {});
        dirty = true;
        return;
      }
      if (updateInFlight) {
        updateQueued = true;
        return;
      }
      updateInFlight = true;
      const epoch = mountEpoch;
      const currentRevision = ++revision;
      try {
        renderCommit("update", epoch, currentRevision, () => {
          updateInFlight = false;
          if (instancePhase !== "alive" || mountPhase !== "mounted" || epoch !== mountEpoch) {
            dirty = true;
            updateQueued = false;
            return;
          }
          moduleHub.setProtoPhase("updated");
          emit({ type: "update.updated", epoch, revision: currentRevision });
          callbackScope.run(run2, () => {
            for (const cb of lifecycle.updated)
              cb(run2);
          });
          if (!updateQueued)
            return;
          updateQueued = false;
          startUpdate();
        });
      } catch (error5) {
        updateInFlight = false;
        updateQueued = false;
        throw error5;
      }
    };
    const controller = {
      applyRawProps(nextRaw) {
        if (instancePhase === "disposed")
          return;
        propsPort.applyRaw({ ...nextRaw ?? {} });
        callbackScope.runNoSync(run2, () => {});
      },
      update() {
        if (instancePhase === "disposing" || instancePhase === "disposed")
          return;
        startUpdate();
      },
      getRuleStyleTokens() {
        return evaluateRuleStyle();
      }
    };
    run2.update = () => controller.update();
    emit({ type: "instance.setup.exit" });
    propsPort.applyRaw({ ...host.getRawProps?.() ?? {} });
    setInstancePhase("alive");
    callbackScope.run(run2, () => {
      for (const cb of lifecycle.created)
        cb(run2);
    });
    emit({ type: "instance.created" });
    const mount = () => {
      if (instancePhase !== "alive") {
        return Promise.reject(new Error(`[Lifecycle] cannot mount instance in phase=${instancePhase}`));
      }
      if (mountPhase === "mounted")
        return Promise.resolve();
      if (mountPhase === "mounting" && mountPending)
        return mountPending.promise;
      if (mountPhase === "unmounting" && unmountPending) {
        return unmountPending.then(() => mount());
      }
      const epoch = ++mountEpoch;
      const version = ++transitionVersion;
      setMountPhase("mounting", epoch);
      let resolveMount;
      let rejectMount;
      const promise = new Promise((resolve, reject) => {
        resolveMount = resolve;
        rejectMount = reject;
      });
      mountPending = { promise, resolve: resolveMount, reject: rejectMount };
      const finishMount = () => {
        if (transitionVersion !== version || instancePhase !== "alive" || mountPhase !== "mounted" || mountEpoch !== epoch) {
          resolveMount();
          return;
        }
        emit({ type: "mount.mounted", epoch });
        callbackScope.run(run2, () => {
          for (const cb of lifecycle.mounted)
            cb(run2);
        });
        dirty = false;
        mountPending = undefined;
        resolveMount();
      };
      try {
        renderCommit("mount", epoch, 0, () => {
          const presence = moduleHub.getPort("presence")?.awaitMount();
          const scheduleFinish = () => {
            if (transitionVersion !== version) {
              resolveMount();
              return;
            }
            moduleHub.setProtoPhase("mounted");
            setMountPhase("mounted", epoch);
            let scheduleReturned = false;
            host.schedule(() => {
              try {
                finishMount();
              } catch (error5) {
                mountPending = undefined;
                if (!scheduleReturned)
                  throw error5;
                rejectMount(error5);
              }
            });
            scheduleReturned = true;
          };
          if (presence)
            presence.then(scheduleFinish, scheduleFinish);
          else
            scheduleFinish();
        });
      } catch (error5) {
        mountPending = undefined;
        setMountPhase("detached", epoch);
        resolveMount();
        throw error5;
      }
      return promise;
    };
    const unmountInternal = (force = false) => {
      if (mountPhase === "detached")
        return Promise.resolve();
      if (mountPhase === "unmounting" && unmountPending && !force)
        return unmountPending;
      const epoch = mountEpoch;
      const currentUnmountVersion = ++unmountVersion;
      ++transitionVersion;
      mountPending?.resolve();
      mountPending = undefined;
      setMountPhase("unmounting", epoch);
      cancelPendingDelayTasks();
      unmountPending = (async () => {
        const presencePort = moduleHub.getPort("presence");
        if (force)
          presencePort?.forceUnmount();
        const presence = force ? undefined : presencePort?.awaitUnmount();
        if (presence)
          await presence;
        if (currentUnmountVersion !== unmountVersion)
          return;
        emit({ type: "unmount.begin", epoch });
        host.onUnmountBegin?.();
        moduleHub.getPort("event")?.unbind?.();
        let callbackError;
        try {
          callbackScope.run(run2, () => {
            for (const cb of lifecycle.unmounted)
              cb(run2);
          });
        } catch (error5) {
          callbackError = error5;
        }
        setMountPhase("detached", epoch);
        cancelPendingDelayTasks();
        emit({ type: "unmount.done", epoch });
        unmountPending = undefined;
        if (callbackError)
          throw callbackError;
      })();
      return unmountPending;
    };
    const unmount = () => unmountInternal(false);
    const dispose = () => {
      if (instancePhase === "disposed")
        return Promise.resolve();
      if (disposePending)
        return disposePending;
      setInstancePhase("disposing");
      cancelPendingDelayTasks();
      kernel3.viewIntent.lockTerminal();
      emit({ type: "instance.dispose.begin" });
      const finalizeDispose = () => {
        let finalError;
        try {
          callbackScope.run(run2, () => {
            for (const cb of lifecycle.beforeDispose)
              cb(run2);
          });
        } catch (error5) {
          finalError = error5;
        }
        const eventRegistry = moduleHub[__RT_EVENT_CALLBACKS];
        eventRegistry?.clear?.();
        moduleHub.setProtoPhase("unmounted");
        moduleHub.getPort("presence")?.setLifecycleDriver(null);
        cancelPendingDelayTasks();
        inst.dispose();
        setInstancePhase("disposed");
        emit({ type: "instance.dispose.done" });
        return finalError;
      };
      const unmountResult = unmountInternal(true);
      if (mountPhase === "detached") {
        const finalError = finalizeDispose();
        disposePending = unmountResult.then(() => {
          if (finalError)
            throw finalError;
        }, (unmountError) => {
          throw unmountError;
        });
      } else {
        disposePending = unmountResult.then(() => {
          const finalError = finalizeDispose();
          if (finalError)
            throw finalError;
        }, (unmountError) => {
          finalizeDispose();
          throw unmountError;
        });
      }
      return disposePending;
    };
    if (host.presenceLifecycle === "session") {
      moduleHub.getPort("presence")?.setLifecycleDriver({
        requestMount() {
          mount();
        },
        requestUnmount() {
          unmount();
        }
      });
    }
    return {
      controller,
      get instancePhase() {
        return instancePhase;
      },
      get mountPhase() {
        return mountPhase;
      },
      get mountEpoch() {
        return mountEpoch;
      },
      get children() {
        return children;
      },
      viewIntent: kernel3.viewIntent,
      mount,
      unmount,
      dispose,
      caps: moduleHub,
      invokeInCallbackScope: (fn) => callbackScope.run(run2, fn),
      kernel: kernel3
    };
  }
  // ../packages/hooks/src/privileged.ts
  function definePrivilegedAsHook(definition) {
    return () => {
      const active = getActiveAsHookContext(definition.name);
      active.rt.ensureSetup(`asHook(${definition.name})`);
      const registration = active.rt.register(definition.name, {
        privileged: true,
        mode: definition.mode ?? "once"
      });
      const ctx = {
        def: active.def,
        rt: active.rt,
        facades: active.facades,
        ports: active.ports,
        registration
      };
      if (registration.action === "skip") {
        const result2 = definition.reuse ? definition.reuse(ctx) : registration.state.result;
        active.rt.recordAsHookResult({
          name: definition.name,
          order: registration.order,
          privileged: true,
          mode: definition.mode ?? "once",
          result: result2,
          handle: result2
        });
        return result2;
      }
      const result = definition.setup(ctx);
      registration.state.result = result;
      active.rt.recordAsHookResult({
        name: definition.name,
        order: registration.order,
        privileged: true,
        mode: definition.mode ?? "once",
        result,
        handle: result
      });
      return result;
    };
  }

  // ../packages/hooks/src/as-boundary.ts
  var getBoundary = definePrivilegedAsHook({
    name: "asBoundary",
    setup: ({ facades }) => {
      const facade = facades.boundary;
      if (!facade || typeof facade.getBoundary !== "function") {
        throw new Error("[AsHook] boundary facade unavailable for asBoundary.");
      }
      return facade.getBoundary();
    }
  });
  function asBoundary() {
    return getBoundary();
  }
  // ../packages/hooks/src/as-collection.ts
  var DEFAULT_ITEM_ROLE = "item";
  var DEFAULT_ITEM_META_EXPOSE_KEY = "__collectionItem";
  var asCollection = definePrivilegedAsHook({
    name: "asCollection",
    setup({ def: def2, ports }) {
      const collection2 = ports.collection;
      if (!collection2) {
        throw new Error("[AsHook] collection port unavailable for asCollection.");
      }
      const count = def2.state.numberDiscrete("collectionCount", 0);
      const store = {
        configured: false,
        offOrder: undefined,
        items: []
      };
      const sync = () => {
        if (!store.configured)
          return;
        const items = collection2.readProviderItems();
        store.items = items;
        count.set(items.length, "reason: asCollection.sync => collection count");
      };
      const handle = {
        count,
        configure: (patch) => {
          const itemRole = patch.itemRole ?? DEFAULT_ITEM_ROLE;
          const ownerRole = patch.ownerRole ?? patch.rootRole ?? false;
          const itemMetaExposeKey = patch.itemMetaExposeKey ?? DEFAULT_ITEM_META_EXPOSE_KEY;
          collection2.configureProvider({
            family: patch.family,
            itemRole,
            itemMetaExposeKey
          });
          if (ownerRole) {
            def2.anatomy.claim(patch.family, { role: ownerRole });
          }
          def2.expose.state(patch.exposeCountStateKey ?? "count", count);
          def2.expose.method(patch.exposeItemsMethodKey ?? "getCollectionItems", () => collection2.readProviderItems());
          def2.expose.method(patch.exposeCountMethodKey ?? "getCollectionCount", () => collection2.readProviderCount());
          store.configured = true;
        },
        getItems: () => collection2.readProviderItems(),
        getCount: () => collection2.readProviderCount()
      };
      def2.lifecycle.onMounted(() => {
        sync();
        store.offOrder = collection2.subscribeProvider(sync);
      });
      def2.lifecycle.onUpdated(() => {
        sync();
      });
      def2.lifecycle.onUnmounted(() => {
        store.offOrder?.();
        store.offOrder = undefined;
      });
      return handle;
    },
    reuse({ registration }) {
      return registration.state.result;
    }
  });
  // ../packages/hooks/src/as-collection-item.ts
  var DEFAULT_ROLE = "item";
  var DEFAULT_META_EXPOSE_KEY = "__collectionItem";
  var asCollectionItem = definePrivilegedAsHook({
    name: "asCollectionItem",
    setup({ def: def2, ports }) {
      const collection2 = ports.collection;
      if (!collection2) {
        throw new Error("[AsHook] collection port unavailable for asCollectionItem.");
      }
      const index = def2.state.numberDiscrete("collectionIndex", -1);
      const total = def2.state.numberDiscrete("collectionTotal", 0);
      const first = def2.state.bool("collectionFirst", false);
      const last = def2.state.bool("collectionLast", false);
      const store = {
        configured: false,
        offOrder: undefined,
        getMeta: undefined,
        run: undefined,
        snapshot: {
          index: -1,
          total: 0,
          first: false,
          last: false
        }
      };
      const readMeta = () => {
        if (store.getMeta && store.run)
          return store.getMeta(store.run);
        const { index: _index, total: _total, first: _first, last: _last, ...meta } = store.snapshot;
        return meta;
      };
      const writeSnapshot = (snapshot) => {
        index.set(snapshot.index, "reason: asCollectionItem.sync => index");
        total.set(snapshot.total, "reason: asCollectionItem.sync => total");
        first.set(snapshot.first, "reason: asCollectionItem.sync => first");
        last.set(snapshot.last, "reason: asCollectionItem.sync => last");
        store.snapshot = snapshot;
      };
      const buildSnapshot = () => {
        const meta = readMeta();
        const position = collection2.readItemPosition();
        const lastKnownPosition = {
          index: store.snapshot.index,
          total: store.snapshot.total,
          first: store.snapshot.first,
          last: store.snapshot.last
        };
        const effectivePosition = position.index < 0 && position.total === 0 ? lastKnownPosition : position;
        return {
          ...meta,
          ...effectivePosition
        };
      };
      const sync = () => {
        if (!store.configured)
          return;
        writeSnapshot(buildSnapshot());
      };
      const handle = {
        collectionIndex: index,
        collectionTotal: total,
        collectionFirst: first,
        collectionLast: last,
        configure: (patch) => {
          const role = patch.role ?? DEFAULT_ROLE;
          const metaExposeKey = patch.metaExposeKey ?? DEFAULT_META_EXPOSE_KEY;
          store.getMeta = patch.getMeta;
          collection2.configureItem({
            family: patch.family,
            role
          });
          def2.anatomy.claim(patch.family, { role });
          def2.expose.state(patch.exposeIndexStateKey ?? "collectionIndex", index);
          def2.expose.state(patch.exposeTotalStateKey ?? "collectionTotal", total);
          def2.expose.state(patch.exposeFirstStateKey ?? "collectionFirst", first);
          def2.expose.state(patch.exposeLastStateKey ?? "collectionLast", last);
          def2.expose.method(patch.exposeSnapshotMethodKey ?? "getCollectionItem", () => buildSnapshot());
          def2.expose.method(metaExposeKey, () => buildSnapshot());
          store.configured = true;
        },
        getSnapshot: () => buildSnapshot()
      };
      def2.lifecycle.onMounted((run2) => {
        store.run = run2;
        sync();
        store.offOrder = collection2.subscribeItem(sync);
      });
      def2.lifecycle.onUpdated((run2) => {
        store.run = run2;
        sync();
      });
      def2.lifecycle.onUnmounted(() => {
        store.offOrder?.();
        store.offOrder = undefined;
        store.run = undefined;
      });
      return handle;
    },
    reuse({ registration }) {
      return registration.state.result;
    }
  });
  // ../packages/hooks/src/as-focus-entry.ts
  var getFocusEntry = definePrivilegedAsHook({
    name: "asFocusEntry",
    setup: ({ facades }) => {
      const facade = facades.focus;
      if (!facade || typeof facade.getEntry !== "function") {
        throw new Error(`[AsHook] focus facade unavailable for asFocusEntry.`);
      }
      return facade.getEntry();
    }
  });
  function asFocusEntry() {
    return getFocusEntry();
  }
  // ../packages/hooks/src/as-focus-roving.ts
  var getFocusRoving = definePrivilegedAsHook({
    name: "asFocusRoving",
    setup: ({ facades }) => {
      const facade = facades.focus;
      if (!facade || typeof facade.getRoving !== "function") {
        throw new Error(`[AsHook] focus facade unavailable for asFocusRoving.`);
      }
      return facade.getRoving();
    }
  });
  function asFocusRoving() {
    return getFocusRoving();
  }
  // ../packages/hooks/src/as-focusable.ts
  var getFocusable = definePrivilegedAsHook({
    name: "asFocusable",
    setup: ({ facades }) => {
      const facade = facades.focus;
      if (!facade || typeof facade.getFocusable !== "function") {
        throw new Error(`[AsHook] focus facade unavailable for asFocusable.`);
      }
      return facade.getFocusable();
    }
  });
  function asFocusable() {
    return getFocusable();
  }
  // ../packages/hooks/src/as-focus-scope.ts
  var getFocusScope = definePrivilegedAsHook({
    name: "asFocusScope",
    setup: ({ facades }) => {
      const facade = facades.focus;
      if (!facade || typeof facade.getScope !== "function") {
        throw new Error(`[AsHook] focus facade unavailable for asFocusScope.`);
      }
      return facade.getScope();
    }
  });
  function asFocusScope() {
    return getFocusScope();
  }
  // ../packages/hooks/src/as-hit-participation.ts
  var getHitParticipation = definePrivilegedAsHook({
    name: "asHitParticipation",
    setup: ({ facades }) => {
      const facade = facades["hit-participation"];
      if (!facade || typeof facade.getHitParticipation !== "function") {
        throw new Error("[AsHook] hit-participation facade unavailable for asHitParticipation.");
      }
      return facade.getHitParticipation();
    }
  });
  function asHitParticipation(patch) {
    const handle = getHitParticipation();
    if (patch)
      handle.configure(patch);
    return handle;
  }
  // ../packages/hooks/src/as-image-view.ts
  var getImageView = definePrivilegedAsHook({
    name: "asImageView",
    setup: ({ facades }) => {
      const facade = facades["image-view"];
      if (!facade || typeof facade.declare !== "function") {
        throw new Error("[AsHook] image-view facade unavailable for asImageView.");
      }
      return facade.declare();
    }
  });
  // ../packages/hooks/src/as-text-control.ts
  function asTextControl() {
    return getTextControl();
  }
  var getTextControl = definePrivilegedAsHook({
    name: "asTextControl",
    setup: ({ facades }) => {
      const facade = facades["text-control"];
      if (!facade || typeof facade.declare !== "function") {
        throw new Error("[AsHook] text-control facade unavailable for asTextControl.");
      }
      return facade.declare();
    }
  });
  // ../packages/hooks/src/as-overlay.ts
  var installOverlay = definePrivilegedAsHook({
    name: "asOverlay",
    setup: ({ def: def2, rt, facades, ports }) => {
      const facade = facades.overlay;
      const port = ports.overlay;
      if (!facade || typeof facade.getOverlay !== "function" || !port) {
        throw new Error(`[AsHook] overlay capability unavailable for asOverlay.`);
      }
      const raw = facade.getOverlay();
      let currentRun = null;
      let presenceBinding = null;
      let retainedView = false;
      let offPresence = null;
      let disposed = false;
      const scheduleBoundViewActive = (active) => {
        port.setViewActive(active);
        if (!active)
          return;
        port.reconcileViewResourcesAfterCallback();
      };
      const driveLogicalPresence = (open) => {
        if (!currentRun || disposed)
          return;
        if (presenceBinding) {
          if (open)
            presenceBinding.enter();
          else
            presenceBinding.leave();
          return;
        }
        port.setViewActive(open);
        if (retainedView)
          return;
        currentRun.lifecycle.setPresent(open);
      };
      const offOpen = raw.open.watch((_run, event2) => {
        if (event2.type !== "next")
          return;
        driveLogicalPresence(event2.next);
      });
      def2.lifecycle.onCreated((run2) => {
        currentRun = run2;
        if (presenceBinding)
          return;
        const open = raw.isOpen();
        port.setViewActive(open);
        run2.lifecycle.setPresent(retainedView ? true : open);
      });
      def2.lifecycle.onMounted((run2) => {
        currentRun = run2;
        const target = run2.host?.get?.();
        if (target)
          raw.registerContent(target);
      });
      def2.lifecycle.onUnmounted((run2) => {
        currentRun = run2;
      });
      def2.lifecycle.onBeforeDispose(() => {
        disposed = true;
        offPresence?.();
        offPresence = null;
        offOpen();
        currentRun = null;
      });
      const handle = {
        open: raw.open,
        isOpen: () => raw.isOpen(),
        openOverlay: (reason) => raw.openOverlay(reason),
        close: (reason) => raw.close(reason),
        toggle: (reason) => raw.toggle(reason),
        configure: (patch) => raw.configure(patch),
        updatePosition: (patch) => raw.updatePosition(patch),
        registerTrigger: (target) => raw.registerTrigger(target),
        registerAnchor: (target) => raw.registerAnchor(target),
        registerAnchorPart: (part) => raw.registerAnchorPart(part),
        registerContent: (target) => raw.registerContent(target),
        getPositionSnapshot: () => raw.getPositionSnapshot(),
        keepMounted() {
          rt.ensureSetup("overlay.keepMounted");
          if (presenceBinding) {
            throw new Error("[asOverlay] cannot keep mounted after Presence binding.");
          }
          retainedView = true;
        },
        bindPresence(binding) {
          rt.ensureSetup("overlay.bindPresence");
          if (presenceBinding === binding)
            return;
          if (presenceBinding) {
            throw new Error("[asOverlay] presence is already bound for this prototype instance.");
          }
          if (retainedView) {
            throw new Error("[asOverlay] cannot bind Presence after keepMounted().");
          }
          presenceBinding = binding;
          port.markPresenceBound();
          port.setViewActive(binding.present.get());
          offPresence = binding.present.watch((_run, event2) => {
            if (event2.type !== "next")
              return;
            scheduleBoundViewActive(event2.next);
          });
          def2.lifecycle.onCreated((run2) => {
            currentRun = run2;
            port.setViewActive(binding.present.get());
            if (raw.isOpen())
              binding.enter();
            else
              binding.leave();
          });
        }
      };
      return Object.freeze(handle);
    }
  });
  function asOverlay() {
    return installOverlay();
  }
  // ../packages/hooks/src/as-scroll-surface.ts
  var getScrollSurface = definePrivilegedAsHook({
    name: "asScrollSurface",
    setup: ({ facades }) => {
      const facade = facades.scroll;
      if (!facade || typeof facade.getSurface !== "function") {
        throw new Error("[AsHook] scroll facade unavailable for asScrollSurface.");
      }
      return facade.getSurface();
    }
  });
  // ../packages/hooks/src/as-trigger.ts
  var asTrigger = definePrivilegedAsHook({
    name: "asTrigger",
    setup: ({ facades }) => {
      const facade = facades["as-trigger"];
      if (!facade || typeof facade.apply !== "function") {
        throw new Error(`[AsHook] asTrigger facade unavailable.`);
      }
      facade.apply();
    }
  });
  // ../packages/prototypes/base/src/textarea/root.proto.ts
  function setupTextareaRoot(def2) {
    def2.props.define({
      value: { type: "string", empty: "fallback" },
      defaultValue: { type: "string", empty: "fallback" },
      disabled: { type: "boolean", empty: "fallback" },
      readOnly: { type: "boolean", empty: "fallback" },
      placeholder: { type: "string", empty: "fallback" },
      rows: { type: "number", empty: "fallback" },
      required: { type: "boolean", empty: "fallback" },
      name: { type: "string", empty: "fallback" },
      autoComplete: { type: "string", empty: "fallback" },
      minLength: { type: "number", empty: "fallback" },
      maxLength: { type: "number", empty: "fallback" },
      wrap: { type: "enum", empty: "fallback", options: ["soft", "hard"] },
      ariaLabel: { type: "string", empty: "fallback" },
      labelledBy: { type: "string", empty: "fallback" },
      describedBy: { type: "string", empty: "fallback" }
    });
    def2.props.setDefaults({
      defaultValue: "",
      disabled: false,
      readOnly: false,
      placeholder: "",
      rows: 2,
      required: false,
      name: "",
      autoComplete: "",
      minLength: -1,
      maxLength: -1,
      wrap: "soft",
      ariaLabel: "",
      labelledBy: "",
      describedBy: ""
    });
    const control = asTextControl();
    const focusable = asFocusable();
    focusable.configure({ disabled: false });
    const value = def2.state.string("value", "");
    const disabled = def2.state.bool("disabled", false);
    const readOnly = def2.state.bool("readOnly", false);
    const composing = def2.state.bool("composing", false);
    const ariaLabel = def2.state.string("textareaAriaLabel", "");
    const labelledBy = def2.state.string("textareaLabelledBy", "");
    const describedBy = def2.state.string("textareaDescribedBy", "");
    const focused = focusable.focused;
    const focusVisible = focusable.focusVisible;
    def2.expose.state("value", value);
    def2.expose.state("disabled", disabled);
    def2.expose.state("readOnly", readOnly);
    def2.expose.state("focused", focused);
    def2.expose.state("focusVisible", focusVisible);
    def2.expose.state("composing", composing);
    def2.expose.method("focusSelf", (options) => {
      if (!disabled.get())
        focusable.focusSelf(options);
    });
    def2.expose.method("blurSelf", () => focusable.blur());
    def2.expose.event("valueChange", { payload: "json" });
    def2.expose.event("change", { payload: "json" });
    def2.expose.event("compositionStart", { payload: "json" });
    def2.expose.event("compositionUpdate", { payload: "json" });
    def2.expose.event("compositionEnd", { payload: "json" });
    def2.a11y.role("textbox");
    def2.a11y.name(ariaLabel);
    def2.a11y.state("disabled", disabled);
    def2.a11y.state("readOnly", readOnly);
    def2.a11y.relation("labelledBy", { target: labelledBy });
    def2.a11y.relation("describedBy", { target: describedBy });
    const sync = (props) => {
      const isControlled = typeof props.value === "string";
      const nextDisabled = props.disabled ?? false;
      disabled.set(nextDisabled, "reason: textarea sync disabled");
      readOnly.set(props.readOnly ?? false, "reason: textarea sync readonly");
      ariaLabel.set(props.ariaLabel ?? "", "reason: textarea sync aria label");
      labelledBy.set(props.labelledBy ?? "", "reason: textarea sync labelledby");
      describedBy.set(props.describedBy ?? "", "reason: textarea sync describedby");
      focusable.setDisabled(nextDisabled);
      control.sync({
        valueMode: isControlled ? "controlled" : "uncontrolled",
        value: isControlled ? props.value : undefined,
        defaultValue: props.defaultValue ?? "",
        disabled: nextDisabled,
        readOnly: props.readOnly ?? false,
        placeholder: props.placeholder ?? "",
        rows: props.rows ?? 2,
        required: props.required ?? false,
        name: props.name ?? "",
        autoComplete: props.autoComplete ?? "",
        minLength: props.minLength ?? -1,
        maxLength: props.maxLength ?? -1,
        wrap: props.wrap ?? "soft"
      });
      value.set(control.snapshot()?.value ?? "", "reason: textarea sync value");
    };
    def2.lifecycle.onCreated((run2) => sync(run2.props.get()));
    def2.props.watch([
      "value",
      "defaultValue",
      "disabled",
      "readOnly",
      "placeholder",
      "rows",
      "required",
      "name",
      "autoComplete",
      "minLength",
      "maxLength",
      "wrap",
      "ariaLabel",
      "labelledBy",
      "describedBy"
    ], (_run, next) => sync(next));
    control.on("input", (run2, event2) => {
      value.set(control.snapshot()?.value ?? event2.value, "reason: textarea input value");
      composing.set(event2.composing, "reason: textarea input composing");
      const detail = Object.freeze({
        value: event2.value,
        composing: event2.composing,
        data: event2.data,
        inputType: event2.inputType
      });
      run2.expose.emit("valueChange", detail);
    });
    control.on("change", (run2, event2) => {
      value.set(control.snapshot()?.value ?? event2.value, "reason: textarea change value");
      run2.expose.emit("change", Object.freeze({ value: event2.value }));
    });
    const emitComposition = (run2, eventName, event2) => {
      const detail = Object.freeze({
        value: event2.value,
        data: event2.data
      });
      run2.expose.emit(eventName, detail);
    };
    control.on("compositionstart", (run2, event2) => {
      composing.set(true, "reason: textarea composition start");
      emitComposition(run2, "compositionStart", event2);
    });
    control.on("compositionupdate", (run2, event2) => {
      emitComposition(run2, "compositionUpdate", event2);
    });
    control.on("compositionend", (run2, event2) => {
      composing.set(false, "reason: textarea composition end");
      value.set(control.snapshot()?.value ?? event2.value, "reason: textarea composition end value");
      emitComposition(run2, "compositionEnd", event2);
    });
    return () => null;
  }
  var asTextareaRoot = defineAsHook({
    name: "as-textarea-root",
    modules: [
      declareTextControl({
        content: "plain-text",
        lineMode: "multiline",
        engine: "host"
      })
    ],
    setup: setupTextareaRoot
  });
  var textareaRoot = definePrototype({
    name: "base-textarea-root",
    modules: asTextareaRoot.modules,
    setup: setupTextareaRoot
  });
  // ../packages/prototypes/base/src/button/button.proto.ts
  function setupButton(def2) {
    asTrigger();
    def2.props.define({
      disabled: { type: "boolean", empty: "fallback" }
    });
    def2.props.setDefaults({
      disabled: false
    });
    const disabled = def2.state.bool("disabled", false);
    def2.expose.state("disabled", disabled);
    def2.a11y.state("disabled", disabled);
    const hovered = def2.state.bool("hovered", false);
    def2.expose.state("hovered", hovered);
    const focusable = asFocusable();
    focusable.configure({ disabled: false });
    const focused = focusable.focused;
    const focusVisible = focusable.focusVisible;
    def2.expose.state("focused", focused);
    def2.expose.state("focusVisible", focusVisible);
    def2.expose.method("focusSelf", (options) => {
      if (disabled.get())
        return;
      focusable.focusSelf(options);
    });
    const pressed = def2.state.bool("pressed", false);
    def2.expose.state("pressed", pressed);
    const clearTransientInteraction = (reason) => {
      hovered.set(false, reason);
      pressed.set(false, reason);
    };
    const syncDisabled = (nextDisabled) => {
      disabled.set(nextDisabled, "reason: sync disabled");
      focusable.setDisabled(nextDisabled);
      if (nextDisabled) {
        clearTransientInteraction("reason: button disabled => reset transient interaction");
      }
    };
    def2.lifecycle.onCreated((run2) => {
      syncDisabled(run2.props.get().disabled);
    });
    def2.props.watch(["disabled"], (_run, next) => {
      syncDisabled(next.disabled);
    });
    def2.expose.event("click", { payload: "void" });
    def2.a11y.action("activate", { event: "click" });
    def2.a11y.role("button");
    def2.a11y.nameFromContent();
    def2.event.onGlobal("key.down", (_run, ev) => {
      const detail = ev;
      if (disabled.get())
        return;
      if (!focused.get())
        return;
      if (detail?.key !== " ")
        return;
      ev.control.requestDefaultActionPrevention({
        reason: "button.space-activation",
        source: "base-button"
      });
    });
    def2.event.on("pointer.enter", () => {
      if (disabled.get())
        return;
      hovered.set(true, "reason: button pointer.enter => hovered");
    });
    def2.event.on("pointer.leave", () => {
      hovered.set(false, "reason: button pointer.leave => hovered");
      pressed.set(false, "reason: button pointer.leave => pressed");
    });
    def2.event.on("pointer.cancel", () => {
      hovered.set(false, "reason: button pointer.cancel => hovered");
      pressed.set(false, "reason: button pointer.cancel => pressed");
    });
    def2.event.on("pointer.down", () => {
      if (disabled.get())
        return;
      pressed.set(true, "reason: button pointer.down => pressed");
    });
    def2.event.on("pointer.up", () => {
      pressed.set(false, "reason: button pointer.up => pressed");
    });
    def2.event.on("press.commit", (run2) => {
      pressed.set(false, "reason: button press.commit => pressed");
      if (disabled.get())
        return;
      run2.expose.emit("click");
    });
  }
  var asButton = defineAsHook({
    name: "as-button",
    setup: setupButton
  });
  var button = definePrototype({
    name: "base-button",
    setup: setupButton
  });
  // ../packages/prototypes/shadcn/src/button/button.proto.ts
  var BUTTON_BASE_TOKENS = [
    "group/button",
    "inline-flex",
    "shrink-0",
    "items-center",
    "justify-center",
    "rounded-lg",
    "border",
    "bg-clip-padding",
    "text-sm",
    "font-medium",
    "whitespace-nowrap",
    "transition-all",
    "outline-none",
    "select-none"
  ].join(" ");
  var VARIANT_TOKENS = {
    default: "border-transparent bg-primary text-primary-foreground",
    destructive: "border-transparent bg-destructive/10 text-destructive",
    outline: "border-border bg-background text-foreground",
    secondary: "border-transparent bg-secondary text-secondary-foreground",
    ghost: "border-transparent bg-transparent text-foreground",
    link: "border-transparent bg-transparent text-primary underline-offset-4"
  };
  var SIZE_TOKENS = {
    default: "h-8 gap-1.5 px-2.5",
    sm: "h-7 gap-1 rounded-[min(var(--radius-md),12px)] px-2.5 text-[0.8rem]",
    lg: "h-9 gap-1.5 px-2.5",
    icon: "size-8"
  };
  var button2 = definePrototype({
    name: "shadcn-button",
    setup(def2) {
      def2.props.define({
        variant: {
          type: "enum",
          empty: "fallback",
          options: ["default", "destructive", "outline", "secondary", "ghost", "link"]
        },
        size: { type: "enum", empty: "fallback", options: ["default", "sm", "lg", "icon"] },
        disabled: { type: "boolean", empty: "fallback" }
      });
      def2.props.setDefaults({
        variant: "default",
        size: "default",
        disabled: false
      });
      const buttonState = asButton().stateHandles;
      if (!buttonState) {
        throw new Error("[shadcn-button] asButton must project Button state handles.");
      }
      const { disabled, hovered, focusVisible, pressed } = buttonState;
      def2.feedback.style.use(tw(BUTTON_BASE_TOKENS));
      Object.keys(VARIANT_TOKENS).forEach((variant) => {
        def2.rule({
          when: (w) => w.prop("variant").eq(variant),
          intent: (i) => i.feedback.style.use(tw(VARIANT_TOKENS[variant]))
        });
      });
      Object.keys(SIZE_TOKENS).forEach((size) => {
        def2.rule({
          when: (w) => w.prop("size").eq(size),
          intent: (i) => i.feedback.style.use(tw(SIZE_TOKENS[size]))
        });
      });
      def2.rule({
        when: (w) => w.state(focusVisible).eq(true),
        intent: (i) => i.feedback.style.use(tw("border-ring ring-3 ring-ring/50"))
      });
      def2.rule({
        when: (w) => w.all(w.state(focusVisible).eq(true), w.prop("variant").eq("destructive")),
        intent: (i) => i.feedback.style.use(tw("border-destructive/40 ring-destructive/20"))
      });
      def2.rule({
        when: (w) => w.state(pressed).eq(true),
        intent: (i) => i.feedback.style.use(tw("translate-y-px"))
      });
      def2.rule({
        when: (w) => w.all(w.state(hovered).eq(true), w.prop("variant").eq("default")),
        intent: (i) => i.feedback.style.use(tw("bg-primary/80"))
      });
      def2.rule({
        when: (w) => w.all(w.state(hovered).eq(true), w.prop("variant").eq("secondary")),
        intent: (i) => i.feedback.style.use(tw("bg-secondary/80"))
      });
      def2.rule({
        when: (w) => w.all(w.state(hovered).eq(true), w.prop("variant").eq("outline")),
        intent: (i) => i.feedback.style.use(tw("bg-muted text-foreground"))
      });
      def2.rule({
        when: (w) => w.all(w.state(hovered).eq(true), w.prop("variant").eq("ghost")),
        intent: (i) => i.feedback.style.use(tw("bg-muted text-foreground"))
      });
      def2.rule({
        when: (w) => w.all(w.state(hovered).eq(true), w.prop("variant").eq("link")),
        intent: (i) => i.feedback.style.use(tw("underline"))
      });
      def2.rule({
        when: (w) => w.all(w.state(hovered).eq(true), w.prop("variant").eq("destructive")),
        intent: (i) => i.feedback.style.use(tw("bg-destructive/20"))
      });
      def2.rule({
        when: (w) => w.all(w.meta("colorScheme").eq("dark"), w.prop("variant").eq("outline")),
        intent: (i) => i.feedback.style.use(tw("border-input bg-input/30"))
      });
      def2.rule({
        when: (w) => w.all(w.meta("colorScheme").eq("dark"), w.state(hovered).eq(true), w.prop("variant").eq("outline")),
        intent: (i) => i.feedback.style.use(tw("bg-input/50"))
      });
      def2.rule({
        when: (w) => w.all(w.meta("colorScheme").eq("dark"), w.prop("variant").eq("destructive")),
        intent: (i) => i.feedback.style.use(tw("bg-destructive/20"))
      });
      def2.rule({
        when: (w) => w.all(w.meta("colorScheme").eq("dark"), w.state(hovered).eq(true), w.prop("variant").eq("destructive")),
        intent: (i) => i.feedback.style.use(tw("bg-destructive/30"))
      });
      def2.rule({
        when: (w) => w.all(w.meta("colorScheme").eq("dark"), w.state(focusVisible).eq(true), w.prop("variant").eq("destructive")),
        intent: (i) => i.feedback.style.use(tw("ring-destructive/40"))
      });
      def2.rule({
        when: (w) => w.state(disabled).eq(true),
        intent: (i) => i.feedback.style.use(tw("pointer-events-none opacity-50"))
      });
    }
  });
  var button_proto_default = button2;
  // ../packages/prototypes/base/src/toggle/toggle.proto.ts
  function setupToggle(def2) {
    asTrigger();
    def2.props.define({
      active: { type: "boolean", empty: "fallback" },
      defaultActive: { type: "boolean", empty: "fallback" },
      disabled: { type: "boolean", empty: "fallback" }
    });
    def2.props.setDefaults({
      defaultActive: false,
      disabled: false
    });
    const active = def2.state.bool("active", false);
    const disabled = def2.state.bool("disabled", false);
    const hovered = def2.state.bool("hovered", false);
    const pressed = def2.state.bool("pressed", false);
    const focusable = asFocusable();
    focusable.configure({ disabled: false });
    const focused = focusable.focused;
    const focusVisible = focusable.focusVisible;
    def2.expose.state("active", active);
    def2.expose.state("disabled", disabled);
    def2.expose.state("hovered", hovered);
    def2.expose.state("focused", focused);
    def2.expose.state("focusVisible", focusVisible);
    def2.expose.state("pressed", pressed);
    def2.expose.method("focusSelf", (options) => {
      if (disabled.get())
        return;
      focusable.focusSelf(options);
    });
    def2.expose.event("activeChange", { payload: "json" });
    def2.a11y.role("button");
    def2.a11y.nameFromContent();
    def2.a11y.state("pressed", active);
    def2.a11y.state("disabled", disabled);
    def2.a11y.action("activate", { event: "activeChange" });
    let controlled = false;
    const clearTransientInteraction = (reason) => {
      hovered.set(false, reason);
      pressed.set(false, reason);
    };
    const syncDisabled = (nextDisabled) => {
      disabled.set(nextDisabled, "reason: toggle sync disabled");
      focusable.setDisabled(nextDisabled);
      if (nextDisabled) {
        clearTransientInteraction("reason: toggle disabled => reset transient interaction");
      }
    };
    def2.lifecycle.onCreated((run2) => {
      controlled = run2.props.isProvided("active");
      active.set(controlled ? !!run2.props.get().active : !!run2.props.get().defaultActive, "reason: toggle initialize active");
      syncDisabled(!!run2.props.get().disabled);
    });
    def2.props.watch(["active"], (run2, next) => {
      controlled = run2.props.isProvided("active");
      if (!controlled)
        return;
      active.set(!!next.active, "reason: toggle controlled active sync");
    });
    def2.props.watch(["disabled"], (_run, next) => {
      syncDisabled(!!next.disabled);
    });
    def2.event.onGlobal("key.down", (_run, ev) => {
      const detail = ev;
      if (disabled.get())
        return;
      if (!focused.get())
        return;
      if (detail?.key !== " ")
        return;
      ev.control.requestDefaultActionPrevention({
        reason: "toggle.space-activation",
        source: "base-toggle"
      });
    });
    def2.event.on("pointer.enter", () => {
      if (disabled.get())
        return;
      hovered.set(true, "reason: toggle pointer.enter => hovered");
    });
    def2.event.on("pointer.leave", () => {
      hovered.set(false, "reason: toggle pointer.leave => hovered");
      pressed.set(false, "reason: toggle pointer.leave => pressed");
    });
    def2.event.on("pointer.cancel", () => {
      hovered.set(false, "reason: toggle pointer.cancel => hovered");
      pressed.set(false, "reason: toggle pointer.cancel => pressed");
    });
    def2.event.on("pointer.down", () => {
      if (disabled.get())
        return;
      pressed.set(true, "reason: toggle pointer.down => pressed");
    });
    def2.event.on("pointer.up", () => {
      pressed.set(false, "reason: toggle pointer.up => pressed");
    });
    def2.event.on("press.commit", (run2) => {
      pressed.set(false, "reason: toggle press.commit => pressed");
      if (disabled.get())
        return;
      const nextActive = !active.get();
      if (!controlled) {
        active.set(nextActive, "reason: toggle press.commit => active");
      }
      run2.expose.emit("activeChange", { active: nextActive });
    });
  }
  var asToggle = defineAsHook({
    name: "as-toggle",
    setup: setupToggle
  });
  var toggle = definePrototype({
    name: "base-toggle",
    setup: setupToggle
  });
  // ../packages/prototypes/shadcn/src/toggle/toggle.proto.ts
  var TOGGLE_BASE_TOKENS = [
    "group/toggle",
    "inline-flex",
    "items-center",
    "justify-center",
    "gap-1",
    "rounded-lg",
    "text-sm",
    "font-medium",
    "transition-all",
    "outline-none",
    "border",
    "whitespace-nowrap"
  ].join(" ");
  var VARIANT_TOKENS2 = {
    default: "border-transparent bg-transparent text-foreground",
    outline: "border-input bg-transparent text-foreground"
  };
  var SIZE_TOKENS2 = {
    default: "h-8 min-w-8 px-2.5",
    sm: "h-7 min-w-7 px-2",
    lg: "h-9 min-w-9 px-3"
  };
  var toggle2 = definePrototype({
    name: "shadcn-toggle",
    setup(def2) {
      def2.props.define({
        variant: { type: "enum", empty: "fallback", options: ["default", "outline"] },
        size: { type: "enum", empty: "fallback", options: ["default", "sm", "lg"] },
        active: { type: "boolean", empty: "fallback" },
        defaultActive: { type: "boolean", empty: "fallback" },
        disabled: { type: "boolean", empty: "fallback" }
      });
      def2.props.setDefaults({
        variant: "default",
        size: "default",
        defaultActive: false,
        disabled: false
      });
      const toggleState = asToggle().stateHandles;
      if (!toggleState) {
        throw new Error("[shadcn-toggle] asToggle must project Toggle state handles.");
      }
      const { active, disabled, hovered, focusVisible } = toggleState;
      def2.feedback.style.use(tw(TOGGLE_BASE_TOKENS));
      Object.keys(VARIANT_TOKENS2).forEach((variant) => {
        def2.rule({
          when: (w) => w.prop("variant").eq(variant),
          intent: (i) => i.feedback.style.use(tw(VARIANT_TOKENS2[variant]))
        });
      });
      Object.keys(SIZE_TOKENS2).forEach((size) => {
        def2.rule({
          when: (w) => w.prop("size").eq(size),
          intent: (i) => i.feedback.style.use(tw(SIZE_TOKENS2[size]))
        });
      });
      def2.rule({
        when: (w) => w.state(active).eq(true),
        intent: (i) => i.feedback.style.use(tw("bg-muted"))
      });
      def2.rule({
        when: (w) => w.all(w.state(hovered).eq(true), w.state(active).eq(false)),
        intent: (i) => i.feedback.style.use(tw("bg-muted text-foreground"))
      });
      def2.rule({
        when: (w) => w.state(focusVisible).eq(true),
        intent: (i) => i.feedback.style.use(tw("border-ring ring-3 ring-ring/50"))
      });
      def2.rule({
        when: (w) => w.state(disabled).eq(true),
        intent: (i) => i.feedback.style.use(tw("pointer-events-none opacity-50"))
      });
    }
  });
  var toggle_proto_default = toggle2;
  // ../packages/prototypes/base/src/checkbox/shared.ts
  var CHECKBOX_FAMILY = createAnatomyFamily("base-checkbox", {
    roles: {
      root: { cardinality: { min: 1, max: 1 } },
      indicator: { cardinality: { min: 0, max: "*" } }
    },
    relations: [{ kind: "contains", parent: "root", child: "indicator" }]
  });
  var CHECKBOX_CONTEXT = createContextKey("base-checkbox");

  // ../packages/prototypes/base/src/checkbox/root.proto.ts
  function isEnterKeyboardCommit(ev) {
    return ev?.key === "Enter";
  }
  function setupCheckboxRoot(def2) {
    def2.anatomy.claim(CHECKBOX_FAMILY, { role: "root" });
    asTrigger();
    def2.props.define({
      checked: { type: "boolean", empty: "fallback" },
      defaultChecked: { type: "boolean", empty: "fallback" },
      disabled: { type: "boolean", empty: "fallback" },
      indeterminate: { type: "boolean", empty: "fallback" },
      defaultIndeterminate: { type: "boolean", empty: "fallback" }
    });
    def2.props.setDefaults({
      defaultChecked: false,
      disabled: false,
      defaultIndeterminate: false
    });
    const checked = def2.state.bool("checked", false);
    const disabled = def2.state.bool("disabled", false);
    const hovered = def2.state.bool("hovered", false);
    const pressed = def2.state.bool("pressed", false);
    const indeterminate = def2.state.bool("indeterminate", false);
    const checkedA11y = def2.state.string("checkedA11y", "false", {
      options: ["true", "false", "mixed"]
    });
    const focusable = asFocusable();
    focusable.configure({ disabled: false });
    const focused = focusable.focused;
    const focusVisible = focusable.focusVisible;
    def2.expose.state("checked", checked);
    def2.expose.state("indeterminate", indeterminate);
    def2.expose.state("disabled", disabled);
    def2.expose.state("hovered", hovered);
    def2.expose.state("focused", focused);
    def2.expose.state("focusVisible", focusVisible);
    def2.expose.state("pressed", pressed);
    def2.expose.method("focusSelf", (options) => {
      if (disabled.get())
        return;
      focusable.focusSelf(options);
    });
    def2.expose.event("checkedChange", { payload: "json" });
    def2.expose.event("indeterminateChange", { payload: "json" });
    def2.a11y.role("checkbox");
    def2.a11y.nameFromContent();
    def2.a11y.state("checked", checkedA11y);
    def2.a11y.state("disabled", disabled);
    def2.a11y.action("activate", { event: "checkedChange" });
    let controlledChecked = false;
    let controlledIndeterminate = false;
    def2.context.provide(CHECKBOX_CONTEXT, {
      checked: false,
      indeterminate: false,
      disabled: false
    });
    const publishContext = (run2) => {
      checkedA11y.set(indeterminate.get() ? "mixed" : checked.get() ? "true" : "false", "reason: checkbox root sync a11y checked");
      run2.context.update(CHECKBOX_CONTEXT, {
        checked: !!checked.get(),
        indeterminate: !!indeterminate.get(),
        disabled: !!disabled.get()
      });
    };
    const emitCheckedChange = (run2, detail) => {
      run2.expose.emit("checkedChange", detail);
    };
    const clearTransientInteraction = (reason) => {
      hovered.set(false, reason);
      pressed.set(false, reason);
    };
    const syncDisabled = (run2, nextDisabled) => {
      disabled.set(nextDisabled, "reason: checkbox root sync disabled");
      focusable.setDisabled(nextDisabled);
      if (nextDisabled) {
        clearTransientInteraction("reason: checkbox root disabled => reset transient interaction");
      }
      publishContext(run2);
    };
    def2.lifecycle.onCreated((run2) => {
      controlledChecked = run2.props.isProvided("checked");
      controlledIndeterminate = run2.props.isProvided("indeterminate");
      checked.set(controlledChecked ? !!run2.props.get().checked : !!run2.props.get().defaultChecked, "reason: checkbox root initialize checked");
      indeterminate.set(controlledIndeterminate ? !!run2.props.get().indeterminate : !!run2.props.get().defaultIndeterminate, "reason: checkbox root initialize indeterminate");
      syncDisabled(run2, !!run2.props.get().disabled);
    });
    def2.lifecycle.onMounted((run2) => {
      publishContext(run2);
    });
    def2.props.watch(["checked"], (run2, next) => {
      controlledChecked = run2.props.isProvided("checked");
      if (!controlledChecked)
        return;
      checked.set(!!next.checked, "reason: checkbox root controlled checked sync");
      publishContext(run2);
    });
    def2.props.watch(["indeterminate"], (run2, next) => {
      controlledIndeterminate = run2.props.isProvided("indeterminate");
      if (controlledIndeterminate) {
        indeterminate.set(!!next.indeterminate, "reason: checkbox root controlled indeterminate sync");
      }
      publishContext(run2);
    });
    def2.props.watch(["disabled"], (run2, next) => {
      syncDisabled(run2, !!next.disabled);
    });
    def2.event.onGlobal("key.down", (_run, ev) => {
      const detail = ev;
      if (disabled.get())
        return;
      if (!focused.get())
        return;
      if (detail?.key !== " ")
        return;
      ev.control.requestDefaultActionPrevention({
        reason: "checkbox.space-activation",
        source: "base-checkbox"
      });
    });
    def2.event.on("pointer.enter", () => {
      if (disabled.get())
        return;
      hovered.set(true, "reason: checkbox root pointer.enter => hovered");
    });
    def2.event.on("pointer.leave", () => {
      hovered.set(false, "reason: checkbox root pointer.leave => hovered");
      pressed.set(false, "reason: checkbox root pointer.leave => pressed");
    });
    def2.event.on("pointer.cancel", () => {
      hovered.set(false, "reason: checkbox root pointer.cancel => hovered");
      pressed.set(false, "reason: checkbox root pointer.cancel => pressed");
    });
    def2.event.on("pointer.down", () => {
      if (disabled.get())
        return;
      pressed.set(true, "reason: checkbox root pointer.down => pressed");
    });
    def2.event.on("pointer.up", () => {
      pressed.set(false, "reason: checkbox root pointer.up => pressed");
    });
    def2.event.on("press.commit", (run2, ev) => {
      pressed.set(false, "reason: checkbox root press.commit => pressed");
      if (disabled.get())
        return;
      if (isEnterKeyboardCommit(ev))
        return;
      const wasIndeterminate = indeterminate.get();
      if (wasIndeterminate) {
        if (!controlledIndeterminate) {
          indeterminate.set(false, "reason: press.commit => clear indeterminate");
        }
        run2.expose.emit("indeterminateChange", { indeterminate: false });
      }
      const nextChecked = !checked.get();
      const nextIndeterminate = wasIndeterminate ? false : indeterminate.get();
      if (controlledChecked) {
        emitCheckedChange(run2, { checked: nextChecked, indeterminate: nextIndeterminate });
        publishContext(run2);
        return;
      }
      checked.set(nextChecked, "reason: press.commit => toggle checked");
      emitCheckedChange(run2, { checked: nextChecked, indeterminate: nextIndeterminate });
      publishContext(run2);
    });
  }
  var asCheckboxRoot = defineAsHook({
    name: "as-checkbox-root",
    setup: setupCheckboxRoot
  });
  var checkboxRoot = definePrototype({
    name: "base-checkbox-root",
    setup: setupCheckboxRoot
  });
  // ../packages/prototypes/base/src/checkbox/indicator.proto.ts
  function setupCheckboxIndicator(def2) {
    def2.anatomy.claim(CHECKBOX_FAMILY, { role: "indicator" });
    const checked = def2.state.bool("checked", false);
    const indeterminate = def2.state.bool("indeterminate", false);
    const disabled = def2.state.bool("disabled", false);
    const syncContext = (next) => {
      checked.set(!!next.checked, "reason: checkbox indicator context checked sync");
      indeterminate.set(!!next.indeterminate, "reason: checkbox indicator context indeterminate sync");
      disabled.set(!!next.disabled, "reason: checkbox indicator context disabled sync");
    };
    def2.expose.state("checked", checked);
    def2.expose.state("indeterminate", indeterminate);
    def2.expose.method("isChecked", () => {
      return checked.get();
    });
    def2.expose.method("isIndeterminate", () => {
      return indeterminate.get();
    });
    def2.context.subscribe(CHECKBOX_CONTEXT, (_run, next) => {
      syncContext(next);
    });
    def2.lifecycle.onMounted((run2) => {
      syncContext(run2.context.read(CHECKBOX_CONTEXT));
    });
    def2.lifecycle.onUpdated((run2) => {
      syncContext(run2.context.read(CHECKBOX_CONTEXT));
    });
  }
  var asCheckboxIndicator = defineAsHook({
    name: "as-checkbox-indicator",
    setup: setupCheckboxIndicator
  });
  var checkboxIndicator = definePrototype({
    name: "base-checkbox-indicator",
    setup: setupCheckboxIndicator
  });
  // ../packages/prototypes/shadcn/src/checkbox/indicator.proto.ts
  var INDICATOR_TOKENS = [
    "flex",
    "size-3.5",
    "items-center",
    "justify-center",
    "transition-none"
  ].join(" ");
  function glyphPath(checked, indeterminate) {
    if (indeterminate)
      return "M5 12h14";
    if (checked)
      return "m20 6-11 11-5-5";
    return null;
  }
  function renderGlyph(renderer, d) {
    if (!d)
      return null;
    return renderer.svg.root({
      viewBox: "0 0 24 24",
      "aria-hidden": "true",
      width: "100%",
      height: "100%",
      fill: "none",
      stroke: "currentColor",
      strokeWidth: 2,
      strokeLinecap: "round",
      strokeLinejoin: "round"
    }, renderer.svg.path({ d }));
  }
  var checkboxIndicator2 = definePrototype({
    name: "shadcn-checkbox-indicator",
    setup(def2) {
      const indicatorState = asCheckboxIndicator().stateHandles;
      if (!indicatorState) {
        throw new Error("[shadcn-checkbox-indicator] asCheckboxIndicator must project Checkbox indicator state handles.");
      }
      const { checked, indeterminate } = indicatorState;
      def2.feedback.style.use(tw(INDICATOR_TOKENS));
      let renderTask = null;
      const requestGlyphUpdate = (run2, event2) => {
        if (event2.type !== "next")
          return;
        renderTask?.cancel();
        renderTask = delay(0, () => {
          renderTask = null;
          run2.update();
        });
      };
      checked.watch(requestGlyphUpdate);
      indeterminate.watch(requestGlyphUpdate);
      def2.lifecycle.onUnmounted(() => {
        renderTask?.cancel();
        renderTask = null;
      });
      return (renderer) => [
        renderer.r.slot(),
        renderGlyph(renderer, glyphPath(checked.get(), indeterminate.get()))
      ];
    }
  });
  var indicator_proto_default2 = checkboxIndicator2;

  // ../packages/prototypes/shadcn/src/checkbox/root.proto.ts
  var ROOT_BASE_TOKENS = [
    "size-4",
    "shrink-0",
    "rounded-[4px]",
    "border",
    "border-input",
    "bg-transparent",
    "shadow-xs",
    "outline-none"
  ].join(" ");
  var checkboxRoot2 = definePrototype({
    name: "shadcn-checkbox-root",
    setup(def2) {
      const checkboxState = asCheckboxRoot().stateHandles;
      if (!checkboxState) {
        throw new Error("[shadcn-checkbox-root] asCheckboxRoot must project Checkbox root state handles.");
      }
      const { checked, indeterminate, disabled, focusVisible } = checkboxState;
      def2.feedback.style.use(tw(ROOT_BASE_TOKENS));
      def2.rule({
        when: (w) => w.state(checked).eq(true),
        intent: (i) => i.feedback.style.use(tw("bg-primary text-primary-foreground border-primary"))
      });
      def2.rule({
        when: (w) => w.state(indeterminate).eq(true),
        intent: (i) => i.feedback.style.use(tw("bg-primary text-primary-foreground border-primary"))
      });
      def2.rule({
        when: (w) => w.state(focusVisible).eq(true),
        intent: (i) => i.feedback.style.use(tw("border-ring ring-ring/50 ring-3"))
      });
      def2.rule({
        when: (w) => w.state(disabled).eq(true),
        intent: (i) => i.feedback.style.use(tw("cursor-not-allowed opacity-50"))
      });
      def2.rule({
        when: (w) => w.all(w.meta("colorScheme").eq("dark"), w.state(checked).eq(false), w.state(indeterminate).eq(false)),
        intent: (i) => i.feedback.style.use(tw("bg-input/30"))
      });
    }
  });
  var root_proto_default3 = checkboxRoot2;
  // ../packages/prototypes/base/src/switch/shared.ts
  var SWITCH_FAMILY = createAnatomyFamily("base-switch", {
    roles: {
      root: { cardinality: { min: 1, max: 1 } },
      thumb: { cardinality: { min: 0, max: "*" } }
    },
    relations: [{ kind: "contains", parent: "root", child: "thumb" }]
  });
  var SWITCH_CONTEXT = createContextKey("base-switch");

  // ../packages/prototypes/base/src/switch/root.proto.ts
  function setupSwitchRoot(def2) {
    def2.anatomy.claim(SWITCH_FAMILY, { role: "root" });
    asTrigger();
    def2.props.define({
      checked: { type: "boolean", empty: "fallback" },
      defaultChecked: { type: "boolean", empty: "fallback" },
      disabled: { type: "boolean", empty: "fallback" }
    });
    def2.props.setDefaults({
      defaultChecked: false,
      disabled: false
    });
    const checked = def2.state.bool("checked", false);
    const disabled = def2.state.bool("disabled", false);
    const hovered = def2.state.bool("hovered", false);
    const pressed = def2.state.bool("pressed", false);
    const focusable = asFocusable();
    focusable.configure({ disabled: false });
    const focused = focusable.focused;
    const focusVisible = focusable.focusVisible;
    def2.expose.state("checked", checked);
    def2.expose.state("disabled", disabled);
    def2.expose.state("hovered", hovered);
    def2.expose.state("focused", focused);
    def2.expose.state("focusVisible", focusVisible);
    def2.expose.state("pressed", pressed);
    def2.expose.method("focusSelf", (options) => {
      if (disabled.get())
        return;
      focusable.focusSelf(options);
    });
    def2.expose.event("checkedChange", { payload: "json" });
    def2.a11y.role("switch");
    def2.a11y.nameFromContent();
    def2.a11y.state("checked", checked);
    def2.a11y.state("disabled", disabled);
    def2.a11y.action("activate", { event: "checkedChange" });
    def2.context.provide(SWITCH_CONTEXT, {
      checked: false,
      disabled: false
    });
    let controlled = false;
    const publishContext = (run2) => {
      run2.context.update(SWITCH_CONTEXT, {
        checked: !!checked.get(),
        disabled: !!disabled.get()
      });
    };
    const clearTransientInteraction = (reason) => {
      hovered.set(false, reason);
      pressed.set(false, reason);
    };
    const syncDisabled = (run2, nextDisabled) => {
      disabled.set(nextDisabled, "reason: switch root sync disabled");
      focusable.setDisabled(nextDisabled);
      if (nextDisabled) {
        clearTransientInteraction("reason: switch root disabled => reset transient interaction");
      }
      publishContext(run2);
    };
    def2.lifecycle.onCreated((run2) => {
      controlled = run2.props.isProvided("checked");
      checked.set(controlled ? !!run2.props.get().checked : !!run2.props.get().defaultChecked, "reason: switch root initialize checked");
      syncDisabled(run2, !!run2.props.get().disabled);
      publishContext(run2);
    });
    def2.lifecycle.onMounted((run2) => {
      publishContext(run2);
    });
    def2.props.watch(["checked"], (run2, next) => {
      controlled = run2.props.isProvided("checked");
      if (!controlled)
        return;
      checked.set(!!next.checked, "reason: switch root controlled checked sync");
      publishContext(run2);
    });
    def2.props.watch(["disabled"], (run2, next) => {
      syncDisabled(run2, !!next.disabled);
    });
    def2.event.onGlobal("key.down", (_run, ev) => {
      const detail = ev;
      if (disabled.get())
        return;
      if (!focused.get())
        return;
      if (detail?.key !== " ")
        return;
      ev.control.requestDefaultActionPrevention({
        reason: "switch.space-activation",
        source: "base-switch"
      });
    });
    def2.event.on("pointer.enter", () => {
      if (disabled.get())
        return;
      hovered.set(true, "reason: switch root pointer.enter => hovered");
    });
    def2.event.on("pointer.leave", () => {
      hovered.set(false, "reason: switch root pointer.leave => hovered");
      pressed.set(false, "reason: switch root pointer.leave => pressed");
    });
    def2.event.on("pointer.cancel", () => {
      hovered.set(false, "reason: switch root pointer.cancel => hovered");
      pressed.set(false, "reason: switch root pointer.cancel => pressed");
    });
    def2.event.on("pointer.down", () => {
      if (disabled.get())
        return;
      pressed.set(true, "reason: switch root pointer.down => pressed");
    });
    def2.event.on("pointer.up", () => {
      pressed.set(false, "reason: switch root pointer.up => pressed");
    });
    def2.event.on("press.commit", (run2) => {
      pressed.set(false, "reason: switch root press.commit => pressed");
      if (disabled.get())
        return;
      const nextChecked = !checked.get();
      if (!controlled) {
        checked.set(nextChecked, "reason: switch root press.commit => checked");
      }
      run2.expose.emit("checkedChange", { checked: nextChecked });
      publishContext(run2);
    });
  }
  var asSwitchRoot = defineAsHook({
    name: "as-switch-root",
    setup: setupSwitchRoot
  });
  var switchRoot = definePrototype({
    name: "base-switch-root",
    setup: setupSwitchRoot
  });
  // ../packages/prototypes/base/src/switch/thumb.proto.ts
  function setupSwitchThumb(def2) {
    def2.anatomy.claim(SWITCH_FAMILY, { role: "thumb" });
    const checked = def2.state.bool("checked", false);
    const disabled = def2.state.bool("disabled", false);
    def2.expose.state("checked", checked);
    def2.expose.method("isChecked", () => {
      return checked.get();
    });
    const syncContext = (next) => {
      checked.set(!!next.checked, "reason: switch thumb context checked sync");
      disabled.set(!!next.disabled, "reason: switch thumb context disabled sync");
    };
    def2.context.subscribe(SWITCH_CONTEXT, (_run, next) => {
      syncContext(next);
    });
    def2.lifecycle.onMounted((run2) => {
      syncContext(run2.context.read(SWITCH_CONTEXT));
    });
    def2.lifecycle.onUpdated((run2) => {
      syncContext(run2.context.read(SWITCH_CONTEXT));
    });
  }
  var asSwitchThumb = defineAsHook({
    name: "as-switch-thumb",
    setup: setupSwitchThumb
  });
  var switchThumb = definePrototype({
    name: "base-switch-thumb",
    setup: setupSwitchThumb
  });
  // ../packages/prototypes/shadcn/src/switch/root.proto.ts
  var ROOT_BASE_TOKENS2 = [
    "peer",
    "inline-flex",
    "h-6",
    "w-11",
    "shrink-0",
    "items-center",
    "rounded-full",
    "border",
    "border-transparent",
    "px-0.5",
    "shadow-xs",
    "transition-all",
    "duration-200",
    "ease-in-out",
    "outline-none",
    "bg-input/80",
    "select-none"
  ].join(" ");
  var switchRoot2 = definePrototype({
    name: "shadcn-switch-root",
    setup(def2) {
      const switchState = asSwitchRoot().stateHandles;
      if (!switchState) {
        throw new Error("[shadcn-switch-root] asSwitchRoot must project Switch root state handles.");
      }
      const { checked, disabled, hovered, focusVisible, pressed } = switchState;
      def2.feedback.style.use(tw(ROOT_BASE_TOKENS2));
      def2.rule({
        when: (w) => w.state(checked).eq(true),
        intent: (i) => i.feedback.style.use(tw("bg-primary text-primary-foreground"))
      });
      def2.rule({
        when: (w) => w.all(w.state(checked).eq(false), w.state(hovered).eq(true)),
        intent: (i) => i.feedback.style.use(tw("bg-input"))
      });
      def2.rule({
        when: (w) => w.all(w.state(checked).eq(true), w.state(hovered).eq(true)),
        intent: (i) => i.feedback.style.use(tw("bg-primary/90"))
      });
      def2.rule({
        when: (w) => w.state(pressed).eq(true),
        intent: (i) => i.feedback.style.use(tw("scale-[0.98]"))
      });
      def2.rule({
        when: (w) => w.state(focusVisible).eq(true),
        intent: (i) => i.feedback.style.use(tw("ring-3 ring-ring/50 ring-offset-2 ring-offset-background"))
      });
      def2.rule({
        when: (w) => w.state(disabled).eq(true),
        intent: (i) => i.feedback.style.use(tw("pointer-events-none opacity-50"))
      });
      def2.rule({
        when: (w) => w.all(w.meta("colorScheme").eq("dark"), w.state(checked).eq(false)),
        intent: (i) => i.feedback.style.use(tw("bg-input/50"))
      });
      def2.rule({
        when: (w) => w.all(w.meta("colorScheme").eq("dark"), w.state(checked).eq(true)),
        intent: (i) => i.feedback.style.use(tw("bg-primary"))
      });
    }
  });
  var root_proto_default5 = switchRoot2;

  // ../packages/prototypes/shadcn/src/switch/thumb.proto.ts
  var THUMB_TOKENS = [
    "pointer-events-none",
    "block",
    "size-5",
    "rounded-full",
    "bg-background",
    "border",
    "border-border/50",
    "shadow-lg",
    "ring-0",
    "ring-offset-0",
    "transition-all",
    "duration-200",
    "ease-in-out",
    "will-change-transform",
    "translate-x-0"
  ].join(" ");
  var switchThumb2 = definePrototype({
    name: "shadcn-switch-thumb",
    setup(def2) {
      const switchState = asSwitchThumb().stateHandles;
      if (!switchState) {
        throw new Error("[shadcn-switch-thumb] asSwitchThumb must project Switch thumb state handles.");
      }
      const { checked } = switchState;
      def2.feedback.style.use(tw(THUMB_TOKENS));
      def2.rule({
        when: (w) => w.state(checked).eq(true),
        intent: (i) => i.feedback.style.use(tw("translate-x-[calc(100%_-_2px)]"))
      });
    }
  });
  var thumb_proto_default2 = switchThumb2;
  // ../packages/prototypes/base/src/tabs/shared.ts
  var nextTabsRootId = 0;
  function createTabsRootId() {
    nextTabsRootId += 1;
    return `pui-tabs-${nextTabsRootId}`;
  }
  function createTabsPartId(rootId, role, value) {
    const normalizedValue = value.trim() ? value.trim() : "empty";
    const safeValue = normalizedValue.replace(/[^a-zA-Z0-9_-]+/g, "-");
    return `${rootId || "pui-tabs"}-${role}-${safeValue}`;
  }
  var TABS_FAMILY = createAnatomyFamily("base-tabs", {
    roles: {
      root: { cardinality: { min: 1, max: 1 } },
      list: { cardinality: { min: 0, max: 1 } },
      trigger: { cardinality: { min: 0, max: 100 } },
      content: { cardinality: { min: 0, max: 100 } },
      indicator: { cardinality: { min: 0, max: "*" } }
    },
    relations: [
      { kind: "contains", parent: "root", child: "list" },
      { kind: "contains", parent: "list", child: "trigger" },
      { kind: "contains", parent: "root", child: "content" },
      { kind: "contains", parent: "root", child: "indicator" }
    ]
  });
  var TABS_CONTEXT = createContextKey("base-tabs");

  // ../packages/prototypes/base/src/tabs/root.proto.ts
  function readTriggerSnapshot(part) {
    const exposed = part.getExpose("__collectionItem");
    const disabledExpose = part.getExpose("disabled");
    const disabled = disabledExpose && typeof disabledExpose === "object" && typeof disabledExpose.get === "function" ? disabledExpose.get() : undefined;
    const withLiveDisabled = (snapshot) => typeof disabled === "undefined" ? snapshot : { ...snapshot, disabled };
    if (typeof exposed === "function") {
      const next = exposed();
      return next && typeof next === "object" ? withLiveDisabled(next) : {};
    }
    return exposed && typeof exposed === "object" ? withLiveDisabled(exposed) : {};
  }
  function getEnabledTriggerValues(run2) {
    return run2.anatomy.order.partsOf(TABS_FAMILY, "trigger").map(readTriggerSnapshot).filter((snapshot) => typeof snapshot.value === "string" && snapshot.value).filter((snapshot) => snapshot.disabled !== true).map((snapshot) => snapshot.value);
  }
  function hasEnabledTriggerValue(run2, target) {
    return getEnabledTriggerValues(run2).includes(target);
  }
  function setupTabsRoot(def2) {
    def2.anatomy.claim(TABS_FAMILY, { role: "root" });
    const collection2 = asCollection();
    collection2.configure({ family: TABS_FAMILY, itemRole: "trigger" });
    def2.props.define({
      value: { type: "string", empty: "fallback" },
      defaultValue: { type: "string", empty: "fallback" },
      orientation: { type: "enum", empty: "fallback", options: ["horizontal", "vertical"] },
      activationMode: { type: "enum", empty: "fallback", options: ["automatic", "manual"] }
    });
    def2.props.setDefaults({
      defaultValue: "",
      orientation: "horizontal",
      activationMode: "automatic"
    });
    def2.context.provide(TABS_CONTEXT, {
      rootId: "",
      value: "",
      activeValue: "",
      orientation: "horizontal",
      activationMode: "automatic",
      controlled: false,
      requestedValue: "",
      requestVersion: 0,
      validationVersion: 0
    });
    const value = def2.state.string("value", "");
    const rootId = createTabsRootId();
    let currentOrientation = "horizontal";
    let currentActivationMode = "automatic";
    let controlled = false;
    let activeValue = "";
    let lastRequestVersion = 0;
    let lastValidationVersion = 0;
    def2.expose.state("value", value);
    def2.expose.event("valueChange", { payload: "json" });
    const resolveSelection = (run2, candidate) => {
      if (candidate && hasEnabledTriggerValue(run2, candidate))
        return candidate;
      const fallback = getEnabledTriggerValues(run2)[0];
      return fallback ?? candidate ?? "";
    };
    const publishContext = (run2) => {
      const next = {
        rootId,
        value: value.get(),
        activeValue,
        orientation: currentOrientation,
        activationMode: currentActivationMode,
        controlled,
        requestedValue: "",
        requestVersion: lastRequestVersion,
        validationVersion: lastValidationVersion
      };
      const current = run2.context.read(TABS_CONTEXT);
      if (current.rootId === next.rootId && current.value === next.value && current.activeValue === next.activeValue && current.orientation === next.orientation && current.activationMode === next.activationMode && current.controlled === next.controlled && current.requestedValue === next.requestedValue && current.requestVersion === next.requestVersion && current.validationVersion === next.validationVersion) {
        return;
      }
      run2.context.update(TABS_CONTEXT, next);
    };
    const validateSelection = (run2, reason) => {
      const currentValue = value.get();
      const nextValue = controlled ? currentValue : resolveSelection(run2, currentValue);
      if (!controlled && nextValue !== currentValue) {
        value.set(nextValue, reason);
      }
      if (!activeValue || !hasEnabledTriggerValue(run2, activeValue)) {
        activeValue = nextValue;
      }
      publishContext(run2);
    };
    def2.context.subscribe(TABS_CONTEXT, (run2, next) => {
      activeValue = next.activeValue ?? "";
      if (next.requestVersion !== lastRequestVersion) {
        lastRequestVersion = next.requestVersion;
        const requestedValue = next.requestedValue ?? "";
        if (!controlled) {
          value.set(requestedValue, "reason: tabs value request => uncontrolled value sync");
        }
        run2.expose.emit("valueChange", { value: requestedValue });
        validateSelection(run2, "reason: tabs value request => uncontrolled fallback sync");
        return;
      }
      if (next.validationVersion !== lastValidationVersion) {
        lastValidationVersion = next.validationVersion;
        if (!controlled && next.value !== value.get()) {
          value.set(next.value, "reason: tabs part validation request => value sync");
        }
        validateSelection(run2, "reason: tabs part validation request => selection fallback");
        return;
      }
      if (!controlled && next.value !== value.get()) {
        value.set(next.value, "reason: context.subscribe => uncontrolled tabs value sync");
      }
      validateSelection(run2, "reason: tabs context notification => selection fallback");
    });
    def2.lifecycle.onCreated((run2) => {
      controlled = run2.props.isProvided("value");
      const initialValue = controlled ? run2.props.get().value ?? "" : run2.props.get().defaultValue ?? "";
      currentOrientation = run2.props.get().orientation ?? "horizontal";
      currentActivationMode = run2.props.get().activationMode ?? "automatic";
      value.set(initialValue, "reason: lifecycle.onCreated => initialize tabs value");
      activeValue = initialValue;
      lastValidationVersion = 0;
      publishContext(run2);
    });
    def2.lifecycle.onMounted((run2) => {
      validateSelection(run2, "reason: lifecycle.onMounted => tabs selection fallback");
    });
    def2.props.watch(["value", "orientation", "activationMode"], (run2, next) => {
      controlled = run2.props.isProvided("value");
      if (controlled) {
        value.set(next.value ?? "", "reason: props.watch(value) => controlled tabs sync");
      }
      currentOrientation = next.orientation ?? "horizontal";
      currentActivationMode = next.activationMode ?? "automatic";
      validateSelection(run2, "reason: props.watch => tabs selection fallback");
    });
  }
  var asTabsRoot = defineAsHook({
    name: "as-tabs-root",
    setup: setupTabsRoot
  });
  var tabsRoot = definePrototype({
    name: "base-tabs-root",
    setup: setupTabsRoot
  });
  // ../packages/prototypes/base/src/tabs/list.proto.ts
  function setupTabsList(def2) {
    def2.anatomy.claim(TABS_FAMILY, { role: "list" });
    def2.props.define({
      orientation: { type: "enum", empty: "fallback", options: ["horizontal", "vertical"] },
      loop: { type: "boolean", empty: "fallback" },
      a11yLabel: { type: "string", empty: "fallback" }
    });
    def2.props.setDefaults({
      orientation: "horizontal",
      loop: false,
      a11yLabel: ""
    });
    const orientation = def2.state.string("orientation", "horizontal", {
      options: ["horizontal", "vertical"]
    });
    const a11yLabel = def2.state.string("a11yLabel", "");
    def2.a11y.role("tablist");
    def2.a11y.name(a11yLabel);
    def2.a11y.state("orientation", orientation);
    const focusRoving = asFocusRoving();
    focusRoving.configure({
      navigation: "arrow",
      orientation: "horizontal",
      entry: "manual"
    });
    def2.expose.method("focusFirst", () => focusRoving.focusFirst());
    def2.expose.method("focusLast", () => focusRoving.focusLast());
    def2.expose.method("focusNext", () => focusRoving.focusNext());
    def2.expose.method("focusPrev", () => focusRoving.focusPrev());
    def2.expose.method("focusSelected", () => focusRoving.focusSelected());
    def2.context.subscribe(TABS_CONTEXT, (_run, next) => {
      const nextOrientation = next.orientation ?? "horizontal";
      orientation.set(nextOrientation, "reason: tabs list context orientation sync");
      focusRoving.setOrientation(nextOrientation);
    });
    def2.lifecycle.onMounted((run2) => {
      const ctx = run2.context.read(TABS_CONTEXT);
      const nextOrientation = ctx.orientation ?? "horizontal";
      orientation.set(nextOrientation, "reason: tabs list mounted orientation sync");
      focusRoving.setOrientation(nextOrientation);
      focusRoving.setLoop(!!run2.props.get().loop);
      a11yLabel.set(run2.props.get().a11yLabel ?? "", "reason: tabs list mounted a11y label sync");
    });
    def2.props.watch(["loop", "a11yLabel"], (_run, next) => {
      focusRoving.setLoop(!!next.loop);
      a11yLabel.set(next.a11yLabel ?? "", "reason: tabs list props a11y label sync");
    });
  }
  var asTabsList = defineAsHook({
    name: "as-tabs-list",
    setup: setupTabsList
  });
  var tabsList = definePrototype({
    name: "base-tabs-list",
    setup: setupTabsList
  });
  // ../packages/prototypes/base/src/tabs/trigger.proto.ts
  function syncSelectedFromContext(nextValue, ownValue, selected) {
    selected.set(ownValue === nextValue, "reason: tabs context sync => selected");
  }
  function syncNavParticipationFromContext(ctx, ownValue, disabled, focusable) {
    const activeValue = ctx.activeValue || ctx.value;
    const participates = !!ownValue && ownValue === activeValue && !disabled.get();
    focusable.setNavParticipation(participates ? "auto" : "none");
    focusable.setRovingStatus({
      selected: !!ownValue && ownValue === ctx.value,
      active: participates
    });
  }
  function readTriggerSnapshot2(part) {
    const exposed = part.getExpose("__collectionItem");
    const snapshot = typeof exposed === "function" ? exposed() : exposed && typeof exposed === "object" ? exposed : {};
    const disabledExpose = part.getExpose("disabled");
    const disabled = disabledExpose && typeof disabledExpose === "object" && typeof disabledExpose.get === "function" ? disabledExpose.get() : undefined;
    return {
      ...snapshot && typeof snapshot === "object" ? snapshot : {},
      ...typeof disabled === "undefined" ? {} : { disabled }
    };
  }
  function resolveEnabledTriggerValue(run2, candidate) {
    const values = run2.anatomy.order.partsOf(TABS_FAMILY, "trigger").map(readTriggerSnapshot2).filter((snapshot) => typeof snapshot.value === "string" && snapshot.value).filter((snapshot) => snapshot.disabled !== true).map((snapshot) => snapshot.value);
    if (candidate && values.includes(candidate))
      return candidate;
    return values[0] ?? candidate ?? "";
  }
  function setupTabsTrigger(def2) {
    asTrigger();
    const focusable = asFocusable();
    focusable.configure({ disabled: false });
    const focused = focusable.focused;
    const focusVisible = focusable.focusVisible;
    const selected = def2.state.bool("selected", false);
    const disabled = def2.state.bool("disabled", false);
    const hovered = def2.state.bool("hovered", false);
    const pressed = def2.state.bool("pressed", false);
    const triggerId = def2.state.string("triggerId", "");
    const contentId = def2.state.string("contentId", "");
    const collectionItem = asCollectionItem();
    collectionItem.configure({
      family: TABS_FAMILY,
      role: "trigger",
      getMeta: (run2) => {
        const props = run2.props.get();
        return {
          value: props.value ?? "",
          disabled: !!props.disabled
        };
      }
    });
    def2.props.define({
      value: { type: "string", empty: "fallback" },
      disabled: { type: "boolean", empty: "fallback" }
    });
    def2.props.setDefaults({
      value: "",
      disabled: false
    });
    let ownValue = "";
    let rootId = "";
    def2.expose.state("disabled", disabled);
    def2.expose.state("hovered", hovered);
    def2.expose.state("focused", focused);
    def2.expose.state("focusVisible", focusVisible);
    def2.expose.state("pressed", pressed);
    def2.expose.state("selected", selected);
    def2.expose.method("focusSelf", (options) => {
      if (disabled.get())
        return;
      focusable.focusSelf(options);
    });
    def2.expose.event("click", { payload: "void" });
    def2.a11y.id(triggerId);
    def2.a11y.role("tab");
    def2.a11y.nameFromContent();
    def2.a11y.state("selected", selected);
    def2.a11y.state("disabled", disabled);
    def2.a11y.relation("controls", { target: contentId });
    def2.a11y.action("activate", { event: "click" });
    const syncIds = () => {
      triggerId.set(createTabsPartId(rootId, "trigger", ownValue), "reason: tabs trigger id sync");
      contentId.set(createTabsPartId(rootId, "content", ownValue), "reason: tabs trigger relation sync");
    };
    const syncDisabled = (nextDisabled) => {
      disabled.set(nextDisabled, "reason: tabs trigger sync disabled");
      focusable.setDisabled(nextDisabled);
      if (nextDisabled) {
        hovered.set(false, "reason: tabs trigger disabled => hovered");
        pressed.set(false, "reason: tabs trigger disabled => pressed");
      }
    };
    const requestSelection = (run2, ctx) => {
      const nextValue = run2.props.get().value ?? "";
      run2.context.update(TABS_CONTEXT, {
        ...ctx,
        activeValue: nextValue
      });
      if (ctx.value === nextValue)
        return false;
      run2.context.update(TABS_CONTEXT, {
        ...ctx,
        value: ctx.controlled ? ctx.value : nextValue,
        activeValue: nextValue,
        requestedValue: nextValue,
        requestVersion: ctx.requestVersion + 1
      });
      return true;
    };
    def2.context.subscribe(TABS_CONTEXT, (_run, next) => {
      rootId = next.rootId;
      syncIds();
      syncSelectedFromContext(next.value, ownValue, selected);
      syncNavParticipationFromContext(next, ownValue, disabled, focusable);
    });
    def2.lifecycle.onCreated((run2) => {
      syncDisabled(!!run2.props.get().disabled);
    });
    def2.lifecycle.onMounted((run2) => {
      ownValue = run2.props.get().value ?? "";
      const ctx = run2.context.read(TABS_CONTEXT);
      rootId = ctx.rootId;
      syncIds();
      syncSelectedFromContext(ctx.value, ownValue, selected);
      syncNavParticipationFromContext(ctx, ownValue, disabled, focusable);
      notifyRootToValidateSelection(run2);
    });
    def2.props.watch(["value"], (run2, next) => {
      ownValue = next.value ?? "";
      const ctx = run2.context.read(TABS_CONTEXT);
      rootId = ctx.rootId;
      syncIds();
      syncSelectedFromContext(ctx.value, ownValue, selected);
      syncNavParticipationFromContext(ctx, ownValue, disabled, focusable);
      notifyRootToValidateSelection(run2);
    });
    def2.props.watch(["disabled"], (run2, next) => {
      syncDisabled(!!next.disabled);
      syncNavParticipationFromContext(run2.context.read(TABS_CONTEXT), ownValue, disabled, focusable);
      notifyRootToValidateSelection(run2);
    });
    const updateActiveValue = (run2) => {
      const nextValue = run2.props.get().value ?? "";
      run2.context.update(TABS_CONTEXT, (prev) => {
        if (prev.activeValue === nextValue)
          return prev;
        return { ...prev, activeValue: nextValue };
      });
    };
    const notifyRootToValidateSelection = (run2) => {
      run2.context.update(TABS_CONTEXT, (prev) => ({
        ...prev,
        value: prev.controlled ? prev.value : resolveEnabledTriggerValue(run2, prev.value ?? ""),
        activeValue: resolveEnabledTriggerValue(run2, prev.activeValue || prev.value || ""),
        validationVersion: (prev.validationVersion ?? 0) + 1
      }));
    };
    def2.event.on("press.commit", (run2) => {
      pressed.set(false, "reason: tabs trigger press.commit => pressed");
      if (disabled.get())
        return;
      const ctx = run2.context.read(TABS_CONTEXT);
      run2.expose.emit("click");
      requestSelection(run2, ctx);
    });
    focused.watch((run2, event2) => {
      if (event2.type !== "next" || !event2.next)
        return;
      if (disabled.get())
        return;
      const nextValue = run2.props.get().value ?? "";
      const ctx = run2.context.read(TABS_CONTEXT);
      updateActiveValue(run2);
      if (ctx.activationMode !== "automatic")
        return;
      if (ctx.value === nextValue)
        return;
      requestSelection(run2, ctx);
    });
    def2.event.onGlobal("key.down", (_run, ev) => {
      const detail = ev;
      if (disabled.get())
        return;
      if (!focused.get())
        return;
      if (detail?.key !== " ")
        return;
      ev.control.requestDefaultActionPrevention({
        reason: "tabs.space-activation",
        source: "base-tabs-trigger"
      });
    });
    def2.event.on("pointer.enter", () => {
      if (disabled.get())
        return;
      hovered.set(true, "reason: tabs trigger pointer.enter => hovered");
    });
    def2.event.on("pointer.leave", () => {
      hovered.set(false, "reason: tabs trigger pointer.leave => hovered");
      pressed.set(false, "reason: tabs trigger pointer.leave => pressed");
    });
    def2.event.on("pointer.cancel", () => {
      hovered.set(false, "reason: tabs trigger pointer.cancel => hovered");
      pressed.set(false, "reason: tabs trigger pointer.cancel => pressed");
    });
    def2.event.on("pointer.down", () => {
      if (disabled.get())
        return;
      pressed.set(true, "reason: tabs trigger pointer.down => pressed");
    });
    def2.event.on("pointer.up", () => {
      pressed.set(false, "reason: tabs trigger pointer.up => pressed");
    });
  }
  var asTabsTrigger = defineAsHook({
    name: "as-tabs-trigger",
    setup: setupTabsTrigger
  });
  var tabsTrigger = definePrototype({
    name: "base-tabs-trigger",
    setup: setupTabsTrigger
  });
  // ../packages/prototypes/base/src/tabs/content.proto.ts
  function syncCurrentFromContext(nextValue, ownValue, current, hidden, focusEntry) {
    const nextCurrent = ownValue === nextValue;
    current.set(nextCurrent, "reason: tabs context sync => current");
    hidden.set(!nextCurrent, "reason: tabs context sync => hidden");
    focusEntry.setDisabled(!nextCurrent);
  }
  function setupTabsContent(def2) {
    def2.anatomy.claim(TABS_FAMILY, { role: "content" });
    const current = def2.state.bool("current", false);
    const hidden = def2.state.bool("hidden", true);
    const contentId = def2.state.string("contentId", "");
    const triggerId = def2.state.string("triggerId", "");
    const focusEntry = asFocusEntry();
    focusEntry.configure({
      strategy: "descendant-first",
      fallback: "self",
      disabled: true
    });
    def2.props.define({
      value: { type: "string", empty: "fallback" },
      keepMounted: { type: "boolean", empty: "fallback" }
    });
    def2.props.setDefaults({
      value: "",
      keepMounted: false
    });
    let ownValue = "";
    let rootId = "";
    let keepMounted = false;
    def2.expose.state("current", current);
    def2.expose.state("hidden", hidden);
    def2.a11y.id(contentId);
    def2.a11y.role("tabpanel");
    def2.a11y.state("hidden", hidden);
    def2.a11y.relation("labelledBy", { target: triggerId });
    const syncIds = () => {
      contentId.set(createTabsPartId(rootId, "content", ownValue), "reason: tabs content id sync");
      triggerId.set(createTabsPartId(rootId, "trigger", ownValue), "reason: tabs content relation sync");
    };
    const syncContext = (next, lifecycle) => {
      rootId = next.rootId;
      syncIds();
      syncCurrentFromContext(next.value, ownValue, current, hidden, focusEntry);
      const nextCurrent = next.value === ownValue;
      lifecycle.setPresent(keepMounted || nextCurrent);
    };
    def2.context.subscribe(TABS_CONTEXT, (run2, next) => {
      syncContext(next, run2.lifecycle);
    });
    const syncProps = (next) => {
      ownValue = next.value ?? "";
      keepMounted = next.keepMounted ?? false;
    };
    def2.lifecycle.onCreated((run2) => {
      syncProps(run2.props.get());
      syncContext(run2.context.read(TABS_CONTEXT), run2.lifecycle);
    });
    def2.lifecycle.onMounted((run2) => {
      syncContext(run2.context.read(TABS_CONTEXT), run2.lifecycle);
    });
    def2.props.watch(["value", "keepMounted"], (run2, next) => {
      syncProps(next);
      syncContext(run2.context.read(TABS_CONTEXT), run2.lifecycle);
    });
    def2.rule({
      when: (w) => w.state(hidden).eq(true),
      intent: (i) => i.feedback.style.use(tw("hidden"))
    });
  }
  var asTabsContent = defineAsHook({
    name: "as-tabs-content",
    setup: setupTabsContent
  });
  var tabsContent = definePrototype({
    name: "base-tabs-content",
    setup: setupTabsContent
  });
  // ../packages/prototypes/shadcn/src/tabs/content.proto.ts
  var tabsContent2 = definePrototype({
    name: "shadcn-tabs-content",
    setup(def2) {
      const contentState = asTabsContent().stateHandles;
      if (!contentState) {
        throw new Error("[shadcn-tabs-content] asTabsContent must project Tabs content state handles.");
      }
      const { hidden } = contentState;
      def2.feedback.style.use(tw("flex-1 outline-none"));
      def2.rule({
        when: (w) => w.state(hidden).eq(true),
        intent: (i) => i.feedback.style.use(tw("hidden"))
      });
    }
  });
  var content_proto_default2 = tabsContent2;

  // ../packages/prototypes/shadcn/src/tabs/list.proto.ts
  var tabsList2 = definePrototype({
    name: "shadcn-tabs-list",
    setup(def2) {
      asTabsList();
      def2.feedback.style.use(tw("inline-flex h-9 w-fit items-center justify-center rounded-lg bg-muted p-[3px] text-muted-foreground"));
    }
  });
  var list_proto_default2 = tabsList2;

  // ../packages/prototypes/shadcn/src/tabs/root.proto.ts
  var tabsRoot2 = definePrototype({
    name: "shadcn-tabs-root",
    setup(def2) {
      asTabsRoot();
      def2.feedback.style.use(tw("flex flex-col gap-2"));
    }
  });
  var root_proto_default7 = tabsRoot2;

  // ../packages/prototypes/shadcn/src/tabs/trigger.proto.ts
  var BASE_TOKENS = [
    "relative",
    "inline-flex",
    "h-[calc(100%_-_1px)]",
    "flex-1",
    "items-center",
    "justify-center",
    "gap-1.5",
    "whitespace-nowrap",
    "rounded-md",
    "border",
    "border-transparent",
    "px-2",
    "py-1",
    "text-sm",
    "font-medium",
    "transition-all",
    "outline-none",
    "text-foreground/60",
    "select-none"
  ].join(" ");
  var tabsTrigger2 = definePrototype({
    name: "shadcn-tabs-trigger",
    setup(def2) {
      const triggerState = asTabsTrigger().stateHandles;
      if (!triggerState) {
        throw new Error("[shadcn-tabs-trigger] asTabsTrigger must project Tabs trigger state handles.");
      }
      const { disabled, hovered, focusVisible, selected } = triggerState;
      def2.feedback.style.use(tw(BASE_TOKENS));
      def2.rule({
        when: (w) => w.state(focusVisible).eq(true),
        intent: (i) => i.feedback.style.use(tw("border-ring ring-3 ring-ring/50 outline-1 outline-ring"))
      });
      def2.rule({
        when: (w) => w.state(selected).eq(true),
        intent: (i) => i.feedback.style.use(tw("bg-background text-foreground shadow-sm"))
      });
      def2.rule({
        when: (w) => w.all(w.state(hovered).eq(true), w.state(selected).eq(false)),
        intent: (i) => i.feedback.style.use(tw("text-foreground"))
      });
      def2.rule({
        when: (w) => w.state(disabled).eq(true),
        intent: (i) => i.feedback.style.use(tw("pointer-events-none opacity-50"))
      });
    }
  });
  var trigger_proto_default2 = tabsTrigger2;
  // ../packages/prototypes/base/src/tools/use-open-state.ts
  var useOpenState = defineHook({
    name: "useOpenState",
    mode: "configurable",
    setup(def2, options, api) {
      const prop = options?.prop ?? "open";
      const defaultProp = options?.defaultProp ?? "defaultOpen";
      const disabledProp = options?.disabledProp ?? "disabled";
      const stateKey = options?.stateKey ?? "open";
      const exposeStateKey = options?.exposeStateKey ?? "open";
      const exposeOpenMethodKey = options?.exposeOpenMethodKey ?? "openDropdown";
      const exposeCloseMethodKey = options?.exposeCloseMethodKey ?? "close";
      const exposeToggleMethodKey = options?.exposeToggleMethodKey ?? "toggle";
      def2.props.define({
        [prop]: { type: "boolean", empty: "fallback" },
        [defaultProp]: { type: "boolean", empty: "fallback" },
        [disabledProp]: { type: "boolean", empty: "fallback" }
      });
      def2.props.setDefaults({
        [defaultProp]: false,
        [disabledProp]: false
      });
      const open = def2.state.bool(stateKey, false);
      api.store.prop = prop;
      api.store.defaultProp = defaultProp;
      api.store.disabledProp = disabledProp;
      api.store.open = open;
      api.store.controlled = false;
      api.store.disabled = false;
      const syncFromProps = (run2) => {
        const controlled = run2.props.isProvided(prop);
        api.store.controlled = controlled;
        api.store.disabled = !!run2.props.get()[disabledProp];
        const nextOpen = controlled ? !!run2.props.get()[prop] : !!run2.props.get()[defaultProp];
        open.set(nextOpen, "reason: useOpenState.syncFromProps => initialize");
      };
      const syncControlled = (run2, nextOpen) => {
        api.store.controlled = run2.props.isProvided(prop);
        api.store.disabled = !!run2.props.get()[disabledProp];
        if (!api.store.controlled)
          return;
        open.set(nextOpen, "reason: useOpenState.syncControlled => controlled sync");
      };
      const setOpen = (next, reason) => {
        open.set(next, reason ?? "reason: useOpenState.setOpen");
      };
      const requestOpen = (next, reason) => {
        const run2 = api.store.run;
        if (options?.requestOpen && run2) {
          options.requestOpen(run2, next, reason);
          return;
        }
        setOpen(next, reason);
      };
      api.store.syncFromProps = syncFromProps;
      api.store.syncControlled = syncControlled;
      api.store.setOpen = setOpen;
      def2.expose.state(exposeStateKey, open);
      def2.expose.method(exposeOpenMethodKey, (reason) => {
        requestOpen(true, reason ?? "reason: useOpenState.openNow");
      });
      def2.expose.method(exposeCloseMethodKey, (reason) => {
        requestOpen(false, reason ?? "reason: useOpenState.close");
      });
      def2.expose.method(exposeToggleMethodKey, (reason) => {
        requestOpen(!open.get(), reason ?? "reason: useOpenState.toggle");
      });
      def2.lifecycle.onCreated((run2) => {
        api.store.run = run2;
        syncFromProps(run2);
      });
      def2.lifecycle.onMounted((run2) => {
        api.store.run = run2;
      });
      def2.lifecycle.onUnmounted(() => {
        api.store.run = undefined;
      });
      def2.props.watch([prop, disabledProp], (run2, next) => {
        syncControlled(run2, !!next[prop]);
        api.store.disabled = !!next[disabledProp];
      });
    }
  });
  // ../packages/prototypes/base/src/transition/machine.ts
  function createTransitionMachine(driver) {
    let targetOpen = false;
    let viewMounted = false;
    let queuedIntent = null;
    let pendingTask = null;
    let generation = 0;
    const invalidateCompletion = () => {
      generation += 1;
      pendingTask?.cancel();
      pendingTask = null;
    };
    const setState = (state2) => {
      driver.setState(state2);
    };
    const armCompletion = (state2) => {
      invalidateCompletion();
      const activeGeneration = generation;
      pendingTask = driver.schedule(driver.getDuration(state2), () => {
        if (activeGeneration !== generation || driver.getState() !== state2)
          return;
        pendingTask = null;
        completeCurrent(true);
      });
    };
    const beginEnter = () => {
      driver.setViewPresent(true);
      if (!viewMounted)
        return;
      queuedIntent = null;
      driver.emit("beforeEnter");
      setState("entering");
      armCompletion("entering");
    };
    const beginLeave = () => {
      queuedIntent = null;
      driver.emit("beforeLeave");
      setState("leaving");
      armCompletion("leaving");
    };
    const consumeQueuedIntent = () => {
      const next = queuedIntent;
      queuedIntent = null;
      if (next === "enter")
        enter();
      else if (next === "leave")
        leave();
    };
    const completeCurrent = (consumeQueue) => {
      const current = driver.getState();
      if (current !== "entering" && current !== "leaving")
        return;
      invalidateCompletion();
      if (current === "entering") {
        setState("entered");
        driver.emit("afterEnter");
      } else {
        setState("closed");
        driver.emit("afterLeave");
        driver.setViewPresent(false);
      }
      if (consumeQueue)
        consumeQueuedIntent();
    };
    const enter = () => {
      targetOpen = true;
      const current = driver.getState();
      if (current === "entered")
        return;
      if (current === "closed") {
        beginEnter();
        return;
      }
      if (current === "entering") {
        if (driver.getInterrupt() === "wait")
          queuedIntent = null;
        return;
      }
      const interrupt = driver.getInterrupt();
      if (interrupt === "wait") {
        queuedIntent = "enter";
      } else if (interrupt === "immediate") {
        completeCurrent(false);
        beginEnter();
      } else {
        invalidateCompletion();
        beginEnter();
      }
    };
    const leave = () => {
      targetOpen = false;
      const current = driver.getState();
      if (current === "closed") {
        driver.setViewPresent(false);
        return;
      }
      if (current === "leaving") {
        if (driver.getInterrupt() === "wait")
          queuedIntent = null;
        return;
      }
      if (current === "entered") {
        beginLeave();
        return;
      }
      const interrupt = driver.getInterrupt();
      if (interrupt === "wait") {
        queuedIntent = "leave";
      } else if (interrupt === "immediate") {
        completeCurrent(false);
        beginLeave();
      } else {
        invalidateCompletion();
        beginLeave();
      }
    };
    const initialize = (open, appear) => {
      targetOpen = open;
      queuedIntent = null;
      invalidateCompletion();
      if (!open) {
        setState("closed");
        driver.setViewPresent(false);
        return;
      }
      driver.setViewPresent(true);
      if (!appear)
        setState("entered");
    };
    const mounted = () => {
      viewMounted = true;
      if (targetOpen && driver.getState() === "closed")
        beginEnter();
    };
    const unmounted = () => {
      viewMounted = false;
      invalidateCompletion();
      queuedIntent = null;
      if (driver.getState() !== "closed")
        setState("closed");
    };
    const dispose = () => {
      viewMounted = false;
      queuedIntent = null;
      invalidateCompletion();
    };
    return {
      initialize,
      setTarget(open) {
        if (open === targetOpen)
          return;
        if (open)
          enter();
        else
          leave();
      },
      enter,
      leave,
      complete: () => completeCurrent(true),
      mounted,
      unmounted,
      dispose
    };
  }

  // ../packages/prototypes/base/src/transition/as-transition.proto.ts
  function requireProjectedHandle(value, name) {
    if (typeof value === "undefined") {
      throw new Error(`[asTransition] missing captured handle: ${name}.`);
    }
    return value;
  }
  var asTransition = defineAsHook({
    name: "asTransition",
    setup(def2) {
      def2.props.define({
        open: { type: "boolean", empty: "fallback" },
        defaultOpen: { type: "boolean", empty: "fallback" },
        appear: { type: "boolean", empty: "fallback" },
        enterDuration: { type: "number", empty: "fallback" },
        leaveDuration: { type: "number", empty: "fallback" },
        interrupt: {
          type: "enum",
          empty: "fallback",
          options: ["reverse", "wait", "immediate"]
        }
      });
      def2.props.setDefaults({
        defaultOpen: false,
        appear: false,
        enterDuration: 300,
        leaveDuration: 200,
        interrupt: "reverse"
      });
      def2.expose.event("beforeEnter", { payload: "void" });
      def2.expose.event("afterEnter", { payload: "void" });
      def2.expose.event("beforeLeave", { payload: "void" });
      def2.expose.event("afterLeave", { payload: "void" });
      const transitionState = def2.state.enum("transitionState", "closed", { options: ["closed", "entering", "entered", "leaving"] });
      const isPresent = def2.state.bool("isPresent", false);
      const appearDefault = def2.state.bool("transitionAppearDefault", false);
      const enterDurationDefault = def2.state.numberDiscrete("transitionEnterDurationDefault", 300);
      const leaveDurationDefault = def2.state.numberDiscrete("transitionLeaveDurationDefault", 200);
      const interruptDefault = def2.state.enum("transitionInterruptDefault", "reverse", { options: ["reverse", "wait", "immediate"] });
      let currentRun = null;
      const getProps = () => currentRun?.props.get();
      const driver = {
        getState: () => transitionState.get(),
        setState(state2) {
          transitionState.set(state2, `reason: asTransition => ${state2}`);
          isPresent.set(state2 !== "closed", `reason: asTransition presence => ${state2}`);
        },
        getInterrupt: () => currentRun?.props.isProvided("interrupt") ? getProps()?.interrupt ?? interruptDefault.get() : interruptDefault.get(),
        getDuration: (state2) => {
          if (currentRun?.meta?.get("reducedMotion") === "reduce")
            return 0;
          return state2 === "entering" ? currentRun?.props.isProvided("enterDuration") ? getProps()?.enterDuration ?? enterDurationDefault.get() : enterDurationDefault.get() : currentRun?.props.isProvided("leaveDuration") ? getProps()?.leaveDuration ?? leaveDurationDefault.get() : leaveDurationDefault.get();
        },
        schedule: (durationMs, callback) => delay(durationMs, callback),
        setViewPresent(present) {
          if (!currentRun) {
            throw new Error("[asTransition] runtime lifecycle handle is not available.");
          }
          currentRun.lifecycle.setPresent(present);
        },
        emit(event2) {
          currentRun?.expose.emit(event2);
        }
      };
      const machine = createTransitionMachine(driver);
      const controls = {
        enter: () => machine.enter(),
        leave: () => machine.leave(),
        complete: () => machine.complete()
      };
      def2.lifecycle.onCreated((run2) => {
        currentRun = run2;
        const props = run2.props.get();
        const controlled = run2.props.isProvided("open");
        const open = controlled ? !!props.open : !!props.defaultOpen;
        const appear = run2.props.isProvided("appear") ? !!props.appear : appearDefault.get();
        machine.initialize(open, appear);
      });
      def2.lifecycle.onMounted((run2) => {
        currentRun = run2;
        machine.mounted();
      });
      def2.lifecycle.onUnmounted((run2) => {
        currentRun = run2;
        machine.unmounted();
      });
      def2.lifecycle.onBeforeDispose(() => {
        machine.dispose();
        currentRun = null;
      });
      def2.props.watch(["open", "interrupt", "enterDuration", "leaveDuration"], (run2, next) => {
        currentRun = run2;
        if (!run2.props.isProvided("open"))
          return;
        machine.setTarget(!!next.open);
      });
      def2.expose.state("transitionState", transitionState);
      def2.expose.state("isPresent", isPresent);
      def2.expose.method("enter", controls.enter);
      def2.expose.method("leave", controls.leave);
      def2.expose.method("complete", controls.complete);
      def2.expose.value("controls", controls);
    },
    projectHandle(result) {
      const transitionState = requireProjectedHandle(result.getState?.("transitionState"), "transitionState");
      const isPresent = requireProjectedHandle(result.getState?.("isPresent"), "isPresent");
      const controls = {
        enter: requireProjectedHandle(result.getMethod?.("enter"), "enter"),
        leave: requireProjectedHandle(result.getMethod?.("leave"), "leave"),
        complete: requireProjectedHandle(result.getMethod?.("complete"), "complete")
      };
      const appearDefault = requireProjectedHandle(result.getState?.("transitionAppearDefault"), "transitionAppearDefault");
      const enterDurationDefault = requireProjectedHandle(result.getState?.("transitionEnterDurationDefault"), "transitionEnterDurationDefault");
      const leaveDurationDefault = requireProjectedHandle(result.getState?.("transitionLeaveDurationDefault"), "transitionLeaveDurationDefault");
      const interruptDefault = requireProjectedHandle(result.getState?.("transitionInterruptDefault"), "transitionInterruptDefault");
      const configure = (config) => {
        let configured = false;
        if (typeof config.appear !== "undefined") {
          appearDefault.setDefault(config.appear);
          configured = true;
        }
        if (typeof config.enterDuration !== "undefined") {
          if (!Number.isFinite(config.enterDuration) || config.enterDuration < 0) {
            throw new Error("[asTransition] enterDuration must be a finite non-negative number.");
          }
          enterDurationDefault.setDefault(config.enterDuration);
          configured = true;
        }
        if (typeof config.leaveDuration !== "undefined") {
          if (!Number.isFinite(config.leaveDuration) || config.leaveDuration < 0) {
            throw new Error("[asTransition] leaveDuration must be a finite non-negative number.");
          }
          leaveDurationDefault.setDefault(config.leaveDuration);
          configured = true;
        }
        if (typeof config.interrupt !== "undefined") {
          interruptDefault.setDefault(config.interrupt);
          configured = true;
        }
        if (!configured)
          appearDefault.setDefault(appearDefault.get());
      };
      return { transitionState, isPresent, controls, configure };
    }
  });
  // ../packages/prototypes/base/src/hover-card/shared.ts
  function deriveHoverCardInteractionOpen(ctx) {
    return ctx.triggerHovered || ctx.triggerFocused || ctx.contentHovered;
  }
  function updateHoverCardInteraction(run2, patch, reason) {
    try {
      run2.context.update(HOVER_CARD_CONTEXT, (prev) => ({
        ...prev,
        ...patch,
        interactionReason: reason,
        interactionVersion: prev.interactionVersion + 1
      }));
      return true;
    } catch (error5) {
      if (error5?.code === "CONTEXT_DISCONNECTED")
        return false;
      throw error5;
    }
  }
  function requestHoverCardOpen(run2, nextOpen, reason) {
    try {
      run2.context.update(HOVER_CARD_CONTEXT, (prev) => ({
        ...prev,
        open: prev.controlled ? prev.open : nextOpen,
        requestedOpen: nextOpen,
        requestReason: reason,
        requestVersion: prev.requestVersion + 1
      }));
      return true;
    } catch (error5) {
      if (error5?.code === "CONTEXT_DISCONNECTED")
        return false;
      throw error5;
    }
  }
  var HOVER_CARD_FAMILY = createAnatomyFamily("base-hover-card", {
    roles: {
      root: { cardinality: { min: 1, max: 1 } },
      trigger: { cardinality: { min: 0, max: 1 } },
      content: { cardinality: { min: 0, max: 1 } }
    },
    relations: [
      { kind: "contains", parent: "root", child: "trigger" },
      { kind: "contains", parent: "root", child: "content" }
    ]
  });
  var HOVER_CARD_CONTEXT = createContextKey("base-hover-card");

  // ../packages/prototypes/base/src/hover-card/root.proto.ts
  var DEFAULT_OPEN_DELAY = 700;
  var DEFAULT_CLOSE_DELAY = 300;
  function normalizeDelay(value, fallback) {
    return typeof value === "number" && Number.isFinite(value) && value >= 0 ? value : fallback;
  }
  function sameContext(a, b) {
    return a.open === b.open && a.controlled === b.controlled && a.disabled === b.disabled && a.openDelay === b.openDelay && a.closeDelay === b.closeDelay && a.triggerHovered === b.triggerHovered && a.triggerFocused === b.triggerFocused && a.contentHovered === b.contentHovered && a.interactionReason === b.interactionReason && a.interactionVersion === b.interactionVersion && a.requestedOpen === b.requestedOpen && a.requestReason === b.requestReason && a.requestVersion === b.requestVersion;
  }
  function setupHoverCardRoot(def2) {
    def2.anatomy.claim(HOVER_CARD_FAMILY, { role: "root" });
    def2.props.define({
      open: { type: "boolean", empty: "fallback" },
      defaultOpen: { type: "boolean", empty: "fallback" },
      disabled: { type: "boolean", empty: "fallback" },
      openDelay: { type: "number", empty: "fallback" },
      closeDelay: { type: "number", empty: "fallback" }
    });
    def2.props.setDefaults({
      defaultOpen: false,
      disabled: false,
      openDelay: DEFAULT_OPEN_DELAY,
      closeDelay: DEFAULT_CLOSE_DELAY
    });
    const initialContext = {
      open: false,
      controlled: false,
      disabled: false,
      openDelay: DEFAULT_OPEN_DELAY,
      closeDelay: DEFAULT_CLOSE_DELAY,
      triggerHovered: false,
      triggerFocused: false,
      contentHovered: false,
      interactionReason: null,
      interactionVersion: 0,
      requestedOpen: false,
      requestReason: null,
      requestVersion: 0
    };
    def2.context.provide(HOVER_CARD_CONTEXT, initialContext);
    const openState = useOpenState({
      exposeOpenMethodKey: "openHoverCard",
      requestOpen(run2, nextOpen, reason) {
        const ctx = run2.context.read(HOVER_CARD_CONTEXT);
        if (ctx.disabled)
          return;
        requestHoverCardOpen(run2, nextOpen, reason);
      }
    });
    const open = openState.getState?.("open");
    def2.expose.event("openChange", { payload: "json" });
    let snapshot = initialContext;
    let published = initialContext;
    let lastRequestVersion = 0;
    let lastInteractionVersion = 0;
    let pendingIntent = null;
    let pendingDelay = null;
    const cancelPending = () => {
      pendingDelay?.cancel();
      pendingDelay = null;
      pendingIntent = null;
    };
    const syncContext = (run2) => {
      const next = { ...snapshot, open: open?.get() ?? false };
      snapshot = next;
      if (sameContext(published, next))
        return;
      published = next;
      run2.context.update(HOVER_CARD_CONTEXT, next);
    };
    const scheduleInteractionRequest = (run2, nextOpen, reason) => {
      cancelPending();
      if (snapshot.disabled || nextOpen === (open?.get() ?? false))
        return;
      const duration = nextOpen ? snapshot.openDelay : snapshot.closeDelay;
      pendingIntent = nextOpen;
      pendingDelay = delay(duration, () => {
        if (pendingIntent !== nextOpen)
          return;
        pendingDelay = null;
        pendingIntent = null;
        const latest = run2.context.read(HOVER_CARD_CONTEXT);
        if (latest.disabled || deriveHoverCardInteractionOpen(latest) !== nextOpen)
          return;
        requestHoverCardOpen(run2, nextOpen, reason);
      });
    };
    def2.context.subscribe(HOVER_CARD_CONTEXT, (run2, next) => {
      snapshot = next;
      published = next;
      if (next.requestVersion !== lastRequestVersion) {
        lastRequestVersion = next.requestVersion;
        if (!next.controlled) {
          open?.set(next.requestedOpen, "reason: hover-card request => uncontrolled sync");
        }
        run2.expose.emit("openChange", {
          open: next.requestedOpen,
          reason: next.requestReason
        });
        return;
      }
      if (next.interactionVersion !== lastInteractionVersion) {
        lastInteractionVersion = next.interactionVersion;
        scheduleInteractionRequest(run2, deriveHoverCardInteractionOpen(next), next.interactionReason ?? "interaction");
      }
    });
    def2.lifecycle.onCreated((run2) => {
      const props = run2.props.get();
      snapshot = {
        ...snapshot,
        controlled: run2.props.isProvided("open"),
        disabled: !!props.disabled,
        openDelay: normalizeDelay(props.openDelay, DEFAULT_OPEN_DELAY),
        closeDelay: normalizeDelay(props.closeDelay, DEFAULT_CLOSE_DELAY)
      };
      syncContext(run2);
    });
    def2.props.watch(["open", "disabled", "openDelay", "closeDelay"], (run2, next) => {
      snapshot = {
        ...snapshot,
        controlled: run2.props.isProvided("open"),
        disabled: !!next.disabled,
        openDelay: normalizeDelay(next.openDelay, DEFAULT_OPEN_DELAY),
        closeDelay: normalizeDelay(next.closeDelay, DEFAULT_CLOSE_DELAY)
      };
      if (snapshot.disabled)
        cancelPending();
      syncContext(run2);
    });
    open?.watch((run2, event2) => {
      if (event2.type !== "next")
        return;
      if (pendingIntent === event2.next)
        cancelPending();
      syncContext(run2);
    });
    def2.lifecycle.onBeforeDispose(cancelPending);
  }
  var asHoverCardRoot = defineAsHook({
    name: "as-hover-card-root",
    setup: setupHoverCardRoot
  });
  var hoverCardRoot = definePrototype({
    name: "base-hover-card-root",
    setup(def2) {
      setupHoverCardRoot(def2);
      def2.feedback.style.use(tw("relative inline-flex items-start"));
    }
  });
  // ../packages/prototypes/base/src/hover-card/trigger.proto.ts
  function setupHoverCardTrigger(def2) {
    def2.anatomy.claim(HOVER_CARD_FAMILY, { role: "trigger" });
    def2.props.define({ disabled: { type: "boolean", empty: "fallback" } });
    def2.props.setDefaults({ disabled: false });
    const disabled = def2.state.bool("disabled", false);
    const hovered = def2.state.bool("hovered", false);
    const focusable = asFocusable();
    focusable.configure({ disabled: false });
    const focused = focusable.focused;
    const focusVisible = focusable.focusVisible;
    def2.expose.state("disabled", disabled);
    def2.expose.state("hovered", hovered);
    def2.expose.state("focused", focused);
    def2.expose.state("focusVisible", focusVisible);
    def2.expose.method("focusSelf", (options) => {
      if (!disabled.get())
        focusable.focusSelf(options);
    });
    const syncDisabled = (run2) => {
      const ctx = run2.context.read(HOVER_CARD_CONTEXT);
      const nextDisabled = !!run2.props.get().disabled || ctx.disabled;
      disabled.set(nextDisabled, "reason: hover-card trigger disabled sync");
      focusable.setDisabled(nextDisabled);
      if (!nextDisabled)
        return;
      hovered.set(false, "reason: hover-card trigger disabled => hovered false");
      if (!ctx.triggerHovered && !ctx.triggerFocused)
        return;
      updateHoverCardInteraction(run2, { triggerHovered: false, triggerFocused: false }, "trigger.pointerleave");
    };
    def2.context.subscribe(HOVER_CARD_CONTEXT, (run2) => syncDisabled(run2));
    def2.props.watch(["disabled"], (run2) => syncDisabled(run2));
    def2.lifecycle.onCreated((run2) => syncDisabled(run2));
    def2.event.on("pointer.enter", (run2) => {
      if (disabled.get())
        return;
      hovered.set(true, "reason: hover-card trigger pointer.enter");
      updateHoverCardInteraction(run2, { triggerHovered: true }, "trigger.pointerenter");
    });
    def2.event.on("pointer.leave", (run2) => {
      hovered.set(false, "reason: hover-card trigger pointer.leave");
      updateHoverCardInteraction(run2, { triggerHovered: false }, "trigger.pointerleave");
    });
    focused.watch((run2, event2) => {
      if (event2.type !== "next")
        return;
      updateHoverCardInteraction(run2, { triggerFocused: event2.next }, "trigger.focus");
    });
  }
  var asHoverCardTrigger = defineAsHook({
    name: "as-hover-card-trigger",
    setup: setupHoverCardTrigger
  });
  var hoverCardTrigger = definePrototype({
    name: "base-hover-card-trigger",
    setup: setupHoverCardTrigger
  });
  // ../packages/prototypes/base/src/hover-card/content.proto.ts
  function projectHoverCardContentHandle(result) {
    const open = result.getState?.("open");
    const asTransition2 = result.getAsHookHandle?.("asTransition");
    if (!open || !asTransition2) {
      throw new Error("[as-hover-card-content] missing captured Hover Card or Transition handles.");
    }
    return { stateHandles: { open }, asTransition: asTransition2 };
  }
  function setupHoverCardContent(def2) {
    def2.anatomy.claim(HOVER_CARD_FAMILY, { role: "content" });
    def2.props.define({
      side: {
        type: "enum",
        empty: "fallback",
        options: ["top", "right", "bottom", "left"]
      },
      align: {
        type: "enum",
        empty: "fallback",
        options: ["start", "center", "end"]
      },
      sideOffset: { type: "number", empty: "fallback" },
      alignOffset: { type: "number", empty: "fallback" },
      avoidCollisions: { type: "boolean", empty: "fallback" },
      collisionPadding: { type: "number", empty: "fallback" }
    });
    def2.props.setDefaults({
      side: "bottom",
      align: "center",
      sideOffset: 4,
      alignOffset: 0,
      avoidCollisions: true,
      collisionPadding: 0
    });
    const overlay2 = asOverlay();
    overlay2.configure({
      closeOnEscape: false,
      closeOnOutsidePress: false,
      closeOnFocusOutside: false,
      restore: "none",
      entry: "manual",
      placement: "bottom",
      align: "center",
      sideOffset: 4,
      alignOffset: 0,
      anchored: true,
      strategy: "fixed",
      avoidCollisions: true,
      collisionBoundary: "clippingAncestors",
      collisionPadding: 0,
      portal: true,
      modal: false,
      layerRole: "hover-card-content",
      meta: { overlayKind: "hover-card" }
    });
    const transition = asTransition();
    overlay2.bindPresence({
      enter: transition.controls.enter,
      leave: transition.controls.leave,
      present: transition.isPresent
    });
    const open = def2.state.bool("open", false);
    const hovered = def2.state.bool("hovered", false);
    def2.expose.state("open", open);
    const updateOpen = (nextOpen, reason) => {
      open.set(nextOpen, reason);
      if (nextOpen)
        overlay2.openOverlay(reason);
      else
        overlay2.close(reason);
    };
    const syncPosition = (run2) => {
      const props = run2.props.get();
      overlay2.updatePosition({
        placement: props.side,
        align: props.align,
        sideOffset: props.sideOffset,
        alignOffset: props.alignOffset,
        avoidCollisions: props.avoidCollisions,
        collisionPadding: props.collisionPadding,
        strategy: "fixed",
        collisionBoundary: "clippingAncestors"
      });
    };
    def2.props.watch(["side", "align", "sideOffset", "alignOffset", "avoidCollisions", "collisionPadding"], (run2) => syncPosition(run2));
    def2.context.subscribe(HOVER_CARD_CONTEXT, (_run, next) => {
      updateOpen(next.open, "reason: hover-card context sync => content open");
    });
    def2.lifecycle.onCreated((run2) => {
      syncPosition(run2);
      const ctx = run2.context.read(HOVER_CARD_CONTEXT);
      updateOpen(ctx.open, "reason: lifecycle.onCreated => hover-card content open sync");
    });
    def2.lifecycle.onMounted((run2) => {
      const trigger = run2.anatomy.partsOf(HOVER_CARD_FAMILY, "trigger")[0] ?? null;
      if (trigger)
        overlay2.registerAnchorPart(trigger);
      syncPosition(run2);
      const ctx = run2.context.read(HOVER_CARD_CONTEXT);
      updateOpen(ctx.open, "reason: lifecycle.onMounted => hover-card content open sync");
    });
    def2.lifecycle.onUnmounted(() => {
      hovered.set(false, "reason: hover-card content unmounted => hovered false");
    });
    def2.event.on("pointer.enter", (run2) => {
      hovered.set(true, "reason: hover-card content pointer.enter");
      updateHoverCardInteraction(run2, { contentHovered: true }, "content.pointerenter");
    });
    def2.event.on("pointer.leave", (run2) => {
      hovered.set(false, "reason: hover-card content pointer.leave");
      updateHoverCardInteraction(run2, { contentHovered: false }, "content.pointerleave");
    });
    def2.rule({
      when: (w) => w.state(transition.isPresent).eq(false),
      intent: (i) => i.feedback.style.use(tw("hidden"))
    });
  }
  var asHoverCardContent = defineAsHook({
    name: "as-hover-card-content",
    setup: setupHoverCardContent,
    projectHandle: projectHoverCardContentHandle
  });
  var hoverCardContent = definePrototype({
    name: "base-hover-card-content",
    setup(def2) {
      setupHoverCardContent(def2);
      def2.feedback.style.use(tw("absolute z-40"));
    }
  });
  // ../packages/prototypes/shadcn/src/hover-card/content.proto.ts
  var hoverCardContent2 = definePrototype({
    name: "shadcn-hover-card-content",
    setup(def2) {
      const hoverCard = asHoverCardContent();
      hoverCard.asTransition.configure({ enterDuration: 200, leaveDuration: 200 });
      const { open } = hoverCard.stateHandles;
      def2.feedback.style.use(tw("z-50 w-64 rounded-md border bg-popover p-4 text-sm text-popover-foreground shadow-md outline-none transition-none duration-200"));
      def2.rule({
        when: (w) => w.state(open).eq(true),
        intent: (i) => i.feedback.style.use(tw("animate-in fade-in-0 zoom-in-95"))
      });
      def2.rule({
        when: (w) => w.state(open).eq(false),
        intent: (i) => i.feedback.style.use(tw("animate-out fade-out-0 zoom-out-95"))
      });
      def2.rule({
        when: (w) => w.all(w.state(open).eq(true), w.prop("side").eq("bottom")),
        intent: (i) => i.feedback.style.use(tw("slide-in-from-top-2"))
      });
      def2.rule({
        when: (w) => w.all(w.state(open).eq(true), w.prop("side").eq("top")),
        intent: (i) => i.feedback.style.use(tw("slide-in-from-bottom-2"))
      });
      def2.rule({
        when: (w) => w.all(w.state(open).eq(true), w.prop("side").eq("left")),
        intent: (i) => i.feedback.style.use(tw("slide-in-from-right-2"))
      });
      def2.rule({
        when: (w) => w.all(w.state(open).eq(true), w.prop("side").eq("right")),
        intent: (i) => i.feedback.style.use(tw("slide-in-from-left-2"))
      });
    }
  });
  var content_proto_default4 = hoverCardContent2;

  // ../packages/prototypes/shadcn/src/hover-card/root.proto.ts
  var hoverCardRoot2 = definePrototype({
    name: "shadcn-hover-card-root",
    setup(def2) {
      asHoverCardRoot();
      def2.feedback.style.use(tw("relative inline-flex items-start"));
    }
  });
  var root_proto_default9 = hoverCardRoot2;

  // ../packages/prototypes/shadcn/src/hover-card/trigger.proto.ts
  var TRIGGER_BASE_TOKENS = "inline-flex cursor-pointer items-center text-sm font-medium underline-offset-4 outline-none";
  var hoverCardTrigger2 = definePrototype({
    name: "shadcn-hover-card-trigger",
    setup(def2) {
      const hoverCard = asHoverCardTrigger();
      const state2 = hoverCard.stateHandles;
      if (!state2) {
        throw new Error("[shadcn-hover-card-trigger] missing Hover Card Trigger state handles.");
      }
      const { disabled, hovered, focusVisible } = state2;
      def2.feedback.style.use(tw(TRIGGER_BASE_TOKENS));
      def2.rule({
        when: (w) => w.state(hovered).eq(true),
        intent: (i) => i.feedback.style.use(tw("underline"))
      });
      def2.rule({
        when: (w) => w.state(focusVisible).eq(true),
        intent: (i) => i.feedback.style.use(tw("ring-2 ring-ring ring-offset-2"))
      });
      def2.rule({
        when: (w) => w.state(disabled).eq(true),
        intent: (i) => i.feedback.style.use(tw("pointer-events-none opacity-50"))
      });
    }
  });
  var trigger_proto_default4 = hoverCardTrigger2;

  // ../packages/prototypes/base/src/dropdown/shared.ts
  var nextDropdownRootId = 0;
  function createDropdownRootId() {
    nextDropdownRootId += 1;
    return `pui-dropdown-${nextDropdownRootId}`;
  }
  function createDropdownContentId(rootId) {
    return `${rootId || "pui-dropdown"}-content`;
  }
  function requestDropdownOpen(run2, nextOpen, reason, focusReason, entry = null) {
    try {
      const root = run2.anatomy.partsOf(DROPDOWN_FAMILY, "root")[0] ?? null;
      const requestOpen = root?.getExpose("requestOpen");
      return requestOpen?.({
        open: nextOpen,
        reason,
        focusReason,
        entry: nextOpen ? entry : null
      }) ?? false;
    } catch (error5) {
      if (error5?.code === "CONTEXT_DISCONNECTED")
        return false;
      throw error5;
    }
  }
  var DROPDOWN_FAMILY = createAnatomyFamily("base-dropdown", {
    roles: {
      root: { cardinality: { min: 1, max: 1 } },
      trigger: { cardinality: { min: 0, max: 1 } },
      content: { cardinality: { min: 0, max: 1 } },
      item: { cardinality: { min: 0, max: 100 } }
    },
    relations: [
      { kind: "contains", parent: "root", child: "trigger" },
      { kind: "contains", parent: "root", child: "content" },
      { kind: "contains", parent: "content", child: "item" }
    ]
  });
  var DROPDOWN_CONTEXT = createContextKey("base-dropdown");

  // ../packages/prototypes/base/src/dropdown/root.proto.ts
  function sameContext2(a, b) {
    return Object.keys(a).every((key) => a[key] === b[key]);
  }
  function setupDropdownRoot(def2) {
    def2.anatomy.claim(DROPDOWN_FAMILY, { role: "root" });
    const collection2 = asCollection();
    collection2.configure({ family: DROPDOWN_FAMILY });
    def2.props.define({
      open: { type: "boolean", empty: "fallback" },
      defaultOpen: { type: "boolean", empty: "fallback" },
      disabled: { type: "boolean", empty: "fallback" },
      closeOnItemCommit: { type: "boolean", empty: "fallback" },
      openEntry: { type: "string", empty: "fallback" },
      openEntryValue: { type: "string", empty: "fallback" }
    });
    def2.props.setDefaults({
      defaultOpen: false,
      disabled: false,
      closeOnItemCommit: true,
      openEntry: "active-or-first",
      openEntryValue: ""
    });
    const initialContext = {
      rootId: "",
      open: false,
      controlled: false,
      disabled: false,
      activeValue: "",
      closeOnItemCommit: true,
      openEntry: "active-or-first",
      openEntryValue: "",
      requestReason: null,
      requestFocusReason: null,
      requestEntry: null
    };
    def2.context.provide(DROPDOWN_CONTEXT, initialContext);
    let submitRequest = (_run, _request) => false;
    const openState = useOpenState({
      requestOpen(run2, nextOpen, reason) {
        submitRequest(run2, {
          open: nextOpen,
          reason,
          focusReason: "programmatic"
        });
      }
    });
    const open = openState.getState?.("open");
    def2.expose.event("openChange", { payload: "json" });
    let snapshot = initialContext;
    let published = initialContext;
    let currentRun = null;
    const syncContext = (run2) => {
      const next = { ...snapshot, open: open?.get() ?? false };
      snapshot = next;
      if (sameContext2(published, next))
        return;
      published = next;
      run2.context.update(DROPDOWN_CONTEXT, next);
    };
    def2.context.subscribe(DROPDOWN_CONTEXT, (run2, next) => {
      snapshot = next;
      published = next;
    });
    submitRequest = (run2, request) => {
      if (snapshot.disabled)
        return false;
      snapshot = {
        ...snapshot,
        activeValue: request.open || snapshot.controlled ? snapshot.activeValue : "",
        requestReason: request.reason,
        requestFocusReason: request.focusReason,
        requestEntry: request.open ? request.entry ?? null : null
      };
      if (!snapshot.controlled) {
        open?.set(request.open, "reason: dropdown root accepted request");
      }
      syncContext(run2);
      run2.expose.emit("openChange", {
        open: request.open,
        reason: request.reason,
        focusReason: request.focusReason
      });
      return true;
    };
    def2.expose.method("requestOpen", (request) => {
      if (!currentRun)
        return false;
      return submitRequest(currentRun, request);
    });
    def2.lifecycle.onCreated((run2) => {
      currentRun = run2;
      const props = run2.props.get();
      snapshot = {
        ...snapshot,
        rootId: createDropdownRootId(),
        controlled: run2.props.isProvided("open"),
        disabled: !!props.disabled,
        closeOnItemCommit: props.closeOnItemCommit !== false,
        openEntry: props.openEntry ?? "active-or-first",
        openEntryValue: props.openEntryValue ?? ""
      };
      syncContext(run2);
    });
    def2.lifecycle.onMounted((run2) => {
      currentRun = run2;
    });
    def2.lifecycle.onUnmounted(() => {
      currentRun = null;
    });
    def2.props.watch(["open", "disabled", "closeOnItemCommit", "openEntry", "openEntryValue"], (run2, next) => {
      snapshot = {
        ...snapshot,
        controlled: run2.props.isProvided("open"),
        disabled: !!next.disabled,
        closeOnItemCommit: next.closeOnItemCommit !== false,
        openEntry: next.openEntry ?? "active-or-first",
        openEntryValue: next.openEntryValue ?? ""
      };
      syncContext(run2);
    });
    open?.watch((run2, event2) => {
      if (event2.type !== "next")
        return;
      if (!event2.next)
        snapshot = { ...snapshot, activeValue: "", requestEntry: null };
      syncContext(run2);
    });
  }
  var asDropdownRoot = defineAsHook({
    name: "as-dropdown-root",
    setup: setupDropdownRoot
  });
  var dropdownRoot = definePrototype({
    name: "base-dropdown-root",
    setup: setupDropdownRoot
  });
  // ../packages/prototypes/base/src/dropdown/command.ts
  function setupDropdownCommand(def2, reasonPrefix, options) {
    asTrigger();
    def2.props.define({ disabled: { type: "boolean", empty: "fallback" } });
    def2.props.setDefaults({ disabled: false });
    const disabled = def2.state.bool("disabled", false);
    const hovered = def2.state.bool("hovered", false);
    const pressed = def2.state.bool("pressed", false);
    const focusable = asFocusable();
    focusable.configure({ disabled: false });
    const focused = focusable.focused;
    const focusVisible = focusable.focusVisible;
    def2.expose.state("disabled", disabled);
    def2.expose.state("hovered", hovered);
    def2.expose.state("focused", focused);
    def2.expose.state("focusVisible", focusVisible);
    def2.expose.state("pressed", pressed);
    const focusSelf = (focusOptions) => {
      if (disabled.get() && !options?.focusableWhenDisabled)
        return;
      focusable.focusSelf(focusOptions);
    };
    def2.expose.method("focusSelf", focusSelf);
    const clearTransient = (reason) => {
      hovered.set(false, reason);
      pressed.set(false, reason);
    };
    const syncDisabled = (nextDisabled) => {
      disabled.set(nextDisabled, `reason: ${reasonPrefix} disabled sync`);
      focusable.setDisabled(nextDisabled && !options?.focusableWhenDisabled);
      if (nextDisabled)
        clearTransient(`reason: ${reasonPrefix} disabled => reset interaction`);
    };
    def2.event.on("key.down", (_run, ev) => {
      const detail = ev;
      if (disabled.get() || !focused.get() || detail?.key !== " ")
        return;
      ev.control.requestDefaultActionPrevention({
        reason: `${reasonPrefix}.space-activation`,
        source: reasonPrefix
      });
    });
    def2.event.on("pointer.enter", () => {
      if (!disabled.get())
        hovered.set(true, `reason: ${reasonPrefix} pointer.enter`);
    });
    def2.event.on("pointer.leave", () => clearTransient(`reason: ${reasonPrefix} pointer.leave`));
    def2.event.on("pointer.cancel", () => clearTransient(`reason: ${reasonPrefix} pointer.cancel`));
    def2.event.on("pointer.down", () => {
      if (!disabled.get())
        pressed.set(true, `reason: ${reasonPrefix} pointer.down`);
    });
    def2.event.on("pointer.up", () => pressed.set(false, `reason: ${reasonPrefix} pointer.up`));
    def2.event.on("press.commit", () => {
      pressed.set(false, `reason: ${reasonPrefix} press.commit`);
    });
    return {
      disabled,
      hovered,
      focused,
      focusVisible,
      pressed,
      focusSelf,
      setRovingStatus: (status) => focusable.setRovingStatus(status),
      syncDisabled
    };
  }

  // ../packages/prototypes/base/src/dropdown/trigger.proto.ts
  function setupDropdownTrigger(def2) {
    def2.anatomy.claim(DROPDOWN_FAMILY, { role: "trigger" });
    const command = setupDropdownCommand(def2, "dropdown trigger");
    const expanded = def2.state.bool("dropdownExpanded", false);
    const hasPopup = def2.state.string("dropdownHasPopup", "menu");
    const controls = def2.state.string("dropdownContentId", "");
    def2.a11y.role("button");
    def2.a11y.nameFromContent();
    def2.a11y.state("disabled", command.disabled);
    def2.a11y.state("expanded", expanded);
    def2.a11y.state("hasPopup", hasPopup);
    def2.a11y.relation("controls", { target: controls });
    def2.a11y.action("activate", { event: "click" });
    const sync = (run2, ctx) => {
      command.syncDisabled(!!run2.props.get().disabled || ctx.disabled);
      expanded.set(ctx.open, "reason: dropdown trigger expanded sync");
      controls.set(createDropdownContentId(ctx.rootId), "reason: dropdown trigger controls sync");
    };
    def2.context.subscribe(DROPDOWN_CONTEXT, (run2, next) => sync(run2, next));
    def2.lifecycle.onCreated((run2) => sync(run2, run2.context.read(DROPDOWN_CONTEXT)));
    def2.props.watch(["disabled"], (run2) => sync(run2, run2.context.read(DROPDOWN_CONTEXT)));
    command.focused.watch((run2, event2) => {
      if (event2.type !== "next" || !event2.next)
        return;
      const ctx = run2.context.read(DROPDOWN_CONTEXT);
      if (!ctx.open || !ctx.activeValue)
        return;
      run2.context.update(DROPDOWN_CONTEXT, (prev) => ({ ...prev, activeValue: "" }));
    });
    def2.event.on("press.commit", (run2, ev) => {
      if (command.disabled.get())
        return;
      const ctx = run2.context.read(DROPDOWN_CONTEXT);
      const key = ev?.key;
      const focusReason = key ? "keyboard" : "pointer";
      if (key === "Enter" || key === " ") {
        requestDropdownOpen(run2, true, "trigger.press", "keyboard", "first");
        return;
      }
      requestDropdownOpen(run2, !ctx.open, "trigger.press", focusReason);
    });
    def2.event.on("key.down", (run2, ev) => {
      if (command.disabled.get())
        return;
      const key = ev?.key;
      if (key !== "ArrowDown" && key !== "ArrowUp")
        return;
      ev.control.requestDefaultActionPrevention({
        reason: "dropdown.arrow-open",
        source: "base-dropdown-trigger"
      });
      const ctx = run2.context.read(DROPDOWN_CONTEXT);
      const entry = key === "ArrowUp" ? "last" : "first";
      if (!ctx.open) {
        requestDropdownOpen(run2, true, `trigger.${key}`, "keyboard", entry);
        return;
      }
      if (ctx.activeValue)
        return;
      const content = run2.anatomy.partsOf(DROPDOWN_FAMILY, "content")[0] ?? null;
      const focusBoundary = content?.getExpose(entry === "first" ? "focusFirst" : "focusLast");
      focusBoundary?.();
    });
  }
  var asDropdownTrigger = defineAsHook({
    name: "as-dropdown-trigger",
    setup: setupDropdownTrigger
  });
  var dropdownTrigger = definePrototype({
    name: "base-dropdown-trigger",
    setup: setupDropdownTrigger
  });
  // ../packages/prototypes/base/src/behaviors/use-typeahead-navigation.ts
  var useTypeaheadNavigation = defineHook({
    name: "useTypeaheadNavigation",
    setup(def2, options, api) {
      const store = api.store;
      store.buffer = "";
      store.resetTask = null;
      const clear = () => {
        store.resetTask?.cancel();
        store.resetTask = null;
        store.buffer = "";
      };
      def2.event.onGlobal("key.down", (run2, ev) => {
        if (!options.isEnabled(run2))
          return;
        const detail = ev;
        const key = detail?.key;
        if (typeof key !== "string" || key.length !== 1)
          return;
        if (detail?.ctrlKey || detail?.metaKey || detail?.altKey)
          return;
        const entries = options.getEntries(run2);
        if (entries.length === 0)
          return;
        const currentIndex = Math.max(-1, options.getCurrentIndex(run2, entries));
        const nextBuffer = `${store.buffer ?? ""}${key}`.toLowerCase();
        store.buffer = nextBuffer;
        store.resetTask?.cancel();
        store.resetTask = delay(options.resetAfter ?? 400, () => {
          store.buffer = "";
          store.resetTask = null;
        });
        const findMatch = (query) => {
          for (let step = 1;step <= entries.length; step++) {
            const entry = entries[(currentIndex + step) % entries.length];
            if (options.getText(entry).toLowerCase().startsWith(query))
              return entry;
          }
          return null;
        };
        const match = findMatch(nextBuffer) ?? findMatch(key.toLowerCase());
        if (match)
          options.onMatch(run2, match);
      });
      def2.lifecycle.onUnmounted(clear);
    }
  });
  // ../packages/prototypes/base/src/dropdown/content.proto.ts
  function projectDropdownContentHandle(result) {
    const open = result.getState?.("open");
    const asTransition2 = result.getAsHookHandle?.("asTransition");
    if (!open || !asTransition2) {
      throw new Error("[as-dropdown-content] missing captured Dropdown or Transition handles.");
    }
    return { stateHandles: { open }, asTransition: asTransition2 };
  }
  function setupDropdownContent(def2, _options, api) {
    def2.anatomy.claim(DROPDOWN_FAMILY, { role: "content" });
    def2.props.define({
      side: { type: "enum", empty: "fallback", options: ["top", "right", "bottom", "left"] },
      align: { type: "enum", empty: "fallback", options: ["start", "center", "end"] },
      sideOffset: { type: "number", empty: "fallback" },
      alignOffset: { type: "number", empty: "fallback" },
      avoidCollisions: { type: "boolean", empty: "fallback" },
      collisionPadding: { type: "number", empty: "fallback" },
      excludeAnchorTranslation: { type: "boolean", empty: "fallback" }
    });
    def2.props.setDefaults({
      side: "bottom",
      align: "center",
      sideOffset: 4,
      alignOffset: 0,
      avoidCollisions: true,
      collisionPadding: 0,
      excludeAnchorTranslation: false
    });
    const contentId = def2.state.string("dropdownContentId", "");
    const orientation = def2.state.string("dropdownOrientation", "vertical");
    def2.a11y.id(contentId);
    def2.a11y.role("menu");
    def2.a11y.state("orientation", orientation);
    let currentContext = null;
    const focusScope = asFocusScope();
    focusScope.configure({ entry: "manual", restore: "none" });
    const focusRoving = asFocusRoving();
    focusRoving.configure({ navigation: "arrow", orientation: "vertical", entry: "manual" });
    def2.expose.method("focusFirst", () => focusRoving.focusFirst());
    def2.expose.method("focusLast", () => focusRoving.focusLast());
    def2.expose.method("focusNext", () => focusRoving.focusNext());
    def2.expose.method("focusPrev", () => focusRoving.focusPrev());
    const overlay2 = asOverlay();
    overlay2.configure({
      closeOnEscape: true,
      closeOnOutsidePress: false,
      closeOnFocusOutside: false,
      restore: "none",
      entry: "manual",
      placement: "bottom",
      align: "center",
      sideOffset: 4,
      alignOffset: 0,
      anchored: true,
      strategy: "fixed",
      avoidCollisions: true,
      collisionBoundary: "clippingAncestors",
      collisionPadding: 0,
      excludeAnchorTranslation: false,
      portal: true,
      modal: false,
      layerRole: "dropdown-menu-content",
      meta: { overlayKind: "dropdown-menu" }
    });
    const boundary2 = asBoundary();
    boundary2.observe("pointer.press");
    const transition = asTransition();
    transition.configure({ enterDuration: 0, leaveDuration: 0 });
    overlay2.bindPresence({
      enter: transition.controls.enter,
      leave: transition.controls.leave,
      present: transition.isPresent
    });
    const open = def2.state.bool("open", false);
    def2.expose.state("open", open);
    const store = api?.store ?? {};
    store.run = null;
    const readContext = (run2) => {
      try {
        return run2.context.read(DROPDOWN_CONTEXT);
      } catch (error5) {
        if (error5?.code === "CONTEXT_DISCONNECTED")
          return null;
        throw error5;
      }
    };
    const getNavigationEntries = (run2) => run2.anatomy.order.partsOf(DROPDOWN_FAMILY, "item").map((item) => ({
      snapshot: item.getExpose("getCollectionItem")?.(),
      focusSelf: item.getExpose("focusSelf"),
      focused: !!item.getExpose("focused")?.get?.()
    }));
    useTypeaheadNavigation({
      isEnabled: (run2) => {
        const ctx = readContext(run2);
        return !!ctx?.open && !ctx.disabled && getNavigationEntries(run2).some((entry) => entry.focused);
      },
      getEntries: (run2) => getNavigationEntries(run2).filter((entry) => !!entry.snapshot && !!entry.focusSelf),
      getCurrentIndex: (_run, entries) => entries.findIndex((entry) => entry.focused),
      getText: (entry) => String(entry.snapshot?.textValue || entry.snapshot?.value || ""),
      onMatch: (_run, entry) => entry.focusSelf?.({ reason: "keyboard" })
    });
    const focusValue = (run2, value, options) => {
      if (!value)
        return false;
      const entry = getNavigationEntries(run2).find((candidate) => String(candidate.snapshot?.value ?? "") === value);
      if (!entry?.focusSelf)
        return false;
      entry.focusSelf(options);
      return true;
    };
    const resolveOpenFocusAction = (run2, ctx) => {
      const entry = ctx.requestEntry ?? ctx.openEntry;
      const options = {
        defer: true,
        preventScroll: true,
        reason: ctx.requestFocusReason ?? "programmatic"
      };
      if (entry === "last") {
        focusRoving.focusLast(options);
        return;
      }
      if (entry === "value-or-first" && focusValue(run2, ctx.openEntryValue, options)) {
        return;
      }
      if (entry === "active-or-first" && focusValue(run2, ctx.activeValue, options)) {
        return;
      }
      focusRoving.focusFirst(options);
    };
    const syncPosition = (run2) => {
      const props = run2.props.get();
      overlay2.updatePosition({
        placement: props.side,
        align: props.align,
        sideOffset: props.sideOffset,
        alignOffset: props.alignOffset,
        avoidCollisions: props.avoidCollisions,
        collisionPadding: props.collisionPadding,
        strategy: "fixed",
        collisionBoundary: "clippingAncestors",
        excludeAnchorTranslation: props.excludeAnchorTranslation
      });
    };
    const focusTrigger = (run2, reason) => {
      const trigger = run2.anatomy.partsOf(DROPDOWN_FAMILY, "trigger")[0] ?? null;
      const focusSelf = trigger?.getExpose("focusSelf");
      focusSelf?.({ reason });
    };
    def2.props.watch([
      "side",
      "align",
      "sideOffset",
      "alignOffset",
      "avoidCollisions",
      "collisionPadding",
      "excludeAnchorTranslation"
    ], (run2) => syncPosition(run2));
    const updateOpen = (run2, ctx, reason) => {
      const wasOpen = open.get();
      const menuStillOwnsFocus = wasOpen && getNavigationEntries(run2).some((entry) => entry.focused);
      currentContext = ctx;
      contentId.set(createDropdownContentId(ctx.rootId), "reason: dropdown content identity sync");
      open.set(ctx.open, reason);
      if (ctx.open) {
        overlay2.openOverlay(reason);
        if (!wasOpen) {
          focusScope.activate({ reason: ctx.requestFocusReason ?? "programmatic" });
          resolveOpenFocusAction(run2, ctx);
        }
        return;
      }
      if (wasOpen)
        focusScope.deactivate({ reason: ctx.requestFocusReason ?? "programmatic" });
      overlay2.close(reason);
      if (wasOpen && (ctx.requestReason === "escape" || ctx.requestReason === "item.select" && menuStillOwnsFocus)) {
        focusTrigger(run2, ctx.requestFocusReason ?? "programmatic");
      }
    };
    def2.context.subscribe(DROPDOWN_CONTEXT, (run2, next) => {
      updateOpen(run2, next, "reason: dropdown context sync => content open");
    });
    def2.lifecycle.onCreated((run2) => {
      const ctx = readContext(run2);
      if (!ctx)
        return;
      currentContext = ctx;
      contentId.set(createDropdownContentId(ctx.rootId), "reason: dropdown content identity init");
      syncPosition(run2);
      updateOpen(run2, ctx, "reason: lifecycle.onCreated => dropdown content sync");
    });
    def2.lifecycle.onMounted((run2) => {
      store.run = run2;
      const trigger = run2.anatomy.partsOf(DROPDOWN_FAMILY, "trigger")[0] ?? null;
      if (trigger)
        overlay2.registerAnchorPart(trigger);
      syncPosition(run2);
      const ctx = run2.context.read(DROPDOWN_CONTEXT);
      const replayDeferredEntry = ctx.open && open.get();
      updateOpen(run2, ctx, "reason: lifecycle.onMounted => dropdown content sync");
      if (replayDeferredEntry)
        resolveOpenFocusAction(run2, ctx);
    });
    overlay2.open.watch((_ctx, event2) => {
      if (event2.type !== "next" || event2.next || event2.reason !== "escape")
        return;
      const run2 = store.run;
      const ctx = currentContext;
      if (!run2 || !ctx?.open)
        return;
      requestDropdownOpen(run2, false, "escape", "keyboard");
      if (ctx.controlled)
        overlay2.openOverlay("controlled.sync");
    });
    def2.event.onGlobal("key.down", (run2, ev) => {
      if (store.run !== run2)
        return;
      const ctx = readContext(run2);
      if (!ctx)
        return;
      if (!ctx.open || ctx.disabled)
        return;
      const key = ev?.key;
      if (key !== "Tab")
        return;
      if (!getNavigationEntries(run2).some((entry) => entry.focused))
        return;
      requestDropdownOpen(run2, false, "tab", "keyboard");
    });
    boundary2.subscribeOutside(() => {
      const run2 = store.run;
      const ctx = currentContext;
      if (!run2 || !ctx?.open || ctx.disabled)
        return;
      requestDropdownOpen(run2, false, "outside.press", "pointer");
    });
    def2.lifecycle.onUnmounted(() => {
      store.run = null;
      currentContext = null;
    });
    def2.rule({
      when: (w) => w.state(transition.isPresent).eq(false),
      intent: (i) => i.feedback.style.use(tw("hidden"))
    });
  }
  var asDropdownContent = defineAsHook({
    name: "as-dropdown-content",
    setup: setupDropdownContent,
    projectHandle: projectDropdownContentHandle
  });
  var dropdownContent = definePrototype({
    name: "base-dropdown-content",
    setup(def2) {
      setupDropdownContent(def2);
      def2.feedback.style.use(tw("absolute z-40"));
    }
  });
  // ../packages/prototypes/base/src/dropdown/item.proto.ts
  function setupDropdownItem(def2) {
    const command = setupDropdownCommand(def2, "dropdown item", { focusableWhenDisabled: true });
    const active = def2.state.bool("active", false);
    const collectionItem = asCollectionItem();
    collectionItem.configure({
      family: DROPDOWN_FAMILY,
      getMeta: (run2) => {
        const props = run2.props.get();
        return {
          value: props.value ?? "",
          textValue: props.textValue ?? "",
          disabled: !!props.disabled
        };
      }
    });
    def2.props.define({
      disabled: { type: "boolean", empty: "fallback" },
      value: { type: "string", empty: "fallback" },
      textValue: { type: "string", empty: "fallback" },
      closeOnCommit: { type: "boolean", empty: "fallback" }
    });
    def2.props.setDefaults({ disabled: false, value: "", textValue: "" });
    def2.a11y.role("menuitem");
    def2.a11y.nameFromContent();
    def2.a11y.state("disabled", command.disabled);
    def2.a11y.action("activate", { event: "select" });
    def2.expose.state("active", active);
    def2.expose.event("select", { payload: "json" });
    const syncDisabled = (run2) => {
      const ctx = run2.context.read(DROPDOWN_CONTEXT);
      command.syncDisabled(!!run2.props.get().disabled || ctx.disabled);
    };
    const syncActive = (ctx, ownValue) => {
      const nextActive = ctx.open !== false && (command.focused.get() || !!ownValue && ownValue === (ctx.activeValue ?? ""));
      active.set(nextActive, "reason: dropdown active sync");
      command.setRovingStatus({ active: nextActive });
    };
    def2.context.subscribe(DROPDOWN_CONTEXT, (run2, next) => {
      syncDisabled(run2);
      syncActive(next, run2.props.get().value ?? "");
    });
    def2.lifecycle.onMounted((run2) => {
      syncDisabled(run2);
      const currentRun = run2;
      syncActive(currentRun.context.read(DROPDOWN_CONTEXT), currentRun.props.get().value ?? "");
    });
    def2.props.watch(["value", "disabled"], (run2, next) => {
      syncDisabled(run2);
      syncActive(run2.context.read(DROPDOWN_CONTEXT), next.value ?? "");
    });
    const updateActiveValue = (run2) => {
      const ownValue = run2.props.get().value ?? "";
      if (!ownValue)
        return;
      active.set(true, "reason: dropdown item interaction => active");
      command.setRovingStatus({ active: true });
      run2.context.update(DROPDOWN_CONTEXT, (prev) => prev.activeValue === ownValue ? prev : { ...prev, activeValue: ownValue });
    };
    def2.event.on("press.commit", (run2, ev) => {
      if (command.disabled.get())
        return;
      const ctx = run2.context.read(DROPDOWN_CONTEXT);
      const reason = ev?.key ? "keyboard" : "pointer";
      const value = run2.props.get().value ?? "";
      updateActiveValue(run2);
      run2.expose.emit("select", { value, reason });
      const closeOnCommit = run2.props.isProvided("closeOnCommit") ? !!run2.props.get().closeOnCommit : ctx.closeOnItemCommit;
      if (closeOnCommit)
        requestDropdownOpen(run2, false, "item.select", reason);
    });
    command.focused.watch((run2, event2) => {
      if (event2.type !== "next")
        return;
      if (event2.next) {
        updateActiveValue(run2);
        return;
      }
      const currentRun = run2;
      syncActive(currentRun.context.read(DROPDOWN_CONTEXT), currentRun.props.get().value ?? "");
    });
    def2.event.on("pointer.enter", (run2) => {
      if (command.disabled.get())
        return;
      if (!run2.context.read(DROPDOWN_CONTEXT).open)
        return;
      updateActiveValue(run2);
    });
  }
  var asDropdownItem = defineAsHook({
    name: "as-dropdown-item",
    setup: setupDropdownItem
  });
  var dropdownItem = definePrototype({
    name: "base-dropdown-item",
    setup: setupDropdownItem
  });
  // ../packages/prototypes/shadcn/src/dropdown/content.proto.ts
  var dropdownContent2 = definePrototype({
    name: "shadcn-dropdown-content",
    setup(def2) {
      const dropdown = asDropdownContent();
      dropdown.asTransition.configure({ enterDuration: 150, leaveDuration: 100 });
      const { open } = dropdown.stateHandles;
      const { transitionState } = dropdown.asTransition;
      def2.feedback.style.use(tw("z-50 max-h-[var(--proto-ui-available-height)] min-w-32 overflow-x-hidden overflow-y-auto rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md outline-none transition-none duration-150"));
      def2.rule({
        when: (w) => w.any(w.state(transitionState).eq("entering"), w.state(transitionState).eq("entered")),
        intent: (i) => i.feedback.style.use(tw("animate-in fade-in-0 zoom-in-95"))
      });
      def2.rule({
        when: (w) => w.state(open).eq(false),
        intent: (i) => i.feedback.style.use(tw("animate-out fade-out-0 zoom-out-95"))
      });
      def2.rule({
        when: (w) => w.all(w.state(open).eq(true), w.prop("side").eq("bottom")),
        intent: (i) => i.feedback.style.use(tw("slide-in-from-top-2"))
      });
      def2.rule({
        when: (w) => w.all(w.state(open).eq(true), w.prop("side").eq("top")),
        intent: (i) => i.feedback.style.use(tw("slide-in-from-bottom-2"))
      });
      def2.rule({
        when: (w) => w.all(w.state(open).eq(true), w.prop("side").eq("left")),
        intent: (i) => i.feedback.style.use(tw("slide-in-from-right-2"))
      });
      def2.rule({
        when: (w) => w.all(w.state(open).eq(true), w.prop("side").eq("right")),
        intent: (i) => i.feedback.style.use(tw("slide-in-from-left-2"))
      });
    }
  });
  var content_proto_default6 = dropdownContent2;

  // ../packages/prototypes/shadcn/src/dropdown/item.proto.ts
  var ITEM_BASE_TOKENS = "relative flex w-full cursor-default select-none items-center gap-2 rounded-sm px-2 py-1.5 text-left text-sm outline-none transition-colors";
  var dropdownItem2 = definePrototype({
    name: "shadcn-dropdown-item",
    setup(def2) {
      def2.props.define({
        inset: { type: "boolean", empty: "fallback" },
        variant: { type: "enum", empty: "fallback", options: ["default", "destructive"] }
      });
      def2.props.setDefaults({ inset: false, variant: "default" });
      const itemState = asDropdownItem().stateHandles;
      if (!itemState) {
        throw new Error("[shadcn-dropdown-item] Dropdown Item must project command states.");
      }
      const { disabled, hovered, focused, focusVisible, pressed, active } = itemState;
      def2.feedback.style.use(tw(ITEM_BASE_TOKENS));
      def2.rule({
        when: (w) => w.prop("inset").eq(true),
        intent: (i) => i.feedback.style.use(tw("pl-8"))
      });
      def2.rule({
        when: (w) => w.prop("variant").eq("destructive"),
        intent: (i) => i.feedback.style.use(tw("text-destructive"))
      });
      def2.rule({
        when: (w) => w.any(w.state(active).eq(true), w.state(hovered).eq(true), w.state(focused).eq(true), w.state(focusVisible).eq(true)),
        intent: (i) => i.feedback.style.use(tw("bg-accent text-accent-foreground"))
      });
      def2.rule({
        when: (w) => w.all(w.state(active).eq(true), w.prop("variant").eq("destructive")),
        intent: (i) => i.feedback.style.use(tw("bg-destructive/10 text-destructive"))
      });
      def2.rule({
        when: (w) => w.state(pressed).eq(true),
        intent: (i) => i.feedback.style.use(tw("bg-accent/80"))
      });
      def2.rule({
        when: (w) => w.state(disabled).eq(true),
        intent: (i) => i.feedback.style.use(tw("pointer-events-none opacity-50"))
      });
    }
  });
  var item_proto_default2 = dropdownItem2;

  // ../packages/prototypes/shadcn/src/dropdown/root.proto.ts
  var dropdownRoot2 = definePrototype({
    name: "shadcn-dropdown-root",
    setup(def2) {
      asDropdownRoot();
    }
  });
  var root_proto_default11 = dropdownRoot2;

  // ../packages/prototypes/shadcn/src/dropdown/trigger.proto.ts
  var TRIGGER_BASE_TOKENS2 = [
    "inline-flex",
    "items-center",
    "justify-center",
    "gap-2",
    "rounded-md",
    "border",
    "border-border/60",
    "bg-background",
    "px-3",
    "py-2",
    "text-sm",
    "font-medium",
    "shadow-xs",
    "transition-colors",
    "outline-none",
    "select-none"
  ].join(" ");
  var DEFAULT_INDICATOR_ICON = "chevron-down";
  var DEFAULT_INDICATOR_SIZE = 16;
  var DEFAULT_INDICATOR_STROKE_WIDTH = 2;
  var INDICATOR_ICON_SHAPES = {
    "chevron-down": (svg) => svg.path({ d: "m6 9 6 6 6-6" }),
    "chevrons-up-down": (svg) => [svg.path({ d: "m7 15 5 5 5-5" }), svg.path({ d: "m7 9 5-5 5 5" })]
  };
  function toPositiveNumber(value, fallback) {
    if (typeof value !== "number")
      return fallback;
    if (!Number.isFinite(value))
      return fallback;
    if (value <= 0)
      return fallback;
    return value;
  }
  function renderIndicatorIcon(renderer, options) {
    return renderer.svg.root({
      viewBox: "0 0 24 24",
      width: options.size,
      height: options.size,
      fill: "none",
      stroke: "currentColor",
      strokeWidth: options.strokeWidth,
      strokeLinecap: "round",
      strokeLinejoin: "round"
    }, INDICATOR_ICON_SHAPES[options.icon](renderer.svg));
  }
  var dropdownTrigger2 = definePrototype({
    name: "shadcn-dropdown-trigger",
    setup(def2) {
      def2.props.define({
        disabled: { type: "boolean", empty: "fallback" },
        indicator: { type: "boolean", empty: "fallback" },
        indicatorIcon: {
          type: "enum",
          empty: "fallback",
          options: ["chevron-down", "chevrons-up-down"]
        },
        indicatorSize: { type: "number", empty: "fallback" },
        indicatorStrokeWidth: { type: "number", empty: "fallback" }
      });
      def2.props.setDefaults({
        disabled: false,
        indicator: false,
        indicatorIcon: DEFAULT_INDICATOR_ICON,
        indicatorSize: DEFAULT_INDICATOR_SIZE,
        indicatorStrokeWidth: DEFAULT_INDICATOR_STROKE_WIDTH
      });
      const buttonState = asDropdownTrigger().stateHandles;
      if (!buttonState) {
        throw new Error("[shadcn-dropdown-trigger] Dropdown Trigger must project command states.");
      }
      const { disabled, hovered, focusVisible, pressed } = buttonState;
      def2.feedback.style.use(tw(TRIGGER_BASE_TOKENS2));
      def2.rule({
        when: (w) => w.state(hovered).eq(true),
        intent: (i) => i.feedback.style.use(tw("bg-muted text-foreground"))
      });
      def2.rule({
        when: (w) => w.state(focusVisible).eq(true),
        intent: (i) => i.feedback.style.use(tw("ring-3 ring-ring/50 ring-offset-2"))
      });
      def2.rule({
        when: (w) => w.state(pressed).eq(true),
        intent: (i) => i.feedback.style.use(tw("bg-muted text-foreground"))
      });
      def2.rule({
        when: (w) => w.state(disabled).eq(true),
        intent: (i) => i.feedback.style.use(tw("pointer-events-none opacity-50"))
      });
      return (renderer) => {
        const props = renderer.read.props.get();
        const indicatorEnabled = props.indicator !== false;
        const indicatorIcon = props.indicatorIcon === "chevrons-up-down" ? props.indicatorIcon : DEFAULT_INDICATOR_ICON;
        const indicatorSize = toPositiveNumber(props.indicatorSize, DEFAULT_INDICATOR_SIZE);
        const indicatorStrokeWidth = toPositiveNumber(props.indicatorStrokeWidth, DEFAULT_INDICATOR_STROKE_WIDTH);
        return [
          renderer.r.slot(),
          indicatorEnabled ? renderIndicatorIcon(renderer, {
            icon: indicatorIcon,
            size: indicatorSize,
            strokeWidth: indicatorStrokeWidth
          }) : null
        ];
      };
    }
  });
  var trigger_proto_default6 = dropdownTrigger2;

  // ../packages/prototypes/base/src/select/shared.ts
  var nextSelectRootId = 0;
  function createSelectRootId() {
    nextSelectRootId += 1;
    return `pui-select-${nextSelectRootId}`;
  }
  function createSelectContentId(rootId) {
    return `${rootId || "pui-select"}-content`;
  }
  function requestSelectOpen(run2, request) {
    try {
      const root = run2.anatomy.partsOf(SELECT_FAMILY, "root")[0] ?? null;
      const requestOpen = root?.getExpose("requestOpen");
      return requestOpen?.(request) ?? false;
    } catch (error5) {
      if (error5?.code === "CONTEXT_DISCONNECTED")
        return false;
      throw error5;
    }
  }
  function requestSelectValue(run2, request) {
    try {
      const root = run2.anatomy.partsOf(SELECT_FAMILY, "root")[0] ?? null;
      const requestValue = root?.getExpose("requestValue");
      return requestValue?.(request) ?? false;
    } catch (error5) {
      if (error5?.code === "CONTEXT_DISCONNECTED")
        return false;
      throw error5;
    }
  }
  function notifySelectItemSnapshotChanged(run2) {
    const root = run2.anatomy.partsOf(SELECT_FAMILY, "root")[0] ?? null;
    const refresh = root?.getExpose("__refreshSelectedText");
    refresh?.();
  }
  var SELECT_FAMILY = createAnatomyFamily("base-select", {
    roles: {
      root: { cardinality: { min: 1, max: 1 } },
      trigger: { cardinality: { min: 0, max: 1 } },
      value: { cardinality: { min: 0, max: 1 } },
      content: { cardinality: { min: 0, max: 1 } },
      item: { cardinality: { min: 0, max: 100 } }
    },
    relations: [
      { kind: "contains", parent: "root", child: "trigger" },
      { kind: "contains", parent: "root", child: "value" },
      { kind: "contains", parent: "root", child: "content" },
      { kind: "contains", parent: "content", child: "item" }
    ]
  });
  var SELECT_CONTEXT = createContextKey("base-select");

  // ../packages/prototypes/base/src/select/root.proto.ts
  function sameContext3(a, b) {
    return Object.keys(a).every((key) => a[key] === b[key]);
  }
  function setupSelectRoot(def2) {
    def2.anatomy.claim(SELECT_FAMILY, { role: "root" });
    const collection2 = asCollection();
    collection2.configure({ family: SELECT_FAMILY });
    def2.props.define({
      open: { type: "boolean", empty: "fallback" },
      defaultOpen: { type: "boolean", empty: "fallback" },
      value: { type: "string", empty: "fallback" },
      defaultValue: { type: "string", empty: "fallback" },
      disabled: { type: "boolean", empty: "fallback" },
      closeOnSelect: { type: "boolean", empty: "fallback" }
    });
    def2.props.setDefaults({
      defaultOpen: false,
      defaultValue: "",
      disabled: false,
      closeOnSelect: true
    });
    const initialContext = {
      rootId: "",
      open: false,
      controlledOpen: false,
      value: "",
      textValue: "",
      controlledValue: false,
      disabled: false,
      activeValue: "",
      closeOnSelect: true,
      requestReason: null,
      requestFocusReason: null,
      requestEntry: null
    };
    def2.context.provide(SELECT_CONTEXT, initialContext);
    let submitOpenRequest = (_run, _request) => false;
    const openState = useOpenState({
      requestOpen(run2, nextOpen, reason) {
        submitOpenRequest(run2, {
          open: nextOpen,
          reason,
          focusReason: "programmatic"
        });
      }
    });
    const open = openState.getState?.("open");
    const value = def2.state.string("value", "");
    const textValue = def2.state.string("textValue", "");
    def2.expose.state("value", value);
    def2.expose.state("textValue", textValue);
    def2.expose.event("openChange", { payload: "json" });
    def2.expose.event("valueChange", { payload: "json" });
    let snapshot = initialContext;
    let published = initialContext;
    let currentRun = null;
    const resolveTextValue = (run2, nextValue) => {
      if (!nextValue)
        return "";
      const itemSnapshot = collection2.getItems().find((item) => item.value === nextValue);
      return itemSnapshot ? String(itemSnapshot.textValue || itemSnapshot.value || "") : "";
    };
    const syncContext = (run2) => {
      const next = {
        ...snapshot,
        open: open?.get() ?? false,
        value: value.get(),
        textValue: textValue.get()
      };
      snapshot = next;
      if (sameContext3(published, next))
        return;
      published = next;
      run2.context.update(SELECT_CONTEXT, next);
    };
    const refreshSelectedText = (run2) => {
      const selectedValue = value.get();
      const nextTextValue = resolveTextValue(run2, selectedValue);
      if (selectedValue && !nextTextValue && !snapshot.open) {
        syncContext(run2);
        return;
      }
      textValue.set(nextTextValue, "reason: select root derive selected text");
      syncContext(run2);
    };
    def2.anatomy.subscribeParts(SELECT_FAMILY, "item", (run2) => {
      refreshSelectedText(run2);
    });
    def2.context.subscribe(SELECT_CONTEXT, (_run, next) => {
      snapshot = next;
      published = next;
    });
    submitOpenRequest = (run2, request) => {
      if (snapshot.disabled)
        return false;
      snapshot = {
        ...snapshot,
        activeValue: request.open ? snapshot.value : "",
        requestReason: request.reason,
        requestFocusReason: request.focusReason,
        requestEntry: request.open ? request.entry ?? "selected-or-first" : null
      };
      if (!snapshot.controlledOpen) {
        open?.set(request.open, "reason: select root accepted open request");
      }
      syncContext(run2);
      run2.expose.emit("openChange", {
        open: request.open,
        reason: request.reason,
        focusReason: request.focusReason
      });
      return true;
    };
    const submitValueRequest = (run2, request) => {
      if (snapshot.disabled)
        return false;
      snapshot = {
        ...snapshot,
        activeValue: request.value,
        requestReason: "item.select",
        requestFocusReason: request.reason
      };
      if (!snapshot.controlledValue) {
        value.set(request.value, "reason: select root accepted value request");
        textValue.set(request.textValue || resolveTextValue(run2, request.value), "reason: select root accepted selected text");
      }
      syncContext(run2);
      run2.expose.emit("valueChange", request);
      return true;
    };
    def2.expose.method("requestOpen", (request) => {
      if (!currentRun)
        return false;
      return submitOpenRequest(currentRun, request);
    });
    def2.expose.method("requestValue", (request) => {
      if (!currentRun)
        return false;
      return submitValueRequest(currentRun, request);
    });
    def2.expose.method("__refreshSelectedText", () => {
      if (currentRun)
        refreshSelectedText(currentRun);
    });
    def2.lifecycle.onCreated((run2) => {
      currentRun = run2;
      const props = run2.props.get();
      const controlledValue = run2.props.isProvided("value");
      value.set(controlledValue ? props.value ?? "" : props.defaultValue ?? "", "reason: select root initialize value");
      snapshot = {
        ...snapshot,
        rootId: createSelectRootId(),
        controlledOpen: run2.props.isProvided("open"),
        controlledValue,
        disabled: !!props.disabled,
        closeOnSelect: props.closeOnSelect !== false
      };
      syncContext(run2);
    });
    def2.lifecycle.onMounted((run2) => {
      currentRun = run2;
      refreshSelectedText(run2);
    });
    def2.lifecycle.onUnmounted(() => {
      currentRun = null;
    });
    def2.props.watch(["value", "disabled", "closeOnSelect"], (run2, next) => {
      const controlledValue = run2.props.isProvided("value");
      snapshot = {
        ...snapshot,
        controlledOpen: run2.props.isProvided("open"),
        controlledValue,
        disabled: !!next.disabled,
        closeOnSelect: next.closeOnSelect !== false
      };
      if (controlledValue) {
        value.set(next.value ?? "", "reason: select root controlled value sync");
        textValue.set(resolveTextValue(run2, next.value ?? ""), "reason: select root controlled selected text sync");
      }
      syncContext(run2);
    });
    open?.watch((run2, event2) => {
      if (event2.type !== "next")
        return;
      snapshot = {
        ...snapshot,
        controlledOpen: run2.props.isProvided("open"),
        activeValue: event2.next ? value.get() : "",
        requestEntry: event2.next ? snapshot.requestEntry : null
      };
      syncContext(run2);
    });
  }
  var asSelectRoot = defineAsHook({
    name: "as-select-root",
    setup: setupSelectRoot
  });
  var selectRoot = definePrototype({ name: "base-select-root", setup: setupSelectRoot });
  // ../packages/prototypes/base/src/select/command.ts
  function setupSelectCommand(def2, reasonPrefix) {
    asTrigger();
    def2.props.define({ disabled: { type: "boolean", empty: "fallback" } });
    def2.props.setDefaults({ disabled: false });
    const disabled = def2.state.bool("disabled", false);
    const hovered = def2.state.bool("hovered", false);
    const pressed = def2.state.bool("pressed", false);
    const focusable = asFocusable();
    focusable.configure({ disabled: false });
    const focused = focusable.focused;
    const focusVisible = focusable.focusVisible;
    def2.expose.state("disabled", disabled);
    def2.expose.state("hovered", hovered);
    def2.expose.state("focused", focused);
    def2.expose.state("focusVisible", focusVisible);
    def2.expose.state("pressed", pressed);
    const focusSelf = (options) => {
      if (!disabled.get())
        focusable.focusSelf(options);
    };
    def2.expose.method("focusSelf", focusSelf);
    const clearTransient = (reason) => {
      hovered.set(false, reason);
      pressed.set(false, reason);
    };
    const syncDisabled = (nextDisabled) => {
      disabled.set(nextDisabled, `reason: ${reasonPrefix} disabled sync`);
      focusable.setDisabled(nextDisabled);
      if (nextDisabled)
        clearTransient(`reason: ${reasonPrefix} disabled => reset interaction`);
    };
    const resetInteraction = (reason, options) => {
      clearTransient(reason);
      if (options?.blur)
        focusable.blur();
    };
    def2.event.on("key.down", (_run, ev) => {
      if (disabled.get() || !focused.get() || ev?.key !== " ")
        return;
      ev.control.requestDefaultActionPrevention({
        reason: `${reasonPrefix}.space-activation`,
        source: reasonPrefix
      });
    });
    def2.event.on("pointer.enter", () => {
      if (!disabled.get())
        hovered.set(true, `reason: ${reasonPrefix} pointer.enter`);
    });
    def2.event.on("pointer.leave", () => clearTransient(`reason: ${reasonPrefix} pointer.leave`));
    def2.event.on("pointer.cancel", () => clearTransient(`reason: ${reasonPrefix} pointer.cancel`));
    def2.event.on("pointer.down", () => {
      if (!disabled.get())
        pressed.set(true, `reason: ${reasonPrefix} pointer.down`);
    });
    def2.event.on("pointer.up", () => pressed.set(false, `reason: ${reasonPrefix} pointer.up`));
    def2.event.on("press.commit", () => pressed.set(false, `reason: ${reasonPrefix} press.commit`));
    return {
      disabled,
      hovered,
      focused,
      focusVisible,
      pressed,
      focusSelf,
      setRovingStatus: (status) => focusable.setRovingStatus(status),
      resetInteraction,
      syncDisabled
    };
  }

  // ../packages/prototypes/base/src/select/trigger.proto.ts
  function setupSelectTrigger(def2) {
    def2.anatomy.claim(SELECT_FAMILY, { role: "trigger" });
    const command = setupSelectCommand(def2, "select trigger");
    const expanded = def2.state.bool("selectExpanded", false);
    const hasPopup = def2.state.string("selectHasPopup", "listbox");
    const controls = def2.state.string("selectContentId", "");
    const placeholder = def2.state.bool("placeholder", true);
    def2.expose.state("placeholder", placeholder);
    def2.a11y.role("combobox");
    def2.a11y.nameFromContent();
    def2.a11y.state("disabled", command.disabled);
    def2.a11y.state("expanded", expanded);
    def2.a11y.state("hasPopup", hasPopup);
    def2.a11y.relation("controls", { target: controls });
    def2.a11y.action("activate", { event: "click" });
    const sync = (run2, ctx) => {
      command.syncDisabled(!!run2.props.get().disabled || ctx.disabled);
      expanded.set(ctx.open, "reason: select trigger expanded sync");
      placeholder.set(!ctx.value, "reason: select trigger placeholder sync");
      controls.set(createSelectContentId(ctx.rootId), "reason: select trigger controls sync");
    };
    def2.context.subscribe(SELECT_CONTEXT, (run2, next) => sync(run2, next));
    def2.lifecycle.onCreated((run2) => sync(run2, run2.context.read(SELECT_CONTEXT)));
    def2.props.watch(["disabled"], (run2) => sync(run2, run2.context.read(SELECT_CONTEXT)));
    def2.event.on("press.commit", (run2, ev) => {
      if (command.disabled.get())
        return;
      const ctx = run2.context.read(SELECT_CONTEXT);
      const key = ev?.key;
      if (key === "Enter" || key === " ") {
        requestSelectOpen(run2, {
          open: true,
          reason: "trigger.press",
          focusReason: "keyboard",
          entry: "selected-or-first"
        });
        return;
      }
      requestSelectOpen(run2, {
        open: !ctx.open,
        reason: "trigger.press",
        focusReason: "pointer",
        entry: "selected-or-first"
      });
    });
    def2.event.on("key.down", (run2, ev) => {
      if (command.disabled.get())
        return;
      const key = ev?.key;
      if (key !== "ArrowDown" && key !== "ArrowUp")
        return;
      ev.control.requestDefaultActionPrevention({
        reason: "select.arrow-open",
        source: "base-select-trigger"
      });
      const ctx = run2.context.read(SELECT_CONTEXT);
      if (ctx.open)
        return;
      requestSelectOpen(run2, {
        open: true,
        reason: `trigger.${key}`,
        focusReason: "keyboard",
        entry: key === "ArrowUp" ? "selected-or-last" : "selected-or-first"
      });
    });
  }
  var asSelectTrigger = defineAsHook({
    name: "as-select-trigger",
    setup: setupSelectTrigger
  });
  var selectTrigger = definePrototype({ name: "base-select-trigger", setup: setupSelectTrigger });
  // ../packages/prototypes/base/src/select/value.proto.ts
  function setupSelectValue(def2) {
    def2.anatomy.claim(SELECT_FAMILY, { role: "value" });
    def2.props.define({ placeholder: { type: "string", empty: "fallback" } });
    def2.props.setDefaults({ placeholder: "" });
    def2.feedback.style.use(tw("pointer-events-none"));
    const displayValue = def2.state.string("displayValue", "");
    def2.expose.state("displayValue", displayValue);
    let mounted = false;
    let renderMissed = false;
    const computeDisplayValue = (run2) => {
      const ctx = run2.context.read(SELECT_CONTEXT);
      return ctx.textValue || ctx.value || run2.props.get().placeholder || "";
    };
    const syncDisplayValue = (run2, requestRender) => {
      const nextValue = computeDisplayValue(run2);
      if (nextValue === displayValue.get())
        return;
      displayValue.set(nextValue, "reason: select value display sync");
      if (!requestRender)
        return;
      if (mounted)
        run2.update();
      else
        renderMissed = true;
    };
    def2.context.subscribe(SELECT_CONTEXT, (run2) => syncDisplayValue(run2, true));
    def2.lifecycle.onCreated((run2) => syncDisplayValue(run2, false));
    def2.lifecycle.onMounted((run2) => {
      syncDisplayValue(run2, false);
      mounted = true;
      if (renderMissed) {
        renderMissed = false;
        run2.update();
      }
    });
    def2.props.watch(["placeholder"], (run2) => syncDisplayValue(run2, true));
    def2.lifecycle.onUnmounted(() => {
      mounted = false;
    });
    return () => displayValue.get() ? [displayValue.get()] : null;
  }
  var asSelectValue = defineAsHook({
    name: "as-select-value",
    setup: setupSelectValue
  });
  var selectValue = definePrototype({ name: "base-select-value", setup: setupSelectValue });
  // ../packages/prototypes/base/src/select/content.proto.ts
  function projectSelectContentHandle(result) {
    const open = result.getState?.("open");
    const asTransition2 = result.getAsHookHandle?.("asTransition");
    if (!open || !asTransition2) {
      throw new Error("[as-select-content] missing captured Select or Transition handles.");
    }
    return { stateHandles: { open }, asTransition: asTransition2 };
  }
  function setupSelectContent(def2, _options, api) {
    def2.anatomy.claim(SELECT_FAMILY, { role: "content" });
    def2.props.define({
      side: { type: "enum", empty: "fallback", options: ["top", "right", "bottom", "left"] },
      align: { type: "enum", empty: "fallback", options: ["start", "center", "end"] },
      sideOffset: { type: "number", empty: "fallback" },
      alignOffset: { type: "number", empty: "fallback" },
      avoidCollisions: { type: "boolean", empty: "fallback" },
      collisionPadding: { type: "number", empty: "fallback" }
    });
    def2.props.setDefaults({
      side: "bottom",
      align: "center",
      sideOffset: 4,
      alignOffset: 0,
      avoidCollisions: true,
      collisionPadding: 10
    });
    const contentId = def2.state.string("selectContentId", "");
    const orientation = def2.state.string("selectOrientation", "vertical");
    def2.a11y.id(contentId);
    def2.a11y.role("listbox");
    def2.a11y.state("orientation", orientation);
    const focusScope = asFocusScope();
    focusScope.configure({ entry: "manual", restore: "none" });
    const focusRoving = asFocusRoving();
    focusRoving.configure({ navigation: "arrow", orientation: "vertical", entry: "manual" });
    def2.expose.method("focusFirst", () => focusRoving.focusFirst());
    def2.expose.method("focusLast", () => focusRoving.focusLast());
    def2.expose.method("focusNext", () => focusRoving.focusNext());
    def2.expose.method("focusPrev", () => focusRoving.focusPrev());
    def2.expose.method("focusSelected", () => focusRoving.focusSelected());
    const overlay2 = asOverlay();
    overlay2.configure({
      closeOnEscape: true,
      closeOnOutsidePress: false,
      closeOnFocusOutside: false,
      restore: "none",
      entry: "manual",
      placement: "bottom",
      align: "center",
      sideOffset: 4,
      alignOffset: 0,
      anchored: true,
      strategy: "fixed",
      avoidCollisions: true,
      collisionBoundary: "clippingAncestors",
      collisionPadding: 10,
      portal: true,
      modal: false,
      layerRole: "select-content",
      meta: { overlayKind: "select" }
    });
    const boundary2 = asBoundary();
    boundary2.observe("pointer.press");
    const transition = asTransition();
    transition.configure({ enterDuration: 0, leaveDuration: 0 });
    overlay2.bindPresence({
      enter: transition.controls.enter,
      leave: transition.controls.leave,
      present: transition.isPresent
    });
    const open = def2.state.bool("open", false);
    def2.expose.state("open", open);
    const store = api?.store ?? {};
    store.run = null;
    let currentContext = null;
    let entryTask = null;
    const readContext = (run2) => {
      try {
        return run2.context.read(SELECT_CONTEXT);
      } catch (error5) {
        if (error5?.code === "CONTEXT_DISCONNECTED")
          return null;
        throw error5;
      }
    };
    const getNavigationEntries = (run2) => run2.anatomy.order.partsOf(SELECT_FAMILY, "item").map((item) => ({
      snapshot: item.getExpose("getCollectionItem")?.(),
      focusSelf: item.getExpose("focusSelf"),
      focused: !!item.getExpose("focused")?.get?.()
    })).filter((entry) => entry.snapshot?.disabled !== true);
    useTypeaheadNavigation({
      isEnabled: (run2) => {
        const ctx = readContext(run2);
        return !!ctx?.open && !ctx.disabled && getNavigationEntries(run2).some((entry) => entry.focused);
      },
      getEntries: (run2) => getNavigationEntries(run2).filter((entry) => !!entry.snapshot && !!entry.focusSelf),
      getCurrentIndex: (_run, entries) => entries.findIndex((entry) => entry.focused),
      getText: (entry) => String(entry.snapshot?.textValue || entry.snapshot?.value || ""),
      onMatch: (_run, entry) => entry.focusSelf?.({ reason: "keyboard", preventScroll: true })
    });
    const focusValue = (run2, candidateValue, options) => {
      if (!candidateValue)
        return false;
      const entry = getNavigationEntries(run2).find((candidate) => String(candidate.snapshot?.value ?? "") === candidateValue);
      if (!entry?.focusSelf)
        return false;
      entry.focusSelf(options);
      return true;
    };
    const resolveOpenFocusAction = (run2, ctx) => {
      const options = {
        defer: true,
        preventScroll: true,
        reason: ctx.requestFocusReason ?? "programmatic"
      };
      if (focusValue(run2, ctx.value, options))
        return;
      if (ctx.requestEntry === "selected-or-last") {
        focusRoving.focusLast(options);
        return;
      }
      focusRoving.focusFirst(options);
    };
    const scheduleOpenFocusAction = (run2) => {
      entryTask?.cancel();
      entryTask = delay(0, () => {
        entryTask = null;
        const ctx = readContext(run2);
        if (!ctx?.open || !open.get())
          return;
        resolveOpenFocusAction(run2, ctx);
      });
    };
    const syncPosition = (run2) => {
      const props = run2.props.get();
      overlay2.updatePosition({
        placement: props.side,
        align: props.align,
        sideOffset: props.sideOffset,
        alignOffset: props.alignOffset,
        avoidCollisions: props.avoidCollisions,
        collisionPadding: props.collisionPadding,
        strategy: "fixed",
        collisionBoundary: "clippingAncestors"
      });
    };
    const focusTrigger = (run2, reason) => {
      const trigger = run2.anatomy.partsOf(SELECT_FAMILY, "trigger")[0] ?? null;
      const focusSelf = trigger?.getExpose("focusSelf");
      focusSelf?.({ reason });
    };
    def2.props.watch(["side", "align", "sideOffset", "alignOffset", "avoidCollisions", "collisionPadding"], (run2) => syncPosition(run2));
    const updateOpen = (run2, ctx, reason) => {
      const wasOpen = open.get();
      currentContext = ctx;
      contentId.set(createSelectContentId(ctx.rootId), "reason: select content identity sync");
      open.set(ctx.open, reason);
      if (ctx.open) {
        overlay2.openOverlay(reason);
        if (!wasOpen) {
          focusScope.activate({ reason: ctx.requestFocusReason ?? "programmatic" });
          scheduleOpenFocusAction(run2);
        }
        return;
      }
      entryTask?.cancel();
      entryTask = null;
      if (wasOpen)
        focusScope.deactivate({ reason: ctx.requestFocusReason ?? "programmatic" });
      overlay2.close(reason);
      if (wasOpen && (ctx.requestReason === "escape" || ctx.requestReason === "item.select")) {
        focusTrigger(run2, ctx.requestFocusReason ?? "programmatic");
      }
    };
    def2.context.subscribe(SELECT_CONTEXT, (run2, next) => {
      updateOpen(run2, next, "reason: select context sync => content open");
    });
    def2.lifecycle.onCreated((run2) => {
      const ctx = readContext(run2);
      if (!ctx)
        return;
      currentContext = ctx;
      contentId.set(createSelectContentId(ctx.rootId), "reason: select content identity init");
      syncPosition(run2);
      updateOpen(run2, ctx, "reason: lifecycle.onCreated => select content sync");
    });
    def2.lifecycle.onMounted((run2) => {
      store.run = run2;
      const trigger = run2.anatomy.partsOf(SELECT_FAMILY, "trigger")[0] ?? null;
      if (trigger)
        overlay2.registerAnchorPart(trigger);
      syncPosition(run2);
      const ctx = run2.context.read(SELECT_CONTEXT);
      const replayDeferredEntry = ctx.open && open.get();
      updateOpen(run2, ctx, "reason: lifecycle.onMounted => select content sync");
      if (replayDeferredEntry)
        scheduleOpenFocusAction(run2);
    });
    def2.anatomy.subscribeParts(SELECT_FAMILY, "item", (run2) => {
      if (entryTask && open.get())
        scheduleOpenFocusAction(run2);
    });
    overlay2.open.watch((_ctx, event2) => {
      if (event2.type !== "next" || event2.next || event2.reason !== "escape")
        return;
      const run2 = store.run;
      const ctx = currentContext;
      if (!run2 || !ctx?.open)
        return;
      requestSelectOpen(run2, {
        open: false,
        reason: "escape",
        focusReason: "keyboard"
      });
      if (ctx.controlledOpen)
        overlay2.openOverlay("controlled.sync");
    });
    def2.event.onGlobal("key.down", (run2, ev) => {
      if (store.run !== run2)
        return;
      const ctx = readContext(run2);
      if (!ctx?.open || ctx.disabled || ev?.key !== "Tab")
        return;
      if (!getNavigationEntries(run2).some((entry) => entry.focused))
        return;
      requestSelectOpen(run2, { open: false, reason: "tab", focusReason: "keyboard" });
    });
    boundary2.subscribeOutside(() => {
      const run2 = store.run;
      const ctx = currentContext;
      if (!run2 || !ctx?.open || ctx.disabled)
        return;
      requestSelectOpen(run2, { open: false, reason: "outside.press", focusReason: "pointer" });
    });
    def2.lifecycle.onUnmounted(() => {
      entryTask?.cancel();
      entryTask = null;
      store.run = null;
      currentContext = null;
    });
    def2.rule({
      when: (w) => w.state(transition.isPresent).eq(false),
      intent: (i) => i.feedback.style.use(tw("hidden"))
    });
  }
  var asSelectContent = defineAsHook({
    name: "as-select-content",
    setup: setupSelectContent,
    projectHandle: projectSelectContentHandle
  });
  var selectContent = definePrototype({
    name: "base-select-content",
    setup(def2) {
      setupSelectContent(def2);
      def2.feedback.style.use(tw("absolute z-40"));
    }
  });
  // ../packages/prototypes/base/src/select/item.proto.ts
  function setupSelectItem(def2) {
    const command = setupSelectCommand(def2, "select item");
    const active = def2.state.bool("active", false);
    const selected = def2.state.fromAccessibility("selected");
    const collectionItem = asCollectionItem();
    collectionItem.configure({
      family: SELECT_FAMILY,
      getMeta: (run2) => {
        const props = run2.props.get();
        return {
          value: props.value ?? "",
          textValue: props.textValue ?? props.value ?? "",
          disabled: !!props.disabled
        };
      }
    });
    def2.props.define({
      disabled: { type: "boolean", empty: "fallback" },
      value: { type: "string", empty: "fallback" },
      textValue: { type: "string", empty: "fallback" },
      closeOnSelect: { type: "boolean", empty: "fallback" }
    });
    def2.props.setDefaults({ disabled: false, value: "", textValue: "" });
    def2.expose.state("active", active);
    def2.expose.state("selected", selected);
    def2.expose.event("select", { payload: "json" });
    def2.a11y.role("option");
    def2.a11y.nameFromContent();
    def2.a11y.state("selected", selected);
    def2.a11y.state("disabled", command.disabled);
    def2.a11y.action("activate", { event: "select" });
    const readContext = (run2) => {
      try {
        return run2.context.read(SELECT_CONTEXT);
      } catch (error5) {
        if (error5?.code === "CONTEXT_DISCONNECTED")
          return null;
        throw error5;
      }
    };
    const sync = (run2, ctx) => {
      const ownValue = run2.props.get().value ?? "";
      const nextDisabled = !!run2.props.get().disabled || ctx.disabled;
      command.syncDisabled(nextDisabled);
      if (!ctx.open) {
        command.resetInteraction("reason: select popup close => reset item interaction", {
          blur: true
        });
      }
      const nextSelected = !!ownValue && ownValue === ctx.value;
      const nextActive = ctx.open && !nextDisabled && (command.focused.get() || !!ownValue && ownValue === ctx.activeValue);
      selected.set(nextSelected, "reason: select item selected sync");
      active.set(nextActive, "reason: select item active sync");
      command.setRovingStatus({ selected: nextSelected, active: nextActive });
    };
    def2.context.subscribe(SELECT_CONTEXT, (run2, next) => sync(run2, next));
    def2.lifecycle.onMounted((run2) => {
      const ctx = readContext(run2);
      if (ctx)
        sync(run2, ctx);
      notifySelectItemSnapshotChanged(run2);
    });
    def2.props.watch(["value", "textValue", "disabled"], (run2) => {
      const ctx = readContext(run2);
      if (ctx)
        sync(run2, ctx);
      notifySelectItemSnapshotChanged(run2);
    });
    const updateActiveValue = (run2) => {
      if (command.disabled.get())
        return;
      const ownValue = run2.props.get().value ?? "";
      if (!ownValue)
        return;
      active.set(true, "reason: select item interaction => active");
      command.setRovingStatus({ active: true, selected: selected.get() });
      run2.context.update(SELECT_CONTEXT, (prev) => prev.activeValue === ownValue ? prev : { ...prev, activeValue: ownValue });
    };
    const clearTransientActive = (run2, reason) => {
      const ctx = readContext(run2);
      const ownValue = run2.props.get().value ?? "";
      active.set(false, reason);
      command.setRovingStatus({ active: false, selected: selected.get() });
      if (!ctx?.open || !ownValue || ctx.activeValue !== ownValue)
        return;
      run2.context.update(SELECT_CONTEXT, (prev) => prev.activeValue === ownValue ? { ...prev, activeValue: "" } : prev);
    };
    def2.event.on("press.commit", (run2, ev) => {
      if (command.disabled.get())
        return;
      const ctx = readContext(run2);
      if (!ctx)
        return;
      const reason = ev?.key ? "keyboard" : "pointer";
      const ownValue = run2.props.get().value ?? "";
      const ownTextValue = run2.props.get().textValue || ownValue;
      updateActiveValue(run2);
      run2.expose.emit("select", { value: ownValue, reason });
      requestSelectValue(run2, { value: ownValue, textValue: ownTextValue, reason });
      const closeOnSelect = run2.props.isProvided("closeOnSelect") ? !!run2.props.get().closeOnSelect : ctx.closeOnSelect;
      if (closeOnSelect) {
        requestSelectOpen(run2, { open: false, reason: "item.select", focusReason: reason });
      }
    });
    command.focused.watch((run2, event2) => {
      if (event2.type !== "next")
        return;
      if (event2.next) {
        updateActiveValue(run2);
        return;
      }
      if (!command.hovered.get()) {
        clearTransientActive(run2, "reason: select item blur => clear transient active");
        return;
      }
      const ctx = readContext(run2);
      if (ctx)
        sync(run2, ctx);
    });
    def2.event.on("pointer.enter", (run2) => {
      const ctx = readContext(run2);
      if (command.disabled.get() || !ctx?.open)
        return;
      updateActiveValue(run2);
    });
    def2.event.on("pointer.leave", (run2) => {
      if (command.focused.get())
        return;
      clearTransientActive(run2, "reason: select item pointer.leave => clear pointer active");
    });
  }
  var asSelectItem = defineAsHook({
    name: "as-select-item",
    setup: setupSelectItem
  });
  var selectItem = definePrototype({ name: "base-select-item", setup: setupSelectItem });
  // ../packages/prototypes/shadcn/src/select/root.proto.ts
  var selectRoot2 = definePrototype({
    name: "shadcn-select-root",
    setup() {
      asSelectRoot();
    }
  });
  var root_proto_default13 = selectRoot2;

  // ../packages/prototypes/shadcn/src/select/trigger.proto.ts
  function renderChevron(renderer) {
    return renderer.el("span", { style: tw("pointer-events-none flex shrink-0 items-center opacity-50") }, renderer.svg.root({
      viewBox: "0 0 24 24",
      width: 16,
      height: 16,
      fill: "none",
      stroke: "currentColor",
      strokeWidth: 2,
      strokeLinecap: "round",
      strokeLinejoin: "round"
    }, renderer.svg.path({ d: "m6 9 6 6 6-6" })));
  }
  var selectTrigger2 = definePrototype({
    name: "shadcn-select-trigger",
    setup(def2) {
      def2.props.define({
        size: { type: "enum", empty: "fallback", options: ["sm", "default"] }
      });
      def2.props.setDefaults({ size: "default" });
      const state2 = asSelectTrigger().stateHandles;
      if (!state2) {
        throw new Error("[shadcn-select-trigger] Select Trigger must project command states.");
      }
      const { disabled, hovered, focusVisible, pressed, placeholder } = state2;
      def2.feedback.style.use(tw("flex items-center justify-between gap-2 rounded-md border border-input bg-transparent px-3 py-2 text-sm whitespace-nowrap shadow-xs transition-colors outline-none select-none"));
      def2.rule({
        when: (w) => w.prop("size").eq("default"),
        intent: (i) => i.feedback.style.use(tw("h-9"))
      });
      def2.rule({
        when: (w) => w.prop("size").eq("sm"),
        intent: (i) => i.feedback.style.use(tw("h-8"))
      });
      def2.rule({
        when: (w) => w.state(placeholder).eq(true),
        intent: (i) => i.feedback.style.use(tw("text-muted-foreground"))
      });
      def2.rule({
        when: (w) => w.state(hovered).eq(true),
        intent: (i) => i.feedback.style.use(tw("bg-input/50"))
      });
      def2.rule({
        when: (w) => w.state(focusVisible).eq(true),
        intent: (i) => i.feedback.style.use(tw("border-ring ring-3 ring-ring/50"))
      });
      def2.rule({
        when: (w) => w.state(pressed).eq(true),
        intent: (i) => i.feedback.style.use(tw("bg-input/70"))
      });
      def2.rule({
        when: (w) => w.state(disabled).eq(true),
        intent: (i) => i.feedback.style.use(tw("pointer-events-none opacity-50"))
      });
      return (renderer) => [renderer.r.slot(), renderChevron(renderer)];
    }
  });
  var trigger_proto_default8 = selectTrigger2;

  // ../packages/prototypes/shadcn/src/select/value.proto.ts
  var selectValue2 = definePrototype({
    name: "shadcn-select-value",
    setup() {
      const value = asSelectValue().stateHandles;
      if (!value)
        throw new Error("[shadcn-select-value] Select Value must project displayValue.");
      return () => value.displayValue.get() ? [value.displayValue.get()] : null;
    }
  });
  var value_proto_default2 = selectValue2;

  // ../packages/prototypes/shadcn/src/select/content.proto.ts
  var selectContent2 = definePrototype({
    name: "shadcn-select-content",
    setup(def2) {
      def2.props.define({
        position: {
          type: "enum",
          empty: "fallback",
          options: ["item-aligned", "popper"]
        }
      });
      def2.props.setDefaults({ position: "item-aligned" });
      const select = asSelectContent();
      select.asTransition.configure({ enterDuration: 150, leaveDuration: 100 });
      const { open } = select.stateHandles;
      const { transitionState } = select.asTransition;
      def2.feedback.style.use(tw("relative z-50 w-[var(--proto-ui-anchor-width)] min-w-[var(--proto-ui-anchor-width)] max-h-[var(--proto-ui-available-height)] overflow-x-hidden overflow-y-auto rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md outline-none transition-none duration-150"));
      def2.rule({
        when: (w) => w.any(w.state(transitionState).eq("entering"), w.state(transitionState).eq("entered")),
        intent: (i) => i.feedback.style.use(tw("animate-in fade-in-0 zoom-in-95"))
      });
      def2.rule({
        when: (w) => w.state(open).eq(false),
        intent: (i) => i.feedback.style.use(tw("animate-out fade-out-0 zoom-out-95"))
      });
      def2.rule({
        when: (w) => w.all(w.state(open).eq(true), w.prop("side").eq("bottom")),
        intent: (i) => i.feedback.style.use(tw("slide-in-from-top-2"))
      });
      def2.rule({
        when: (w) => w.all(w.state(open).eq(true), w.prop("side").eq("top")),
        intent: (i) => i.feedback.style.use(tw("slide-in-from-bottom-2"))
      });
      def2.rule({
        when: (w) => w.all(w.state(open).eq(true), w.prop("side").eq("left")),
        intent: (i) => i.feedback.style.use(tw("slide-in-from-right-2"))
      });
      def2.rule({
        when: (w) => w.all(w.state(open).eq(true), w.prop("side").eq("right")),
        intent: (i) => i.feedback.style.use(tw("slide-in-from-left-2"))
      });
    }
  });
  var content_proto_default8 = selectContent2;

  // ../packages/prototypes/shadcn/src/select/item.proto.ts
  function renderCheck(renderer, selected) {
    return renderer.el("span", { style: tw("pointer-events-none flex size-5 shrink-0 items-center justify-center") }, selected ? renderer.svg.root({
      viewBox: "0 0 24 24",
      width: 16,
      height: 16,
      fill: "none",
      stroke: "currentColor",
      strokeWidth: 2,
      strokeLinecap: "round",
      strokeLinejoin: "round"
    }, renderer.svg.path({ d: "m20 6-11 11-5-5" })) : null);
  }
  var selectItem2 = definePrototype({
    name: "shadcn-select-item",
    setup(def2) {
      const state2 = asSelectItem().stateHandles;
      if (!state2)
        throw new Error("[shadcn-select-item] Select Item must project option states.");
      const { disabled, hovered, focused, focusVisible, pressed, active, selected } = state2;
      def2.feedback.style.use(tw("relative flex w-full cursor-default items-center justify-between gap-2 rounded-sm px-2 py-1.5 text-sm outline-none select-none"));
      def2.rule({
        when: (w) => w.any(w.state(active).eq(true), w.state(hovered).eq(true), w.state(focused).eq(true), w.state(focusVisible).eq(true)),
        intent: (i) => i.feedback.style.use(tw("bg-accent text-accent-foreground"))
      });
      def2.rule({
        when: (w) => w.state(pressed).eq(true),
        intent: (i) => i.feedback.style.use(tw("bg-accent/80"))
      });
      def2.rule({
        when: (w) => w.state(disabled).eq(true),
        intent: (i) => i.feedback.style.use(tw("pointer-events-none opacity-50"))
      });
      let renderTask = null;
      selected.watch((run2, event2) => {
        if (event2.type !== "next")
          return;
        renderTask?.cancel();
        renderTask = delay(0, () => {
          renderTask = null;
          run2.update();
        });
      });
      def2.lifecycle.onUnmounted(() => {
        renderTask?.cancel();
        renderTask = null;
      });
      return (renderer) => [renderer.r.slot(), renderCheck(renderer, selected.get())];
    }
  });
  var item_proto_default4 = selectItem2;

  // ../packages/prototypes/base/src/separator/root.proto.ts
  function setupSeparatorRoot(def2) {
    def2.props.define({
      orientation: { type: "enum", empty: "fallback", options: ["horizontal", "vertical"] },
      decorative: { type: "boolean", empty: "fallback" }
    });
    def2.props.setDefaults({ orientation: "horizontal", decorative: true });
    const orientation = def2.state.enum("orientation", "horizontal", {
      options: ["horizontal", "vertical"]
    });
    const decorative = def2.state.bool("decorative", true);
    const role = def2.state.string("role", "");
    const hidden = def2.state.bool("hidden", true);
    const a11yOrientation = def2.state.string("a11yOrientation", "");
    def2.expose.state("orientation", orientation);
    def2.expose.state("decorative", decorative);
    def2.a11y.role(role);
    def2.a11y.state("orientation", a11yOrientation);
    def2.a11y.tree({ hidden });
    const sync = (props) => {
      const nextOrientation = props.orientation ?? "horizontal";
      const nextDecorative = props.decorative ?? true;
      orientation.set(nextOrientation, "reason: separator orientation");
      decorative.set(nextDecorative, "reason: separator decorative");
      role.set(nextDecorative ? "" : "separator", "reason: separator role");
      a11yOrientation.set(nextDecorative ? "" : nextOrientation, "reason: separator a11y orientation");
      hidden.set(nextDecorative, "reason: separator hidden");
    };
    def2.lifecycle.onCreated((run2) => sync(run2.props.get()));
    def2.props.watchAll((_run, next) => sync(next));
    return () => null;
  }
  var asSeparatorRoot = defineAsHook({ name: "as-separator-root", setup: setupSeparatorRoot });
  var separatorRoot = definePrototype({ name: "base-separator-root", setup: setupSeparatorRoot });
  // ../packages/prototypes/shadcn/src/separator/root.proto.ts
  var ROOT_BASE_TOKENS3 = ["shrink-0", "bg-border"].join(" ");
  var separatorRoot2 = definePrototype({
    name: "shadcn-separator-root",
    setup(def2) {
      const separatorState = asSeparatorRoot().stateHandles;
      if (!separatorState) {
        throw new Error("[shadcn-separator-root] asSeparatorRoot must project Separator root state handles.");
      }
      const { orientation } = separatorState;
      def2.feedback.style.use(tw(ROOT_BASE_TOKENS3));
      def2.rule({
        when: (w) => w.state(orientation).eq("horizontal"),
        intent: (i) => i.feedback.style.use(tw("h-px w-full"))
      });
      def2.rule({
        when: (w) => w.state(orientation).eq("vertical"),
        intent: (i) => i.feedback.style.use(tw("h-full w-px"))
      });
      return () => null;
    }
  });
  var root_proto_default14 = separatorRoot2;
  // ../packages/prototypes/base/src/dialog/shared.ts
  var nextDialogRootId = 0;
  function createDialogRootId() {
    nextDialogRootId += 1;
    return `pui-dialog-${nextDialogRootId}`;
  }
  function createDialogPartId(rootId, role) {
    return `${rootId || "pui-dialog"}-${role}`;
  }
  function requestDialogOpen(run2, nextOpen, reason, focusReason) {
    try {
      run2.context.update(DIALOG_CONTEXT, (prev) => ({
        ...prev,
        open: prev.controlled ? prev.open : nextOpen,
        openFocusReason: nextOpen ? focusReason : null,
        returnFocusReason: nextOpen ? null : focusReason,
        requestedOpen: nextOpen,
        requestReason: reason,
        requestFocusReason: focusReason,
        requestVersion: prev.requestVersion + 1
      }));
      return true;
    } catch (error5) {
      if (error5?.code === "CONTEXT_DISCONNECTED")
        return false;
      throw error5;
    }
  }
  var DIALOG_FAMILY = createAnatomyFamily("base-dialog", {
    roles: {
      root: { cardinality: { min: 1, max: 1 } },
      trigger: { cardinality: { min: 0, max: 100 } },
      mask: { cardinality: { min: 0, max: 1 } },
      content: { cardinality: { min: 0, max: 1 } },
      title: { cardinality: { min: 0, max: 1 } },
      description: { cardinality: { min: 0, max: 1 } },
      header: { cardinality: { min: 0, max: 1 } },
      footer: { cardinality: { min: 0, max: 1 } },
      close: { cardinality: { min: 0, max: 100 } }
    },
    relations: [
      { kind: "contains", parent: "root", child: "trigger" },
      { kind: "contains", parent: "root", child: "mask" },
      { kind: "contains", parent: "root", child: "content" },
      { kind: "contains", parent: "content", child: "title" },
      { kind: "contains", parent: "content", child: "description" },
      { kind: "contains", parent: "content", child: "header" },
      { kind: "contains", parent: "content", child: "footer" },
      { kind: "contains", parent: "content", child: "close" }
    ]
  });
  var DIALOG_CONTEXT = createContextKey("base-dialog");

  // ../packages/prototypes/base/src/dialog/root.proto.ts
  function sameContext4(a, b) {
    return a.rootId === b.rootId && a.open === b.open && a.openFocusReason === b.openFocusReason && a.returnFocusReason === b.returnFocusReason && a.controlled === b.controlled && a.disabled === b.disabled && a.alert === b.alert && a.a11yLabel === b.a11yLabel && a.requestedOpen === b.requestedOpen && a.requestReason === b.requestReason && a.requestFocusReason === b.requestFocusReason && a.requestVersion === b.requestVersion;
  }
  function setupDialogRoot(def2) {
    def2.anatomy.claim(DIALOG_FAMILY, { role: "root" });
    const rootId = createDialogRootId();
    def2.props.define({
      open: { type: "boolean", empty: "fallback" },
      defaultOpen: { type: "boolean", empty: "fallback" },
      disabled: { type: "boolean", empty: "fallback" },
      alert: { type: "boolean", empty: "fallback" },
      a11yLabel: { type: "string", empty: "fallback" }
    });
    def2.props.setDefaults({
      defaultOpen: false,
      disabled: false,
      alert: false,
      a11yLabel: ""
    });
    def2.context.provide(DIALOG_CONTEXT, {
      rootId,
      open: false,
      openFocusReason: null,
      returnFocusReason: null,
      controlled: false,
      disabled: false,
      alert: false,
      a11yLabel: "",
      requestedOpen: false,
      requestReason: null,
      requestFocusReason: null,
      requestVersion: 0
    });
    const openState = useOpenState({
      exposeOpenMethodKey: "openDialog",
      requestOpen(run2, nextOpen, reason) {
        const ctx = run2.context.read(DIALOG_CONTEXT);
        if (ctx.disabled)
          return;
        requestDialogOpen(run2, nextOpen, reason, "programmatic");
      }
    });
    const open = openState.getState?.("open");
    def2.expose.event("openChange", { payload: "json" });
    const initialContext = {
      rootId,
      open: false,
      openFocusReason: null,
      returnFocusReason: null,
      controlled: false,
      disabled: false,
      alert: false,
      a11yLabel: "",
      requestedOpen: false,
      requestReason: null,
      requestFocusReason: null,
      requestVersion: 0
    };
    let snapshot = initialContext;
    let published = initialContext;
    let lastRequestVersion = 0;
    const syncContext = (run2) => {
      const next = {
        ...snapshot,
        open: open?.get() ?? false
      };
      snapshot = next;
      if (sameContext4(published, next))
        return;
      published = next;
      run2.context.update(DIALOG_CONTEXT, next);
    };
    def2.context.subscribe(DIALOG_CONTEXT, (run2, next) => {
      snapshot = next;
      published = next;
      if (next.requestVersion !== lastRequestVersion) {
        lastRequestVersion = next.requestVersion;
        if (!next.controlled) {
          open?.set(next.requestedOpen, "reason: dialog open request => uncontrolled sync");
        }
        run2.expose.emit("openChange", {
          open: next.requestedOpen,
          reason: next.requestReason,
          focusReason: next.requestFocusReason
        });
        return;
      }
      if (!snapshot.controlled) {
        open?.set(next.open, "reason: dialog context sync => open");
      }
    });
    def2.lifecycle.onCreated((run2) => {
      snapshot = {
        ...snapshot,
        controlled: run2.props.isProvided("open"),
        disabled: !!run2.props.get().disabled,
        alert: !!run2.props.get().alert,
        a11yLabel: run2.props.get().a11yLabel ?? ""
      };
      syncContext(run2);
    });
    def2.props.watch(["open", "disabled", "alert", "a11yLabel"], (run2, next) => {
      snapshot = {
        ...snapshot,
        controlled: run2.props.isProvided("open"),
        disabled: !!next.disabled,
        alert: !!next.alert,
        a11yLabel: next.a11yLabel ?? ""
      };
      syncContext(run2);
    });
    open?.watch((run2, event2) => {
      if (event2.type !== "next")
        return;
      syncContext(run2);
    });
  }
  var asDialogRoot = defineAsHook({
    name: "as-dialog-root",
    setup: setupDialogRoot
  });
  var dialogRoot = definePrototype({
    name: "base-dialog-root",
    setup(def2) {
      setupDialogRoot(def2);
    }
  });
  // ../packages/prototypes/base/src/dialog/command.ts
  function setupDialogCommand(def2, reasonPrefix) {
    asTrigger();
    def2.props.define({
      disabled: { type: "boolean", empty: "fallback" }
    });
    def2.props.setDefaults({ disabled: false });
    const disabled = def2.state.bool("disabled", false);
    const hovered = def2.state.bool("hovered", false);
    const pressed = def2.state.bool("pressed", false);
    const focusable = asFocusable();
    focusable.configure({ disabled: false });
    const focused = focusable.focused;
    const focusVisible = focusable.focusVisible;
    def2.expose.state("disabled", disabled);
    def2.expose.state("hovered", hovered);
    def2.expose.state("focused", focused);
    def2.expose.state("focusVisible", focusVisible);
    def2.expose.state("pressed", pressed);
    def2.expose.method("focusSelf", (options) => {
      if (disabled.get())
        return;
      focusable.focusSelf(options);
    });
    def2.a11y.role("button");
    def2.a11y.nameFromContent();
    def2.a11y.state("disabled", disabled);
    def2.a11y.action("activate", { event: "click" });
    const clearTransient = (reason) => {
      hovered.set(false, reason);
      pressed.set(false, reason);
    };
    const syncDisabled = (nextDisabled) => {
      disabled.set(nextDisabled, `reason: ${reasonPrefix} disabled sync`);
      focusable.setDisabled(nextDisabled);
      if (nextDisabled)
        clearTransient(`reason: ${reasonPrefix} disabled => reset interaction`);
    };
    def2.event.onGlobal("key.down", (_run, ev) => {
      const detail = ev;
      if (disabled.get() || !focused.get() || detail?.key !== " ")
        return;
      ev.control.requestDefaultActionPrevention({
        reason: `${reasonPrefix}.space-activation`,
        source: reasonPrefix
      });
    });
    def2.event.on("pointer.enter", () => {
      if (!disabled.get())
        hovered.set(true, `reason: ${reasonPrefix} pointer.enter`);
    });
    def2.event.on("pointer.leave", () => clearTransient(`reason: ${reasonPrefix} pointer.leave`));
    def2.event.on("pointer.cancel", () => clearTransient(`reason: ${reasonPrefix} pointer.cancel`));
    def2.event.on("pointer.down", () => {
      if (!disabled.get())
        pressed.set(true, `reason: ${reasonPrefix} pointer.down`);
    });
    def2.event.on("pointer.up", () => pressed.set(false, `reason: ${reasonPrefix} pointer.up`));
    def2.event.on("press.commit", () => {
      pressed.set(false, `reason: ${reasonPrefix} press.commit`);
    });
    return { disabled, syncDisabled };
  }

  // ../packages/prototypes/base/src/dialog/trigger.proto.ts
  function setupDialogTrigger(def2) {
    def2.anatomy.claim(DIALOG_FAMILY, { role: "trigger" });
    const command = setupDialogCommand(def2, "dialog trigger");
    const expanded = def2.state.bool("dialogExpanded", false);
    const hasPopup = def2.state.string("dialogHasPopup", "dialog");
    const controls = def2.state.string("dialogContentId", "");
    def2.a11y.state("expanded", expanded);
    def2.a11y.state("hasPopup", hasPopup);
    def2.a11y.relation("controls", { target: controls });
    const syncDialogFacts = (ctx) => {
      expanded.set(ctx.open, "reason: dialog trigger expanded sync");
      controls.set(createDialogPartId(ctx.rootId, "content"), "reason: dialog trigger controls sync");
    };
    def2.context.subscribe(DIALOG_CONTEXT, (run2, next) => {
      command.syncDisabled(!!run2.props.get().disabled || next.disabled);
      syncDialogFacts(next);
    });
    def2.lifecycle.onCreated((run2) => {
      const ctx = run2.context.read(DIALOG_CONTEXT);
      command.syncDisabled(!!run2.props.get().disabled || ctx.disabled);
      syncDialogFacts(ctx);
    });
    def2.props.watch(["disabled"], (run2, next) => {
      command.syncDisabled(!!next.disabled || run2.context.read(DIALOG_CONTEXT).disabled);
    });
    def2.event.on("press.commit", (run2, ev) => {
      const ctx = run2.context.read(DIALOG_CONTEXT);
      if (command.disabled.get())
        return;
      const openFocusReason = ev?.key ? "keyboard" : "pointer";
      requestDialogOpen(run2, !ctx.open, "trigger.press", openFocusReason);
    });
  }
  var asDialogTrigger = defineAsHook({
    name: "as-dialog-trigger",
    setup: setupDialogTrigger
  });
  var dialogTrigger = definePrototype({
    name: "base-dialog-trigger",
    setup: setupDialogTrigger
  });
  // ../packages/prototypes/base/src/dialog/overlay.proto.ts
  function projectDialogMaskHandle(result) {
    const open = result.getState?.("open");
    const asTransition2 = result.getAsHookHandle?.("asTransition");
    if (!open || !asTransition2) {
      throw new Error("[as-dialog-mask] missing captured Dialog or Transition handles.");
    }
    return { stateHandles: { open }, asTransition: asTransition2 };
  }
  function setupDialogMask(def2) {
    def2.anatomy.claim(DIALOG_FAMILY, { role: "mask" });
    def2.props.define({
      passthrough: { type: "boolean", empty: "fallback" }
    });
    def2.props.setDefaults({
      passthrough: false
    });
    const overlay2 = asOverlay();
    overlay2.configure({
      closeOnEscape: false,
      closeOnOutsidePress: false,
      closeOnFocusOutside: false,
      portal: true,
      modal: true,
      layerRole: "dialog-mask"
    });
    const hitParticipation = asHitParticipation({
      debugLabel: "dialog-mask",
      meta: {
        overlayKind: "dialog-mask"
      }
    });
    const transition = asTransition();
    overlay2.bindPresence({
      enter: transition.controls.enter,
      leave: transition.controls.leave,
      present: transition.isPresent
    });
    const open = def2.state.bool("open", false);
    let hitRegionDispose = null;
    let hitSyncDisposed = false;
    const syncHitParticipation = (run2) => {
      if (hitSyncDisposed)
        return;
      const target = run2.host?.get?.() ?? null;
      hitRegionDispose?.();
      hitRegionDispose = null;
      if (!target)
        return;
      hitRegionDispose = hitParticipation.registerRegion(target, {
        role: "mask",
        mode: run2.props.get().passthrough ? "passthrough" : "participating",
        meta: {
          overlayKind: "dialog-mask"
        }
      });
    };
    const updateOpen = (nextOpen, reason) => {
      open.set(nextOpen, reason ?? "reason: dialog mask sync => open");
      if (nextOpen) {
        overlay2.openOverlay(reason ?? "dialog.open");
      } else {
        overlay2.close(reason ?? "dialog.close");
      }
    };
    def2.context.subscribe(DIALOG_CONTEXT, (_run, next) => {
      updateOpen(next.open, "reason: dialog context sync => mask open");
    });
    def2.props.watch(["passthrough"], (run2) => {
      syncHitParticipation(run2);
    });
    def2.lifecycle.onCreated((run2) => {
      const ctx = run2.context.read(DIALOG_CONTEXT);
      updateOpen(ctx.open, "reason: lifecycle.onCreated => dialog mask open sync");
    });
    def2.lifecycle.onMounted((run2) => {
      hitSyncDisposed = false;
      syncHitParticipation(run2);
      updateOpen(open.get(), "reason: lifecycle.onMounted => dialog mask open sync");
    });
    def2.lifecycle.onUnmounted(() => {
      hitSyncDisposed = true;
      hitRegionDispose?.();
      hitRegionDispose = null;
    });
    def2.rule({
      when: (w) => w.state(transition.isPresent).eq(false),
      intent: (i) => i.feedback.style.use(tw("hidden"))
    });
  }
  var asDialogMask = defineAsHook({
    name: "as-dialog-mask",
    setup: setupDialogMask,
    projectHandle: projectDialogMaskHandle
  });
  var dialogMask = definePrototype({
    name: "base-dialog-mask",
    setup(def2) {
      setupDialogMask(def2);
      def2.feedback.style.use(tw("fixed inset-0"));
    }
  });
  // ../packages/prototypes/base/src/dialog/content.proto.ts
  function projectDialogContentHandle(result) {
    const open = result.getState?.("open");
    const asTransition2 = result.getAsHookHandle?.("asTransition");
    if (!open || !asTransition2) {
      throw new Error("[as-dialog-content] missing captured Dialog or Transition handles.");
    }
    return { stateHandles: { open }, asTransition: asTransition2 };
  }
  function setupDialogContent(def2) {
    def2.anatomy.claim(DIALOG_FAMILY, { role: "content" });
    const alertProp = def2.state.bool("alert", false);
    const role = def2.state.string("dialogRole", "dialog", {
      options: ["dialog", "alertdialog"]
    });
    const modal = def2.state.bool("dialogModal", true);
    const contentId = def2.state.string("dialogContentId", "");
    const accessibleLabel = def2.state.string("dialogAccessibleLabel", "");
    const labelledBy = def2.state.string("dialogLabelledBy", "");
    const describedBy = def2.state.string("dialogDescribedBy", "");
    def2.a11y.id(contentId);
    def2.a11y.role(role);
    def2.a11y.name(accessibleLabel);
    def2.a11y.state("modal", modal);
    def2.a11y.relation("labelledBy", { target: labelledBy });
    def2.a11y.relation("describedBy", { target: describedBy });
    const overlay2 = asOverlay();
    overlay2.configure({
      closeOnEscape: true,
      closeOnOutsidePress: false,
      closeOnFocusOutside: false,
      restore: "trigger",
      entry: "content",
      placement: "center",
      portal: true,
      modal: false,
      layerRole: "dialog-content"
    });
    const boundary2 = asBoundary();
    boundary2.observe("pointer.press");
    const focusScope = asFocusScope();
    focusScope.configure({ trap: true, loop: true });
    const transition = asTransition();
    overlay2.bindPresence({
      enter: transition.controls.enter,
      leave: transition.controls.leave,
      present: transition.isPresent
    });
    const open = def2.state.bool("open", false);
    def2.expose.state("open", open);
    let mountedRun = null;
    let currentContext = null;
    let warnedMissingAlertDescription = false;
    const hasLivePart = (run2, role2) => {
      try {
        return run2.anatomy.has(DIALOG_FAMILY, role2);
      } catch (error5) {
        if (error5?.code === "ANATOMY_CLAIM_INVALID")
          return false;
        throw error5;
      }
    };
    const syncA11yRelations = (run2, ctx) => {
      const hasTitle = hasLivePart(run2, "title");
      const hasDescription = hasLivePart(run2, "description");
      labelledBy.set(hasTitle ? createDialogPartId(ctx.rootId, "title") : "", "reason: dialog live title relation sync");
      accessibleLabel.set(hasTitle ? "" : ctx.a11yLabel, "reason: dialog accessible label fallback sync");
      describedBy.set(hasDescription ? createDialogPartId(ctx.rootId, "description") : "", "reason: dialog live description relation sync");
      if (!mountedRun || !ctx.alert || hasDescription) {
        warnedMissingAlertDescription = false;
        return;
      }
      if (warnedMissingAlertDescription)
        return;
      warnedMissingAlertDescription = true;
      console.warn("[base-dialog-content] Alert Dialog requires a Dialog Description containing its primary message.");
    };
    def2.anatomy.subscribeParts(DIALOG_FAMILY, "title", (run2) => {
      if (currentContext)
        syncA11yRelations(run2, currentContext);
    });
    def2.anatomy.subscribeParts(DIALOG_FAMILY, "description", (run2) => {
      if (currentContext)
        syncA11yRelations(run2, currentContext);
    });
    const updateOpen = (nextOpen, reason, options) => {
      const prevOpen = open.get();
      open.set(nextOpen, reason ?? "reason: dialog content sync => open");
      if (nextOpen) {
        overlay2.openOverlay(reason ?? "dialog.open");
      } else {
        overlay2.close(reason ?? "dialog.close");
      }
      if (!mountedRun)
        return;
      if (nextOpen) {
        if (!prevOpen || !focusScope.isActive()) {
          focusScope.activate({ reason: options?.focusReason ?? "programmatic" });
        }
      } else {
        if (prevOpen)
          focusScope.deactivate({ reason: options?.focusReason ?? "programmatic" });
      }
    };
    const syncIdentity = (ctx) => {
      contentId.set(createDialogPartId(ctx.rootId, "content"), "reason: dialog content id sync");
    };
    const syncAlert = (ctx) => {
      const alert = ctx.alert;
      alertProp.set(alert, "reason: dialog alert sync");
      role.set(alert ? "alertdialog" : "dialog", "reason: dialog semantic role sync");
    };
    def2.context.subscribe(DIALOG_CONTEXT, (run2, next) => {
      currentContext = next;
      syncIdentity(next);
      syncAlert(next);
      syncA11yRelations(run2, next);
      updateOpen(next.open, "reason: dialog context sync => content", {
        focusReason: next.open ? next.openFocusReason : next.returnFocusReason
      });
    });
    def2.lifecycle.onCreated((run2) => {
      const ctx = run2.context.read(DIALOG_CONTEXT);
      currentContext = ctx;
      syncIdentity(ctx);
      syncAlert(ctx);
      syncA11yRelations(run2, ctx);
      updateOpen(ctx.open, "reason: lifecycle.onCreated => dialog content open sync", {
        focusReason: ctx.open ? ctx.openFocusReason : ctx.returnFocusReason
      });
    });
    def2.lifecycle.onMounted((run2) => {
      mountedRun = run2;
      const ctx = run2.context.read(DIALOG_CONTEXT);
      currentContext = ctx;
      syncIdentity(ctx);
      syncAlert(ctx);
      syncA11yRelations(run2, ctx);
      updateOpen(ctx.open, "reason: lifecycle.onMounted => dialog content open sync", {
        focusReason: ctx.open ? ctx.openFocusReason : ctx.returnFocusReason
      });
    });
    def2.lifecycle.onUnmounted(() => {
      mountedRun = null;
      currentContext = null;
    });
    overlay2.open.watch((_ctx, event2) => {
      if (event2.type !== "next" || event2.next || event2.reason !== "escape")
        return;
      const run2 = mountedRun;
      if (!run2)
        return;
      const ctx = currentContext;
      if (!ctx)
        return;
      if (!ctx.open)
        return;
      requestDialogOpen(run2, false, "escape", "keyboard");
      if (ctx.controlled)
        overlay2.openOverlay("controlled.sync");
    });
    boundary2.subscribeOutside(() => {
      if (!overlay2.isOpen())
        return;
      const returnFocusReason = "pointer";
      const run2 = mountedRun;
      if (!run2)
        return;
      const ctx = currentContext;
      if (!ctx)
        return;
      if (!ctx.open)
        return;
      if (alertProp.get())
        return;
      requestDialogOpen(run2, false, "outside.press", returnFocusReason);
    });
    def2.rule({
      when: (w) => w.state(transition.isPresent).eq(false),
      intent: (i) => i.feedback.style.use(tw("hidden"))
    });
  }
  var asDialogContent = defineAsHook({
    name: "as-dialog-content",
    setup: setupDialogContent,
    projectHandle: projectDialogContentHandle
  });
  var dialogContent = definePrototype({
    name: "base-dialog-content",
    setup(def2) {
      setupDialogContent(def2);
      def2.feedback.style.use(tw("fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2"));
    }
  });
  // ../packages/prototypes/base/src/dialog/title.proto.ts
  function setupDialogTitle(def2) {
    def2.anatomy.claim(DIALOG_FAMILY, { role: "title" });
    const id = def2.state.string("dialogTitleId", "");
    def2.a11y.id(id);
    def2.a11y.nameFromContent();
    def2.context.subscribe(DIALOG_CONTEXT, (_run, next) => {
      id.set(createDialogPartId(next.rootId, "title"), "reason: dialog title id sync");
    });
    def2.lifecycle.onCreated((run2) => {
      id.set(createDialogPartId(run2.context.read(DIALOG_CONTEXT).rootId, "title"), "reason: dialog title created id sync");
    });
  }
  var asDialogTitle = defineAsHook({
    name: "as-dialog-title",
    setup: setupDialogTitle
  });
  var dialogTitle = definePrototype({
    name: "base-dialog-title",
    setup: setupDialogTitle
  });
  // ../packages/prototypes/base/src/dialog/description.proto.ts
  function setupDialogDescription(def2) {
    def2.anatomy.claim(DIALOG_FAMILY, { role: "description" });
    const id = def2.state.string("dialogDescriptionId", "");
    def2.a11y.id(id);
    def2.context.subscribe(DIALOG_CONTEXT, (_run, next) => {
      id.set(createDialogPartId(next.rootId, "description"), "reason: dialog description id sync");
    });
    def2.lifecycle.onCreated((run2) => {
      id.set(createDialogPartId(run2.context.read(DIALOG_CONTEXT).rootId, "description"), "reason: dialog description created id sync");
    });
  }
  var asDialogDescription = defineAsHook({
    name: "as-dialog-description",
    setup: setupDialogDescription
  });
  var dialogDescription = definePrototype({
    name: "base-dialog-description",
    setup: setupDialogDescription
  });
  // ../packages/prototypes/base/src/dialog/close.proto.ts
  function setupDialogClose(def2) {
    def2.anatomy.claim(DIALOG_FAMILY, { role: "close" });
    const command = setupDialogCommand(def2, "dialog close");
    def2.context.subscribe(DIALOG_CONTEXT, (run2, next) => {
      command.syncDisabled(!!run2.props.get().disabled || next.disabled);
    });
    def2.lifecycle.onCreated((run2) => {
      command.syncDisabled(!!run2.props.get().disabled || run2.context.read(DIALOG_CONTEXT).disabled);
    });
    def2.props.watch(["disabled"], (run2, next) => {
      command.syncDisabled(!!next.disabled || run2.context.read(DIALOG_CONTEXT).disabled);
    });
    def2.event.on("press.commit", (run2, ev) => {
      if (command.disabled.get())
        return;
      const returnFocusReason = ev?.key ? "keyboard" : "pointer";
      requestDialogOpen(run2, false, "close.press", returnFocusReason);
    });
  }
  var asDialogClose = defineAsHook({
    name: "as-dialog-close",
    setup: setupDialogClose
  });
  var dialogClose = definePrototype({
    name: "base-dialog-close",
    setup: setupDialogClose
  });
  // ../packages/prototypes/shadcn/src/dialog/close.proto.ts
  var dialogClose2 = definePrototype({
    name: "shadcn-dialog-close",
    setup() {
      asDialogClose();
    }
  });
  var close_proto_default2 = dialogClose2;

  // ../packages/prototypes/shadcn/src/dialog/content.proto.ts
  var dialogContent2 = definePrototype({
    name: "shadcn-dialog-content",
    setup(def2) {
      const dialog = asDialogContent();
      dialog.asTransition.configure({ enterDuration: 200, leaveDuration: 200 });
      const dialogState = dialog.stateHandles;
      const { open } = dialogState;
      def2.feedback.style.use(tw("fixed left-1/2 top-1/2 grid w-full max-w-lg gap-4 -translate-x-1/2 -translate-y-1/2 rounded-lg border border-border bg-background p-6 shadow-lg duration-200 outline-none"));
      def2.rule({
        when: (w) => w.state(open).eq(true),
        intent: (i) => i.feedback.style.use(tw("animate-in fade-in-0 zoom-in-95"))
      });
      def2.rule({
        when: (w) => w.state(open).eq(false),
        intent: (i) => i.feedback.style.use(tw("animate-out fade-out-0 zoom-out-95"))
      });
    }
  });
  var content_proto_default10 = dialogContent2;

  // ../packages/prototypes/shadcn/src/dialog/close-icon.proto.ts
  var dialogCloseIcon = definePrototype({
    name: "shadcn-dialog-close-icon",
    setup(def2) {
      const state2 = asDialogClose().stateHandles;
      if (!state2)
        throw new Error("[shadcn-dialog-close-icon] command states are required.");
      const { disabled, hovered, focusVisible } = state2;
      def2.a11y.name("Close");
      def2.feedback.style.use(tw("absolute right-4 top-4 inline-flex items-center justify-center rounded-sm opacity-70 transition-opacity outline-none ring-offset-0"));
      def2.rule({
        when: (w) => w.state(hovered).eq(true),
        intent: (i) => i.feedback.style.use(tw("opacity-100"))
      });
      def2.rule({
        when: (w) => w.state(focusVisible).eq(true),
        intent: (i) => i.feedback.style.use(tw("ring-2 ring-ring ring-offset-2 ring-offset-background"))
      });
      def2.rule({
        when: (w) => w.state(disabled).eq(true),
        intent: (i) => i.feedback.style.use(tw("pointer-events-none opacity-50"))
      });
      return (renderer) => [
        renderer.r.slot(),
        renderer.svg.root({
          viewBox: "0 0 24 24",
          width: 16,
          height: 16,
          fill: "none",
          stroke: "currentColor",
          strokeWidth: 2,
          strokeLinecap: "round",
          strokeLinejoin: "round"
        }, [renderer.svg.path({ d: "M18 6 6 18" }), renderer.svg.path({ d: "m6 6 12 12" })])
      ];
    }
  });
  var close_icon_proto_default = dialogCloseIcon;

  // ../packages/prototypes/shadcn/src/dialog/description.proto.ts
  var dialogDescription2 = definePrototype({
    name: "shadcn-dialog-description",
    setup(def2) {
      asDialogDescription();
      def2.feedback.style.use(tw("text-sm text-muted-foreground"));
    }
  });
  var description_proto_default2 = dialogDescription2;

  // ../packages/prototypes/shadcn/src/dialog/overlay.proto.ts
  var dialogMask2 = definePrototype({
    name: "shadcn-dialog-mask",
    setup(def2) {
      const dialog = asDialogMask();
      dialog.asTransition.configure({ enterDuration: 150, leaveDuration: 150 });
      const dialogState = dialog.stateHandles;
      const { open } = dialogState;
      def2.feedback.style.use(tw("fixed inset-0 bg-black/50 backdrop-blur-xs"));
      def2.rule({
        when: (w) => w.state(open).eq(true),
        intent: (i) => i.feedback.style.use(tw("animate-in fade-in-0"))
      });
      def2.rule({
        when: (w) => w.state(open).eq(false),
        intent: (i) => i.feedback.style.use(tw("animate-out fade-out-0"))
      });
    }
  });
  var overlay_proto_default2 = dialogMask2;

  // ../packages/prototypes/shadcn/src/dialog/root.proto.ts
  var dialogRoot2 = definePrototype({
    name: "shadcn-dialog-root",
    setup(def2) {
      asDialogRoot();
      def2.feedback.style.use(tw("relative inline-flex items-start"));
    }
  });
  var root_proto_default16 = dialogRoot2;

  // ../packages/prototypes/shadcn/src/dialog/title.proto.ts
  var dialogTitle2 = definePrototype({
    name: "shadcn-dialog-title",
    setup(def2) {
      asDialogTitle();
      def2.feedback.style.use(tw("text-lg font-semibold leading-none tracking-tight"));
    }
  });
  var title_proto_default2 = dialogTitle2;

  // ../packages/prototypes/shadcn/src/dialog/trigger.proto.ts
  var dialogTrigger2 = definePrototype({
    name: "shadcn-dialog-trigger",
    setup() {
      asDialogTrigger();
    }
  });
  var trigger_proto_default10 = dialogTrigger2;

  // ../packages/prototypes/shadcn/src/dialog/header.proto.ts
  var dialogHeader = definePrototype({
    name: "shadcn-dialog-header",
    setup(def2) {
      def2.anatomy.claim(DIALOG_FAMILY, { role: "header" });
      def2.feedback.style.use(tw("flex flex-col gap-2 text-left"));
      return (renderer) => renderer.r.slot();
    }
  });
  var header_proto_default = dialogHeader;

  // ../packages/prototypes/shadcn/src/dialog/footer.proto.ts
  var dialogFooter = definePrototype({
    name: "shadcn-dialog-footer",
    setup(def2) {
      def2.anatomy.claim(DIALOG_FAMILY, { role: "footer" });
      def2.feedback.style.use(tw("flex gap-2 items-center"));
      return (renderer) => renderer.r.slot();
    }
  });
  var footer_proto_default = dialogFooter;
  // index.ts
  globalThis.__sailbreak_proto_ui_metadata = { proto_ui_version: "0.3.0-alpha.0", proto_ui_commit: "02666e149e146094c439b538c9406cbba7d87341" };
  var BUILD_METADATA = globalThis.__sailbreak_proto_ui_metadata;
  var PROTO_UI_VERSION = typeof BUILD_METADATA?.proto_ui_version === "string" ? BUILD_METADATA.proto_ui_version : "main-snapshot";
  var PROTO_UI_COMMIT = typeof BUILD_METADATA?.proto_ui_commit === "string" ? BUILD_METADATA.proto_ui_commit : "unrecorded";
  var PROTOCOL_MAJOR = 1;
  var PROTOCOL_MINOR = 0;
  var HOST_NAME = "sailbreak";
  var GPUI_VERSION = "0.2.2";
  var HOST_PLATFORM = "embedded-quickjs";
  var REGISTRY_DIGEST = `proto-ui-main@${PROTO_UI_COMMIT}`;
  var MAX_BRIDGE_MESSAGE_BYTES = 256 * 1024;
  var DOCUMENT_POSITION_PRECEDING = 2;
  var DOCUMENT_POSITION_FOLLOWING = 4;
  var nodeGlobal = globalThis;
  if (!nodeGlobal.Node) {
    nodeGlobal.Node = {
      DOCUMENT_POSITION_PRECEDING,
      DOCUMENT_POSITION_FOLLOWING
    };
  }
  var nextSurfaceOrder = 0;
  var bridgeMicrotasks = [];
  var microtaskGlobal = globalThis;
  if (typeof microtaskGlobal.queueMicrotask !== "function") {
    microtaskGlobal.queueMicrotask = (callback) => {
      if (bridgeMicrotasks.length >= 256)
        throw new Error("microtask queue overflow");
      bridgeMicrotasks.push(callback);
    };
  }

  class LogicalBus {
    listeners = new Map;
    addEventListener(type, listener) {
      const listeners = this.listeners.get(type) ?? new Set;
      listeners.add(listener);
      this.listeners.set(type, listeners);
    }
    removeEventListener(type, listener) {
      const listeners = this.listeners.get(type);
      listeners?.delete(listener);
      if (listeners?.size === 0)
        this.listeners.delete(type);
    }
    dispatch(type, event2) {
      for (const listener of [...this.listeners.get(type) ?? []])
        listener(event2);
    }
  }
  var TEXTAREA_BASE_TOKENS = "flex min-h-16 w-full rounded-md border border-input bg-transparent px-3 py-2 text-base shadow-xs outline-none transition-colors";
  var shadcnTextareaRoot = definePrototype({
    name: "shadcn-textarea-root",
    modules: asTextareaRoot.modules,
    setup(def2) {
      const base = asTextareaRoot();
      const state2 = base.stateHandles;
      if (!state2)
        throw new Error("[shadcn-textarea-root] asTextareaRoot must project state handles.");
      const { disabled, focusVisible } = state2;
      def2.feedback.style.use(tw(TEXTAREA_BASE_TOKENS));
      def2.rule({
        when: (w) => w.state(focusVisible).eq(true),
        intent: (i) => i.feedback.style.use(tw("border-ring ring-3 ring-ring/50"))
      });
      def2.rule({
        when: (w) => w.state(disabled).eq(true),
        intent: (i) => i.feedback.style.use(tw("cursor-not-allowed opacity-50"))
      });
      def2.rule({
        when: (w) => w.all(w.meta("colorScheme").eq("dark"), w.state(disabled).eq(false)),
        intent: (i) => i.feedback.style.use(tw("bg-input/30"))
      });
    }
  });
  var registry = {
    "shadcn-button": button_proto_default,
    "shadcn-toggle": toggle_proto_default,
    "shadcn-checkbox-root": root_proto_default3,
    "shadcn-checkbox-indicator": indicator_proto_default2,
    "shadcn-separator-root": root_proto_default14,
    "shadcn-switch-root": root_proto_default5,
    "shadcn-switch-thumb": thumb_proto_default2,
    "shadcn-tabs-root": root_proto_default7,
    "shadcn-textarea-root": shadcnTextareaRoot,
    "shadcn-tabs-list": list_proto_default2,
    "shadcn-tabs-trigger": trigger_proto_default2,
    "shadcn-tabs-content": content_proto_default2,
    "shadcn-hover-card-root": root_proto_default9,
    "shadcn-hover-card-trigger": trigger_proto_default4,
    "shadcn-hover-card-content": content_proto_default4,
    "shadcn-dropdown-root": root_proto_default11,
    "shadcn-dropdown-trigger": trigger_proto_default6,
    "shadcn-dropdown-content": content_proto_default6,
    "shadcn-dropdown-item": item_proto_default2,
    "shadcn-select-root": root_proto_default13,
    "shadcn-select-trigger": trigger_proto_default8,
    "shadcn-select-value": value_proto_default2,
    "shadcn-select-content": content_proto_default8,
    "shadcn-select-item": item_proto_default4,
    "shadcn-dialog-root": root_proto_default16,
    "shadcn-dialog-trigger": trigger_proto_default10,
    "shadcn-dialog-mask": overlay_proto_default2,
    "shadcn-dialog-content": content_proto_default10,
    "shadcn-dialog-title": title_proto_default2,
    "shadcn-dialog-description": description_proto_default2,
    "shadcn-dialog-close": close_proto_default2,
    "shadcn-dialog-close-icon": close_icon_proto_default,
    "shadcn-dialog-header": header_proto_default,
    "shadcn-dialog-footer": footer_proto_default
  };
  var sessions = new Map;
  var bridgeFailed = false;
  function recordOf(value) {
    if (value === null || typeof value !== "object" || Array.isArray(value))
      return null;
    return value;
  }
  function stringValue(value) {
    return typeof value === "string" ? value : null;
  }
  function numberValue(value) {
    return typeof value === "number" && Number.isSafeInteger(value) ? value : null;
  }
  function booleanValue(value) {
    return typeof value === "boolean" ? value : null;
  }
  function getRequiredString(object, key) {
    const value = stringValue(object[key]);
    if (value === null || value.trim().length === 0)
      throw new Error(`missing string field: ${key}`);
    return value;
  }
  function getRequiredNumber(object, key) {
    const value = numberValue(object[key]);
    if (value === null || value <= 0)
      throw new Error(`invalid positive number field: ${key}`);
    return value;
  }
  function jsonValue(value, depth = 0) {
    if (depth > 16)
      return;
    if (value === null)
      return null;
    if (typeof value === "string" || typeof value === "boolean")
      return value;
    if (typeof value === "number")
      return Number.isFinite(value) ? value : undefined;
    if (Array.isArray(value)) {
      const values = [];
      for (const item of value) {
        const next = jsonValue(item, depth + 1);
        if (typeof next === "undefined")
          return;
        values.push(next);
      }
      return values;
    }
    const object = recordOf(value);
    if (!object)
      return;
    const result = {};
    for (const [key, item] of Object.entries(object)) {
      const next = jsonValue(item, depth + 1);
      if (typeof next === "undefined")
        return;
      result[key] = next;
    }
    return result;
  }
  function utf8Length(value) {
    let length = 0;
    for (const character of value) {
      const code = character.codePointAt(0) ?? 0;
      if (code <= 127)
        length += 1;
      else if (code <= 2047)
        length += 2;
      else if (code <= 65535)
        length += 3;
      else
        length += 4;
    }
    return length;
  }
  function jsonObject(value) {
    const parsed = jsonValue(value);
    if (!parsed || Array.isArray(parsed) || typeof parsed !== "object") {
      throw new Error("expected a JSON object");
    }
    return parsed;
  }
  function styleTokens(value) {
    const object = recordOf(value);
    if (!object)
      return [];
    const tokens = object.tokens;
    if (!Array.isArray(tokens))
      return [];
    return tokens.filter((token) => typeof token === "string");
  }
  function templateNode(value, record) {
    if (typeof value === "string" || typeof value === "number") {
      return { kind: "text", text: String(value) };
    }
    const object = recordOf(value);
    if (!object)
      return null;
    if (object.kind === "svg-node") {
      const tag = stringValue(object.tag);
      if (!tag)
        throw new Error("SVG template node is missing a tag");
      const children2 = templateChildren(object.children, record);
      const attributes = svgAttributes(object.props, tag);
      return {
        kind: "svg",
        tag,
        attributes,
        ...children2.length > 0 ? { children: children2 } : {}
      };
    }
    const type = object.type;
    const reserved = recordOf(type);
    if (reserved && reserved.kind === "slot") {
      return { kind: "slot", slot_id: record.slot.slot_id };
    }
    if (typeof type !== "string") {
      throw new Error("unsupported Proto UI template node");
    }
    const children = templateChildren(object.children, record);
    const style2 = styleTokens(object.style);
    return {
      kind: "container",
      tag: type,
      ...style2.length > 0 ? { style: style2 } : {},
      ...children.length > 0 ? { children } : {}
    };
  }
  function svgAttributes(value, tag) {
    const object = recordOf(value);
    if (!object)
      throw new Error(`SVG ${tag} props must be an object`);
    const attributes = {};
    for (const [name, candidate] of Object.entries(object)) {
      if (typeof candidate !== "string" && typeof candidate !== "number" && typeof candidate !== "boolean") {
        throw new Error(`SVG ${tag} attribute ${name} is not a primitive`);
      }
      attributes[name] = String(candidate);
    }
    return attributes;
  }
  function templateChildren(value, record) {
    const values = Array.isArray(value) ? value : [value];
    const nodes = [];
    for (const item of values) {
      if (item === null || typeof item === "boolean" || typeof item === "undefined")
        continue;
      const node = templateNode(item, record);
      if (node)
        nodes.push(node);
    }
    return nodes;
  }
  function a11ySnapshot(value, record) {
    const object = recordOf(value);
    const states = object ? recordOf(object.states) : null;
    const tree = object ? recordOf(object.tree) : null;
    const nameObject = object ? recordOf(object.name) : null;
    const name = nameObject?.kind === "content" ? record.slot.accessible_name : stringValue(nameObject?.value) ?? stringValue(object?.name) ?? "";
    const actionsObject = object ? recordOf(object.actions) : null;
    const actions = actionsObject ? Object.keys(actionsObject) : [];
    const selected = booleanValue(states?.selected);
    const toggled = booleanValue(states?.checked) ?? booleanValue(states?.pressed);
    const orientation = stringValue(states?.orientation);
    return {
      role: stringValue(object?.role) ?? "generic",
      name,
      disabled: booleanValue(states?.disabled) ?? false,
      focused: booleanValue(states?.focused) ?? false,
      focus_visible: booleanValue(states?.focusVisible) ?? false,
      hidden: booleanValue(tree?.hidden) ?? false,
      ...orientation !== null && orientation.length > 0 ? { orientation } : {},
      ...selected !== null ? { selected } : {},
      ...toggled !== null ? { toggled } : {},
      ...actions.length > 0 ? { actions } : {}
    };
  }
  function exposedStates(value) {
    const object = recordOf(value);
    if (!object)
      return {};
    const result = {};
    for (const [key, candidate] of Object.entries(object)) {
      const state2 = recordOf(candidate);
      const getter = state2?.get;
      const resolved = typeof getter === "function" ? getter.call(candidate) : candidate;
      const json = jsonValue(resolved);
      if (typeof json !== "undefined")
        result[key] = json;
    }
    return result;
  }
  function nextOutputSequence(record) {
    record.output_sequence += 1;
    return record.output_sequence;
  }
  function emitDiagnostic(record, code, detail, fatal) {
    record.events.push({ type: "diagnostic", diagnostic: { code, detail, fatal } });
  }
  function cancelDelayTasks(record) {
    for (const task of [...record.delay_tasks])
      task.cancel();
  }
  function scheduleTask(record, task) {
    if (record.scheduled_tasks.length >= 256) {
      bridgeFailed = true;
      throw new Error(`scheduled task overflow for session ${record.session_id}`);
    }
    record.scheduled_tasks.push(task);
  }
  function flushScheduledTasks() {
    let executed = 0;
    while (true) {
      let task = bridgeMicrotasks.shift();
      if (!task) {
        const record = [...sessions.values()].find((candidate) => candidate.scheduled_tasks.length > 0);
        task = record?.scheduled_tasks.shift();
      }
      if (!task)
        return;
      executed += 1;
      if (executed > 1024) {
        bridgeFailed = true;
        throw new Error("scheduled task flush overflow");
      }
      task();
    }
  }
  function advanceTime(record, milliseconds) {
    if (!Number.isSafeInteger(milliseconds) || milliseconds < 0) {
      throw new Error("advance_time requires a non-negative safe integer");
    }
    if (record.virtual_time > Number.MAX_SAFE_INTEGER - milliseconds) {
      throw new Error("virtual clock overflow");
    }
    record.virtual_time += milliseconds;
    while (true) {
      const due = [...record.delay_tasks].filter((task) => task.due <= record.virtual_time).sort((left, right) => left.due - right.due);
      if (due.length === 0)
        return;
      due[0].run();
    }
  }
  function emitStyle(record) {
    if (record.disposed)
      return;
    record.events.push({
      type: "style",
      session_id: record.session_id,
      instance_id: record.instance_id,
      view_epoch: record.session?.mountEpoch ?? 1,
      style: { tokens: [...record.style_tokens] }
    });
  }
  function emitStateValues(record) {
    if (record.disposed)
      return;
    record.state_values = exposedStates(record.exposed_handles);
    record.events.push({
      type: "state",
      session_id: record.session_id,
      instance_id: record.instance_id,
      view_epoch: record.session?.mountEpoch ?? 1,
      values: { ...record.state_values }
    });
  }
  function emitA11y(record, value) {
    if (record.disposed)
      return;
    record.a11y = a11ySnapshot(value, record);
    record.events.push({
      type: "a11y",
      session_id: record.session_id,
      instance_id: record.instance_id,
      view_epoch: record.session?.mountEpoch ?? 1,
      a11y: record.a11y
    });
  }
  function setExposes(record, value) {
    if (record.disposed)
      return;
    for (const unsubscribe of record.state_unsubs.splice(0))
      unsubscribe();
    record.exposed_handles = recordOf(value) ?? {};
    emitStateValues(record);
    for (const candidate of Object.values(record.exposed_handles)) {
      const candidateObject = recordOf(candidate);
      const subscribe = candidateObject?.subscribe;
      if (typeof subscribe !== "function")
        continue;
      const unsubscribe = subscribe.call(candidate, () => emitStateValues(record));
      if (typeof unsubscribe === "function")
        record.state_unsubs.push(unsubscribe);
    }
  }
  function parentRecordFor(record) {
    const parent = record.parent_ref;
    if (!parent)
      return null;
    const candidate = sessions.get(parent.session_id);
    if (!candidate || candidate.disposed)
      return null;
    if (candidate.instance_id !== parent.instance_id)
      return null;
    if (candidate.route_ref !== parent.route_ref)
      return null;
    if (candidate.session?.mountEpoch !== parent.view_epoch)
      return null;
    if (candidate.session?.mountPhase === "detached" || candidate.session?.mountPhase === "unmounting") {
      return null;
    }
    return candidate;
  }
  function parentTokenFor(instance3) {
    for (const record of sessions.values()) {
      if (record.surface === instance3)
        return parentRecordFor(record)?.surface ?? null;
    }
    return null;
  }
  function prototypeFor(instance3) {
    for (const record of sessions.values()) {
      if (record.surface === instance3)
        return record.definition;
    }
    return null;
  }
  function rootRecordFor(record) {
    let current = record;
    const visited = new Set;
    while (!visited.has(current.session_id)) {
      visited.add(current.session_id);
      const parent = parentRecordFor(current);
      if (!parent)
        break;
      current = parent;
    }
    return current;
  }
  function rootTargetFor(instance3) {
    for (const record of sessions.values()) {
      if (record.surface === instance3)
        return record.surface;
    }
    return null;
  }
  function familyGlobalBus(record) {
    return rootRecordFor(record).global_bus;
  }
  function recordForSurface(surface) {
    for (const record of sessions.values()) {
      if (record.surface === surface)
        return record;
    }
    return null;
  }
  function blurSurface(surface) {
    if (!surface.focused)
      return;
    surface.focused = false;
    const record = recordForSurface(surface);
    record?.root_bus.dispatch("host:blur", { target: surface, nativeEvent: { target: surface } });
  }
  function focusSurface(surface) {
    if (surface.focused)
      return;
    const record = recordForSurface(surface);
    if (!record)
      return;
    const root = rootRecordFor(record);
    for (const candidate of sessions.values()) {
      if (candidate.surface !== surface && rootRecordFor(candidate) === root) {
        blurSurface(candidate.surface);
      }
    }
    surface.focused = true;
    record.root_bus.dispatch("host:focus", { target: surface, nativeEvent: { target: surface } });
  }
  function parseParent(value) {
    if (typeof value === "undefined" || value === null)
      return null;
    const object = recordOf(value);
    if (!object)
      throw new Error("parent must be a JSON object");
    return {
      session_id: getRequiredString(object, "session_id"),
      instance_id: getRequiredString(object, "instance_id"),
      view_epoch: getRequiredNumber(object, "view_epoch"),
      route_ref: getRequiredString(object, "route_ref")
    };
  }
  function validateParent(parent) {
    const candidate = sessions.get(parent.session_id);
    if (!candidate)
      throw new Error(`unknown parent session: ${parent.session_id}`);
    if (candidate.instance_id !== parent.instance_id) {
      throw new Error(`parent instance mismatch: ${parent.instance_id}`);
    }
    if (candidate.route_ref !== parent.route_ref) {
      throw new Error(`parent route mismatch: expected ${candidate.route_ref ?? ""}/${parent.route_ref}`);
    }
    const epoch = candidate.session?.mountEpoch ?? 0;
    if (epoch !== parent.view_epoch) {
      throw new Error(`stale parent view epoch: expected ${epoch}/${parent.view_epoch}`);
    }
    if (candidate.disposed || candidate.session?.mountPhase === "detached" || candidate.session?.mountPhase === "unmounting") {
      throw new Error(`parent session is not mounted: ${parent.session_id}`);
    }
  }
  function textControlPatchValue(state2, patch) {
    if (typeof patch.value === "string")
      return patch.value;
    if (!state2.initialized && patch.valueMode === "uncontrolled" && typeof patch.defaultValue === "string") {
      return patch.defaultValue;
    }
    return null;
  }
  function replaceTextControlValue(state2, value) {
    state2.value = value;
    state2.selection = {
      ...state2.selection,
      start: Math.min(state2.selection.start, value.length),
      end: Math.min(state2.selection.end, value.length)
    };
  }
  function applyTextControlPatch(state2, patch, allowValueProjection) {
    state2.patch = { ...state2.patch, ...patch };
    if (typeof patch.defaultValue === "string")
      state2.defaultValue = patch.defaultValue;
    const value = textControlPatchValue(state2, patch);
    state2.initialized = true;
    if (value === null || value === state2.value)
      return;
    if (allowValueProjection) {
      replaceTextControlValue(state2, value);
      state2.deferredValue = null;
    } else {
      state2.deferredValue = value;
    }
  }
  function createTextControlHost(record) {
    return {
      attach(connection) {
        if (!connection || typeof connection !== "object" || typeof connection.onEvent !== "function") {
          throw new Error("[TextControl] host connection is invalid.");
        }
        if (record.text_control) {
          record.text_control.disposed = true;
          record.text_control.connection = null;
        }
        const initialPatch = { ...connection.patch };
        const state2 = {
          patch: initialPatch,
          connection,
          value: "",
          defaultValue: typeof initialPatch.defaultValue === "string" ? initialPatch.defaultValue : "",
          composing: false,
          selection: { start: 0, end: 0, direction: "none" },
          deferredValue: null,
          initialized: false,
          disposed: false
        };
        record.text_control = state2;
        applyTextControlPatch(state2, initialPatch, true);
        return {
          update(next) {
            if (state2.disposed)
              return;
            applyTextControlPatch(state2, next, !state2.composing);
          },
          snapshot() {
            return Object.freeze({
              value: state2.value,
              composing: state2.composing,
              selection: { ...state2.selection }
            });
          },
          dispose() {
            if (state2.disposed)
              return;
            state2.disposed = true;
            state2.connection = null;
            if (record.text_control === state2)
              record.text_control = null;
          }
        };
      }
    };
  }
  function textControlExposeEvent(record, key, payload) {
    const typeByKey = {
      valueChange: "input",
      change: "change",
      compositionStart: "compositionstart",
      compositionUpdate: "compositionupdate",
      compositionEnd: "compositionend"
    };
    const type = typeByKey[key];
    if (!type || !record.text_control || record.text_control.disposed)
      return null;
    const object = recordOf(payload);
    if (!object || typeof object.value !== "string")
      return null;
    return Object.freeze({
      type,
      value: object.value,
      composing: typeof object.composing === "boolean" ? object.composing : type === "compositionstart" || type === "compositionupdate",
      data: typeof object.data === "string" ? object.data : null,
      inputType: typeof object.inputType === "string" ? object.inputType : null
    });
  }
  function attachCapabilities(record, wiring) {
    const parentGetter = (instance3) => parentTokenFor(instance3);
    wiring.attach("rule-meta", [
      [RULE_META_GET_CAP, (key) => record.meta[key]]
    ]);
    wiring.attach("context", [
      [CONTEXT_INSTANCE_TOKEN_CAP, record.surface],
      [CONTEXT_PARENT_CAP, parentGetter]
    ]);
    wiring.attach("anatomy", [
      [ANATOMY_INSTANCE_TOKEN_CAP, record.surface],
      [ANATOMY_PARENT_CAP, parentGetter],
      [ANATOMY_GET_PROTO_CAP, (instance3) => prototypeFor(instance3)],
      [ANATOMY_ROOT_TARGET_CAP, (instance3) => rootTargetFor(instance3)],
      [ANATOMY_ORDER_OBSERVER_CAP, () => () => {
        return;
      }]
    ]);
    wiring.attach("as-trigger", [
      [AS_TRIGGER_INSTANCE_CAP, record.surface],
      [AS_TRIGGER_PARENT_CAP, parentGetter],
      [AS_TRIGGER_GET_PROTO_CAP, (instance3) => prototypeFor(instance3)],
      [AS_TRIGGER_MERGE_GROUP_CAP, () => {
        return;
      }],
      [AS_TRIGGER_GET_GROUP_EVENT_TARGET_CAP, (instance3) => {
        const target = rootTargetFor(instance3);
        if (!target)
          return record.root_bus;
        for (const candidate of sessions.values()) {
          if (candidate.surface === target)
            return candidate.root_bus;
        }
        return record.root_bus;
      }]
    ]);
    wiring.attach("event", [
      [EVENT_ROOT_TARGET_CAP, () => record.root_bus],
      [EVENT_GLOBAL_TARGET_CAP, () => familyGlobalBus(record)],
      [EVENT_CANCEL_DEFAULT_ACTION_CAP, () => {
        emitDiagnostic(record, "default-action-not-applicable", "GPUI owns the native default action; no browser cancellation was claimed", false);
      }]
    ]);
    wiring.attach("expose-event", [
      [EXPOSE_EVENT_SINK_CAP, (key, payload) => {
        const textEvent = textControlExposeEvent(record, key, payload);
        if (textEvent) {
          record.events.push({
            type: "text_control",
            session_id: record.session_id,
            instance_id: record.instance_id,
            view_epoch: record.session?.mountEpoch ?? 1,
            sequence: nextOutputSequence(record),
            control_ref: record.text_control_ref,
            event: textEvent
          });
          return;
        }
        record.events.push({
          type: "signal",
          session_id: record.session_id,
          instance_id: record.instance_id,
          view_epoch: record.session?.mountEpoch ?? 1,
          sequence: nextOutputSequence(record),
          key
        });
      }]
    ]);
    wiring.attach("text-control", [
      [TEXT_CONTROL_HOST_CAP, createTextControlHost(record)],
      [TEXT_CONTROL_RUN_IN_CALLBACK_CAP, (callback) => {
        const session2 = record.session;
        if (session2?.invokeInCallbackScope)
          session2.invokeInCallbackScope(callback);
        else
          callback();
      }]
    ]);
    wiring.attach("focus", [
      [FOCUS_ROOT_TARGET_CAP, () => record.surface],
      [FOCUS_TARGET_READY_CAP, (listener) => {
        record.ready_listeners.add(listener);
        return () => record.ready_listeners.delete(listener);
      }],
      [FOCUS_INSTANCE_TOKEN_CAP, record.surface],
      [FOCUS_PARENT_CAP, parentGetter],
      [FOCUS_IS_NATIVELY_FOCUSABLE_CAP, () => true],
      [FOCUS_SET_FOCUSABLE_CAP, (_target, _enabled) => {
        return;
      }],
      [FOCUS_REQUEST_FOCUS_CAP, (target) => {
        target.focus();
        return target.focused;
      }],
      [FOCUS_BLUR_CAP, (target) => target.blur()],
      [FOCUS_RUN_IN_CALLBACK_CAP, (callback) => {
        const session2 = record.session;
        if (session2?.invokeInCallbackScope)
          session2.invokeInCallbackScope(callback);
        else
          callback();
      }]
    ]);
    wiring.attach("feedback", [
      [EFFECTS_CAP, {
        queueStyle: (handle) => {
          record.style_tokens = styleTokens(handle);
        },
        requestFlush: () => emitStyle(record)
      }]
    ]);
    wiring.attach("a11y", [
      [A11Y_PROJECT_CAP, (snapshot) => emitA11y(record, snapshot)]
    ]);
    wiring.attach("expose-state", [
      [EXPOSE_STATE_SET_EXPOSES_CAP, (exposes) => setExposes(record, exposes)]
    ]);
  }
  function projection(record, children) {
    const viewEpoch = record.session?.mountEpoch ?? 1;
    record.commit_id += 1;
    const template2 = templateChildren(children, record);
    record.pending_commit = {
      commit_id: record.commit_id,
      view_epoch: viewEpoch,
      signal: record.pending_commit?.signal ?? { done: () => {
        return;
      } }
    };
    return {
      type: "projection",
      projection: {
        session_id: record.session_id,
        instance_id: record.instance_id,
        view_epoch: viewEpoch,
        commit_id: record.commit_id,
        template: template2,
        slot: { ...record.slot },
        style: { tokens: [...record.style_tokens] },
        a11y: record.a11y
      }
    };
  }
  function createRecord(sessionId, instanceId, prototype3, definition, props, meta, slot, routeRef, parentRef) {
    const order = nextSurfaceOrder++;
    const surface = {
      focused: false,
      order,
      compareDocumentPosition(other) {
        if (surface === other)
          return 0;
        return surface.order < other.order ? DOCUMENT_POSITION_FOLLOWING : DOCUMENT_POSITION_PRECEDING;
      },
      focus() {
        focusSurface(surface);
      },
      blur() {
        blurSurface(surface);
      }
    };
    return {
      session_id: sessionId,
      instance_id: instanceId,
      prototype: prototype3,
      definition,
      props,
      meta,
      slot,
      route_ref: routeRef,
      parent_ref: parentRef,
      root_bus: new LogicalBus,
      global_bus: new LogicalBus,
      surface,
      ready_listeners: new Set,
      seen_press_samples: new Set,
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
      delay_tasks: new Set,
      scheduled_tasks: []
    };
  }
  function runtimeHost(record) {
    return {
      prototypeName: record.prototype,
      getRawProps: () => record.props,
      schedule: (task) => scheduleTask(record, task),
      scheduleDelay: (durationMs, task) => {
        if (!Number.isFinite(durationMs) || durationMs < 0) {
          emitDiagnostic(record, "delayed-task-dropped", "delay must be a finite non-negative number", false);
          return { cancel: () => {
            return;
          } };
        }
        if (durationMs === 0) {
          let active2 = true;
          scheduleTask(record, () => {
            if (!active2)
              return;
            active2 = false;
            task();
          });
          return { cancel: () => {
            active2 = false;
          } };
        }
        if (record.delay_tasks.size >= 64 || record.virtual_time > Number.MAX_SAFE_INTEGER - durationMs) {
          emitDiagnostic(record, "delayed-task-dropped", "delay queue capacity or clock range exceeded", false);
          return { cancel: () => {
            return;
          } };
        }
        let active = true;
        let timer;
        timer = {
          due: record.virtual_time + durationMs,
          cancel: () => {
            if (!active)
              return;
            active = false;
            record.delay_tasks.delete(timer);
          },
          run: () => {
            if (!active)
              return;
            active = false;
            record.delay_tasks.delete(timer);
            task();
          }
        };
        record.delay_tasks.add(timer);
        return timer;
      },
      commit: (children, signal) => {
        if (!signal)
          throw new Error("Proto UI host commit requires a completion signal");
        record.pending_commit = {
          commit_id: record.commit_id + 1,
          view_epoch: record.session?.mountEpoch ?? 1,
          signal
        };
        record.events.push(projection(record, children));
      },
      onRuntimeReady: (wiring) => attachCapabilities(record, wiring)
    };
  }
  function startSession(command) {
    const sessionId = getRequiredString(command, "session_id");
    const instanceId = getRequiredString(command, "instance_id");
    const prototype3 = getRequiredString(command, "prototype");
    const definition = registry[prototype3];
    if (!definition) {
      throw new Error(`unknown Proto UI prototype: ${prototype3}`);
    }
    const props = jsonObject(command.props ?? {});
    const meta = jsonObject(command.meta ?? {});
    const slotObject = recordOf(command.slot);
    if (!slotObject)
      throw new Error("slot must be a JSON object");
    const slot = {
      slot_id: getRequiredString(slotObject, "slot_id"),
      accessible_name: getRequiredString(slotObject, "accessible_name")
    };
    const routeRef = command.route_ref === undefined || command.route_ref === null ? null : getRequiredString({ route_ref: command.route_ref }, "route_ref");
    const parentRef = parseParent(command.parent);
    if (parentRef)
      validateParent(parentRef);
    if (sessions.has(sessionId))
      throw new Error(`duplicate session: ${sessionId}`);
    const record = createRecord(sessionId, instanceId, prototype3, definition, props, meta, slot, routeRef, parentRef);
    sessions.set(sessionId, record);
    try {
      const session2 = createRuntimeSession(definition, runtimeHost(record));
      record.session = session2;
      record.events.unshift({
        type: "ready",
        handshake: {
          protocol: { major: PROTOCOL_MAJOR, minor: PROTOCOL_MINOR },
          proto_ui: PROTO_UI_VERSION,
          host: { name: HOST_NAME, gpui: GPUI_VERSION, platform: HOST_PLATFORM },
          registry_digest: REGISTRY_DIGEST
        }
      });
      session2.mount().catch((error5) => {
        emitDiagnostic(record, "runtime-mount-failed", String(error5), true);
      });
    } catch (error5) {
      sessions.delete(sessionId);
      throw error5;
    }
  }
  function sessionFor(command) {
    const sessionId = getRequiredString(command, "session_id");
    const record = sessions.get(sessionId);
    if (!record)
      throw new Error(`unknown session: ${sessionId}`);
    const instanceId = getRequiredString(command, "instance_id");
    if (record.instance_id !== instanceId)
      throw new Error(`instance mismatch: ${instanceId}`);
    return record;
  }
  function acknowledge(command) {
    const ack = recordOf(command.ack);
    if (!ack)
      throw new Error("projection ack must be a JSON object");
    const record = sessionFor(ack);
    const pending = record.pending_commit;
    if (!pending)
      throw new Error("projection ack has no pending commit");
    const epoch = getRequiredNumber(ack, "view_epoch");
    const commitId = getRequiredNumber(ack, "commit_id");
    if (pending.view_epoch !== epoch || pending.commit_id !== commitId) {
      throw new Error(`stale projection ack: expected ${pending.view_epoch}/${pending.commit_id}`);
    }
    const status = getRequiredString(ack, "status");
    if (status === "applied") {
      record.pending_commit = undefined;
      pending.signal.done();
      for (const listener of [...record.ready_listeners])
        listener();
    } else if (status !== "superseded") {
      throw new Error(`projection rejected: ${status}`);
    }
  }
  function input(command) {
    const inputObject = recordOf(command.input);
    if (!inputObject)
      throw new Error("input must be a JSON object");
    const record = sessionFor(inputObject);
    const kind = getRequiredString(inputObject, "kind");
    const routeRef = getRequiredString(inputObject, "route_ref");
    if (record.route_ref !== null && record.route_ref !== routeRef) {
      throw new Error(`input route mismatch: expected ${record.route_ref}/${routeRef}`);
    }
    if (record.route_ref === null)
      record.route_ref = routeRef;
    const sampleId = getRequiredString(inputObject, "sample_id");
    if (kind === "press_commit") {
      if (record.seen_press_samples.has(sampleId))
        return;
      if (record.seen_press_samples.size >= 1024) {
        const oldest = record.seen_press_samples.values().next().value;
        if (typeof oldest === "string")
          record.seen_press_samples.delete(oldest);
      }
      record.seen_press_samples.add(sampleId);
    }
    const eventType = kind.replaceAll("_", ".");
    const detailObject = recordOf(command.detail);
    const detail = detailObject ? { ...detailObject } : {};
    if (kind === "key_down" || kind === "key_up") {
      detail.preventDefault = () => {
        emitDiagnostic(record, "default-action-not-applicable", "GPUI owns the native default action; no browser cancellation was claimed", false);
      };
    }
    const event2 = { detail };
    if (kind === "key_down" || kind === "key_up") {
      familyGlobalBus(record).dispatch(eventType, event2);
    } else if (kind === "focus") {
      focusSurface(record.surface);
    } else if (kind === "blur") {
      blurSurface(record.surface);
    } else {
      record.root_bus.dispatch(eventType, event2);
    }
  }
  function textControl(command) {
    const operation = recordOf(command.command);
    if (!operation)
      throw new Error("text-control command must be a JSON object");
    const request = { ...command, ...operation };
    const record = sessionFor(request);
    const epoch = getRequiredNumber(request, "view_epoch");
    const currentEpoch = record.session?.mountEpoch ?? 0;
    if (currentEpoch !== epoch) {
      throw new Error(`stale text-control view epoch: expected ${currentEpoch}/${epoch}`);
    }
    if (operation.kind !== "event") {
      throw new Error(`unsupported text-control operation: ${String(operation.kind)}`);
    }
    const controlRef = getRequiredString(request, "control_ref");
    if (controlRef !== record.text_control_ref) {
      throw new Error(`text-control reference mismatch: expected ${record.text_control_ref}/${controlRef}`);
    }
    const state2 = record.text_control;
    if (!state2 || state2.disposed || !state2.connection) {
      throw new Error("text-control host lease is unavailable");
    }
    const eventObject = recordOf(request.event);
    if (!eventObject)
      throw new Error("text-control event must be a JSON object");
    const eventType = getRequiredString(eventObject, "type");
    if (!["input", "change", "compositionstart", "compositionupdate", "compositionend"].includes(eventType)) {
      throw new Error(`unsupported text-control event: ${eventType}`);
    }
    const value = stringValue(eventObject.value);
    if (value === null)
      throw new Error("text-control event value must be a string");
    const disabled = state2.patch.disabled === true;
    const readOnly = state2.patch.readOnly === true;
    if (disabled || readOnly && eventType !== "change")
      return;
    const composing = typeof eventObject.composing === "boolean" ? eventObject.composing : eventType === "compositionstart" || eventType === "compositionupdate";
    if (eventType === "compositionstart" || eventType === "compositionupdate")
      state2.composing = true;
    if (eventType === "compositionend")
      state2.composing = false;
    state2.value = value.replace(/\r\n/g, `
`).replace(/\r/g, `
`);
    const selectionObject = recordOf(request.selection);
    if (selectionObject) {
      const start = numberValue(selectionObject.start);
      const end = numberValue(selectionObject.end);
      if (start === null || end === null || start < 0 || end < 0) {
        throw new Error("text-control selection must contain non-negative safe integers");
      }
      const direction = selectionObject.direction;
      state2.selection = {
        start: Math.min(start, state2.value.length),
        end: Math.min(end, state2.value.length),
        direction: direction === "forward" || direction === "backward" ? direction : "none"
      };
    } else {
      state2.selection = {
        ...state2.selection,
        start: Math.min(state2.selection.start, state2.value.length),
        end: Math.min(state2.selection.end, state2.value.length)
      };
    }
    state2.connection.onEvent(Object.freeze({
      type: eventType,
      value: state2.value,
      composing,
      data: typeof eventObject.data === "string" ? eventObject.data : null,
      inputType: typeof eventObject.inputType === "string" ? eventObject.inputType : null
    }));
    if (eventType === "compositionend" && state2.deferredValue !== null) {
      replaceTextControlValue(state2, state2.deferredValue);
      state2.deferredValue = null;
    }
  }
  function setProps(command) {
    const record = sessionFor(command);
    record.props = jsonObject(command.props ?? {});
    record.session?.controller.applyRawProps(record.props);
    record.session?.controller.update();
  }
  function remount(command) {
    const record = sessionFor(command);
    const session2 = record.session;
    if (!session2)
      throw new Error("session is not ready to remount");
    record.pending_commit = undefined;
    cancelDelayTasks(record);
    session2.unmount();
    session2.mount().catch((error5) => {
      emitDiagnostic(record, "runtime-mount-failed", String(error5), true);
    });
  }
  function advanceSessionTime(command) {
    const record = sessionFor(command);
    advanceTime(record, getRequiredNumber(command, "milliseconds"));
  }
  function unmount(command) {
    const record = sessionFor(command);
    const epoch = getRequiredNumber(command, "view_epoch");
    if (record.session && record.session.mountEpoch !== epoch) {
      throw new Error(`stale unmount request: expected ${record.session.mountEpoch}/${epoch}`);
    }
    cancelDelayTasks(record);
    record.session?.unmount().catch((error5) => {
      emitDiagnostic(record, "runtime-unmount-failed", String(error5), true);
    });
  }
  function dispose(command) {
    const record = sessionFor(command);
    record.disposed = true;
    cancelDelayTasks(record);
    record.scheduled_tasks.splice(0);
    for (const unsubscribe of record.state_unsubs.splice(0))
      unsubscribe();
    record.session?.dispose().catch((error5) => {
      emitDiagnostic(record, "runtime-dispose-failed", String(error5), true);
    });
    sessions.delete(record.session_id);
  }
  function registryEvent() {
    return {
      type: "registry",
      proto_ui: PROTO_UI_VERSION,
      keys: Object.keys(registry).sort()
    };
  }
  function dispatch(serialized) {
    if (bridgeFailed)
      throw new Error("bridge is terminally failed after message overflow");
    const command = recordOf(JSON.parse(serialized));
    if (!command)
      throw new Error("bridge command must be a JSON object");
    const directEvents = [];
    switch (command.type) {
      case "registry":
        directEvents.push(registryEvent());
        break;
      case "start":
        startSession(command);
        break;
      case "projection_ack":
        acknowledge(command);
        break;
      case "input":
        input(command);
        break;
      case "text_control":
        textControl(command);
        break;
      case "advance_time":
        advanceSessionTime(command);
        break;
      case "set_props":
        setProps(command);
        break;
      case "remount":
        remount(command);
        break;
      case "unmount":
        unmount(command);
        break;
      case "dispose":
        dispose(command);
        break;
      default:
        throw new Error(`unknown bridge command: ${String(command.type)}`);
    }
    flushScheduledTasks();
    const events = [...directEvents];
    for (const record of sessions.values()) {
      if (record.events.length === 0)
        continue;
      events.push(...record.events);
    }
    const output = JSON.stringify(events);
    if (utf8Length(output) > MAX_BRIDGE_MESSAGE_BYTES) {
      bridgeFailed = true;
      return JSON.stringify([
        {
          type: "diagnostic",
          diagnostic: {
            code: "message-overflow",
            detail: `bridge response exceeds ${MAX_BRIDGE_MESSAGE_BYTES} bytes`,
            fatal: true
          }
        }
      ]);
    }
    for (const record of sessions.values())
      record.events.splice(0);
    return output;
  }
  var bridge = { dispatch };
  globalThis.__sailbreak_proto_ui_bridge_v1 = bridge;
})();
