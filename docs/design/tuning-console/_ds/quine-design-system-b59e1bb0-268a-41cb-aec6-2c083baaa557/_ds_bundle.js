/* @ds-bundle: {"format":3,"namespace":"QUINEDesignSystem_b59e1b","components":[{"name":"Avatar","sourcePath":"components/core/Avatar.jsx"},{"name":"Badge","sourcePath":"components/core/Badge.jsx"},{"name":"Button","sourcePath":"components/core/Button.jsx"},{"name":"Callout","sourcePath":"components/core/Callout.jsx"},{"name":"Card","sourcePath":"components/core/Card.jsx"},{"name":"IconButton","sourcePath":"components/core/IconButton.jsx"},{"name":"Spinner","sourcePath":"components/feedback/Spinner.jsx"},{"name":"Tooltip","sourcePath":"components/feedback/Tooltip.jsx"},{"name":"Checkbox","sourcePath":"components/forms/Checkbox.jsx"},{"name":"Input","sourcePath":"components/forms/Input.jsx"},{"name":"Select","sourcePath":"components/forms/Select.jsx"},{"name":"Switch","sourcePath":"components/forms/Switch.jsx"},{"name":"Tabs","sourcePath":"components/navigation/Tabs.jsx"},{"name":"ConsoleApp","sourcePath":"ui_kits/console/ConsoleApp.jsx"},{"name":"LandingPage","sourcePath":"ui_kits/landing/LandingPage.jsx"},{"name":"NibliApp","sourcePath":"ui_kits/nibli/NibliApp.jsx"}],"sourceHashes":{"components/core/Avatar.jsx":"db94e8672c11","components/core/Badge.jsx":"17ba9b934e6e","components/core/Button.jsx":"7577f29fc2d8","components/core/Callout.jsx":"33cd6b588f56","components/core/Card.jsx":"f8c6b8a5203e","components/core/IconButton.jsx":"8f1b0dafa773","components/core/dom.js":"fe9b73fe6e14","components/feedback/Spinner.jsx":"35970a032076","components/feedback/Tooltip.jsx":"34ee0b19d0d7","components/forms/Checkbox.jsx":"94828c2d0500","components/forms/Input.jsx":"9ab3c5f9e494","components/forms/Select.jsx":"6d6e52384633","components/forms/Switch.jsx":"9f5b61bda2fc","components/navigation/Tabs.jsx":"b882096944a7","q-loader.js":"27e3a5970b56","ui_kits/console/ConsoleApp.jsx":"a47b8faf1e79","ui_kits/landing/LandingPage.jsx":"ad8b8ccdc8b8","ui_kits/nibli/NibliApp.jsx":"c8cd68296aad"},"inlinedExternals":[],"unexposedExports":[{"name":"injectStyle","sourcePath":"components/core/dom.js"}]} */

(() => {

const __ds_ns = (window.QUINEDesignSystem_b59e1b = window.QUINEDesignSystem_b59e1b || {});

const __ds_scope = {};

(__ds_ns.__errors = __ds_ns.__errors || []);

// components/core/dom.js
try { (() => {
// QUINE — tiny DOM helper shared by components. Injects a component's scoped
// CSS exactly once, the first time its module is evaluated in the bundle.
function injectStyle(id, css) {
  if (typeof document === 'undefined') return;
  if (document.getElementById(id)) return;
  const el = document.createElement('style');
  el.id = id;
  el.textContent = css;
  document.head.appendChild(el);
}
Object.assign(__ds_scope, { injectStyle });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/dom.js", error: String((e && e.message) || e) }); }

// components/core/Avatar.jsx
try { (() => {
__ds_scope.injectStyle('q-avatar', `
.q-avatar {
  display: inline-flex; align-items: center; justify-content: center;
  width: 36px; height: 36px; flex: none;
  border-radius: var(--radius-md);
  background: var(--surface-inset);
  color: var(--text-strong);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
  letter-spacing: 0.02em;
  border: var(--border-hair) solid var(--border-subtle);
  overflow: hidden; position: relative;
}
.q-avatar img { width: 100%; height: 100%; object-fit: cover; }
.q-avatar--sm { width: 26px; height: 26px; font-size: var(--text-2xs); border-radius: var(--radius-sm); }
.q-avatar--lg { width: 52px; height: 52px; font-size: var(--text-md); border-radius: var(--radius-lg); }
.q-avatar--round { border-radius: var(--radius-full); }
.q-avatar__status { position: absolute; right: -2px; bottom: -2px; width: 10px; height: 10px; border-radius: 50%; border: 2px solid var(--bg-base); }
`);
const TONES = ['var(--ember-500)', 'var(--phosphor-500)', 'var(--quanta-500)', 'var(--sky-500)', 'var(--amber-500)'];
function initials(name = '?') {
  return name.trim().split(/\s+/).slice(0, 2).map(s => s[0]).join('').toUpperCase();
}

/** Avatar — initials tile (deterministic accent) or image, with optional status dot. */
function Avatar({
  name = '',
  src,
  size = 'md',
  round = false,
  status,
  className = ''
}) {
  const tone = TONES[(name.charCodeAt(0) || 0) % TONES.length];
  return /*#__PURE__*/React.createElement("span", {
    className: ['q-avatar', size !== 'md' ? `q-avatar--${size}` : '', round ? 'q-avatar--round' : '', className].filter(Boolean).join(' '),
    style: !src ? {
      color: tone,
      borderColor: 'color-mix(in oklab, ' + tone + ' 40%, transparent)'
    } : undefined
  }, src ? /*#__PURE__*/React.createElement("img", {
    src: src,
    alt: name
  }) : initials(name), status && /*#__PURE__*/React.createElement("span", {
    className: "q-avatar__status",
    style: {
      background: status === 'online' ? 'var(--alive)' : status === 'away' ? 'var(--warning)' : 'var(--text-faint)'
    }
  }));
}
Object.assign(__ds_scope, { Avatar });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Avatar.jsx", error: String((e && e.message) || e) }); }

// components/core/Badge.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
__ds_scope.injectStyle('q-badge', `
.q-badge {
  display: inline-flex; align-items: center; gap: var(--space-1);
  font-family: var(--font-mono);
  font-size: var(--text-2xs);
  font-weight: var(--weight-medium);
  letter-spacing: var(--tracking-wide);
  text-transform: uppercase;
  height: 1.4rem; padding: 0 var(--space-2);
  border-radius: var(--radius-sm);
  border: var(--border-hair) solid transparent;
  white-space: nowrap;
}
.q-badge__dot { width: 6px; height: 6px; border-radius: var(--radius-full); background: currentColor; }
.q-badge--neutral { color: var(--text-muted); background: var(--surface-inset); border-color: var(--border-subtle); }
.q-badge--ember   { color: var(--ember-300); background: color-mix(in oklab, var(--ember-500) 16%, transparent); border-color: color-mix(in oklab, var(--ember-500) 35%, transparent); }
.q-badge--alive   { color: var(--phosphor-300); background: color-mix(in oklab, var(--phosphor-500) 14%, transparent); border-color: color-mix(in oklab, var(--phosphor-500) 32%, transparent); }
.q-badge--symbol  { color: var(--quanta-300); background: color-mix(in oklab, var(--quanta-500) 16%, transparent); border-color: color-mix(in oklab, var(--quanta-500) 35%, transparent); }
.q-badge--warning { color: var(--amber-500); background: color-mix(in oklab, var(--amber-500) 14%, transparent); border-color: color-mix(in oklab, var(--amber-500) 32%, transparent); }
.q-badge--danger  { color: var(--crimson-500); background: color-mix(in oklab, var(--crimson-500) 14%, transparent); border-color: color-mix(in oklab, var(--crimson-500) 32%, transparent); }
.q-badge--solid   { color: var(--text-on-ember); background: var(--accent); border-color: transparent; }
`);

/** Badge — a compact status/label chip. Optional leading status dot. */
function Badge({
  tone = 'neutral',
  dot = false,
  className = '',
  children,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("span", _extends({
    className: ['q-badge', `q-badge--${tone}`, className].filter(Boolean).join(' ')
  }, rest), dot && /*#__PURE__*/React.createElement("span", {
    className: "q-badge__dot"
  }), children);
}
Object.assign(__ds_scope, { Badge });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Badge.jsx", error: String((e && e.message) || e) }); }

// components/core/Button.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
__ds_scope.injectStyle('q-button', `
.q-btn {
  --_h: var(--control-h);
  display: inline-flex; align-items: center; justify-content: center;
  gap: var(--space-2);
  height: var(--_h);
  padding: 0 var(--space-4);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  letter-spacing: 0.02em;
  line-height: 1;
  border: var(--border-hair) solid transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  white-space: nowrap;
  user-select: none;
  transition: background var(--dur-fast) var(--ease-out),
              border-color var(--dur-fast) var(--ease-out),
              transform var(--dur-fast) var(--ease-out),
              box-shadow var(--dur-fast) var(--ease-out),
              color var(--dur-fast) var(--ease-out);
}
.q-btn:active { transform: translateY(1px); }
.q-btn:disabled, .q-btn[aria-disabled="true"] { opacity: 0.45; cursor: not-allowed; transform: none; }
.q-btn--sm { --_h: var(--control-h-sm); font-size: var(--text-xs); padding: 0 var(--space-3); }
.q-btn--lg { --_h: var(--control-h-lg); font-size: var(--text-base); padding: 0 var(--space-5); }
.q-btn--block { display: flex; width: 100%; }

/* primary — the ember */
.q-btn--primary { background: var(--accent); color: var(--text-on-ember); }
.q-btn--primary:hover:not(:disabled) { background: var(--accent-hover); box-shadow: var(--glow-ember); }
.q-btn--primary:active { background: var(--accent-press); }

/* secondary — outlined on the void */
.q-btn--secondary { background: transparent; color: var(--text-strong); border-color: var(--border-strong); }
.q-btn--secondary:hover:not(:disabled) { background: var(--surface-hover); border-color: var(--accent); color: var(--accent-hover); }

/* ghost — quietest */
.q-btn--ghost { background: transparent; color: var(--text-muted); }
.q-btn--ghost:hover:not(:disabled) { background: var(--surface-hover); color: var(--text-strong); }

/* danger */
.q-btn--danger { background: transparent; color: var(--danger); border-color: color-mix(in oklab, var(--danger) 45%, transparent); }
.q-btn--danger:hover:not(:disabled) { background: color-mix(in oklab, var(--danger) 14%, transparent); border-color: var(--danger); }
`);

/**
 * Button — the primary action control. Mono label, tight radius, ember fill.
 */
function Button({
  variant = 'primary',
  size = 'md',
  block = false,
  disabled = false,
  leading = null,
  trailing = null,
  className = '',
  children,
  ...rest
}) {
  const cls = ['q-btn', `q-btn--${variant}`, size !== 'md' ? `q-btn--${size}` : '', block ? 'q-btn--block' : '', className].filter(Boolean).join(' ');
  return /*#__PURE__*/React.createElement("button", _extends({
    className: cls,
    disabled: disabled
  }, rest), leading, children, trailing);
}
Object.assign(__ds_scope, { Button });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Button.jsx", error: String((e && e.message) || e) }); }

// components/core/Callout.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
__ds_scope.injectStyle('q-callout', `
.q-callout {
  display: flex; gap: var(--space-3);
  padding: var(--space-4);
  border-radius: var(--radius-md);
  border: var(--border-hair) solid var(--border-subtle);
  border-left-width: 3px;
  background: var(--surface-inset);
  font-size: var(--text-sm);
  line-height: var(--leading-normal);
  color: var(--text-body);
}
.q-callout__glyph { font-family: var(--font-mono); font-size: var(--text-md); line-height: 1.2; flex: none; }
.q-callout__body { min-width: 0; }
.q-callout__title { font-family: var(--font-mono); font-weight: var(--weight-semibold); color: var(--text-strong); margin: 0 0 var(--space-1); letter-spacing: 0.01em; }
.q-callout--note    { border-left-color: var(--info); }
.q-callout--note .q-callout__glyph { color: var(--info); }
.q-callout--ok      { border-left-color: var(--success); }
.q-callout--ok .q-callout__glyph { color: var(--success); }
.q-callout--warn    { border-left-color: var(--warning); }
.q-callout--warn .q-callout__glyph { color: var(--warning); }
.q-callout--error   { border-left-color: var(--danger); }
.q-callout--error .q-callout__glyph { color: var(--danger); }
.q-callout--theorem { border-left-color: var(--symbol); background: color-mix(in oklab, var(--quanta-500) 8%, var(--surface-inset)); }
.q-callout--theorem .q-callout__glyph { color: var(--quanta-300); }
`);
const GLYPHS = {
  note: 'ℹ',
  ok: '✓',
  warn: '⚠',
  error: '⊥',
  theorem: '∴'
};

/** Callout — an inline annotation. `theorem` uses the symbolic violet + ∴. */
function Callout({
  kind = 'note',
  title,
  glyph,
  className = '',
  children,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("div", _extends({
    className: ['q-callout', `q-callout--${kind}`, className].filter(Boolean).join(' '),
    role: "note"
  }, rest), /*#__PURE__*/React.createElement("span", {
    className: "q-callout__glyph",
    "aria-hidden": "true"
  }, glyph || GLYPHS[kind]), /*#__PURE__*/React.createElement("div", {
    className: "q-callout__body"
  }, title && /*#__PURE__*/React.createElement("p", {
    className: "q-callout__title"
  }, title), /*#__PURE__*/React.createElement("div", null, children)));
}
Object.assign(__ds_scope, { Callout });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Callout.jsx", error: String((e && e.message) || e) }); }

// components/core/Card.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
__ds_scope.injectStyle('q-card', `
.q-card {
  background: var(--surface-card);
  border: var(--border-hair) solid var(--border-subtle);
  border-radius: var(--radius-lg);
  color: var(--text-body);
  overflow: clip;
  transition: border-color var(--dur-fast) var(--ease-out), box-shadow var(--dur-fast) var(--ease-out);
}
.q-card--pad { padding: var(--space-5); }
.q-card--inset { background: var(--surface-inset); }
.q-card--grid { background-image: var(--bg-grid); background-color: var(--surface-card); }
.q-card--interactive { cursor: pointer; }
.q-card--interactive:hover { border-color: var(--accent); box-shadow: var(--shadow-2); }
.q-card--glow:hover { box-shadow: var(--glow-ember); border-color: transparent; }
/* a hairline ember rule along the top edge, the "active" marker */
.q-card--marked { border-top: var(--border-bold) solid var(--accent); }
.q-card__header { display: flex; align-items: center; justify-content: space-between; gap: var(--space-3); padding: var(--space-4) var(--space-5); border-bottom: var(--border-hair) solid var(--border-subtle); }
.q-card__title { font-family: var(--font-display); font-weight: var(--weight-semibold); color: var(--text-strong); font-size: var(--text-md); margin: 0; }
.q-card__body { padding: var(--space-5); }
.q-card__footer { padding: var(--space-4) var(--space-5); border-top: var(--border-hair) solid var(--border-subtle); display: flex; align-items: center; gap: var(--space-3); }
`);

/** Card — the primary surface. Compose with Card.Header / Body / Footer or use `pad`. */
function Card({
  pad = false,
  inset = false,
  grid = false,
  interactive = false,
  glow = false,
  marked = false,
  className = '',
  children,
  ...rest
}) {
  const cls = ['q-card', pad ? 'q-card--pad' : '', inset ? 'q-card--inset' : '', grid ? 'q-card--grid' : '', interactive ? 'q-card--interactive' : '', glow ? 'q-card--glow' : '', marked ? 'q-card--marked' : '', className].filter(Boolean).join(' ');
  return /*#__PURE__*/React.createElement("div", _extends({
    className: cls
  }, rest), children);
}
Card.Header = function CardHeader({
  title,
  actions,
  children
}) {
  return /*#__PURE__*/React.createElement("div", {
    className: "q-card__header"
  }, title ? /*#__PURE__*/React.createElement("h3", {
    className: "q-card__title"
  }, title) : children, actions);
};
Card.Body = function CardBody({
  className = '',
  children,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("div", _extends({
    className: ['q-card__body', className].filter(Boolean).join(' ')
  }, rest), children);
};
Card.Footer = function CardFooter({
  children
}) {
  return /*#__PURE__*/React.createElement("div", {
    className: "q-card__footer"
  }, children);
};
Object.assign(__ds_scope, { Card });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/Card.jsx", error: String((e && e.message) || e) }); }

// components/core/IconButton.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
__ds_scope.injectStyle('q-iconbutton', `
.q-iconbtn {
  --_s: var(--control-h);
  display: inline-flex; align-items: center; justify-content: center;
  width: var(--_s); height: var(--_s);
  color: var(--text-muted);
  background: transparent;
  border: var(--border-hair) solid transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--dur-fast) var(--ease-out);
}
.q-iconbtn:hover:not(:disabled) { color: var(--text-strong); background: var(--surface-hover); }
.q-iconbtn:active { transform: translateY(1px); }
.q-iconbtn:disabled { opacity: 0.4; cursor: not-allowed; }
.q-iconbtn--sm { --_s: var(--control-h-sm); }
.q-iconbtn--lg { --_s: var(--control-h-lg); }
.q-iconbtn--solid { background: var(--accent); color: var(--text-on-ember); }
.q-iconbtn--solid:hover:not(:disabled) { background: var(--accent-hover); box-shadow: var(--glow-ember); }
.q-iconbtn--outline { border-color: var(--border-strong); }
.q-iconbtn--outline:hover:not(:disabled) { border-color: var(--accent); color: var(--accent-hover); background: transparent; }
`);

/** IconButton — a square, label-less action. Pass an icon as children. */
function IconButton({
  variant = 'ghost',
  size = 'md',
  label,
  className = '',
  children,
  ...rest
}) {
  const cls = ['q-iconbtn', variant !== 'ghost' ? `q-iconbtn--${variant}` : '', size !== 'md' ? `q-iconbtn--${size}` : '', className].filter(Boolean).join(' ');
  return /*#__PURE__*/React.createElement("button", _extends({
    className: cls,
    "aria-label": label,
    title: label
  }, rest), children);
}
Object.assign(__ds_scope, { IconButton });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/core/IconButton.jsx", error: String((e && e.message) || e) }); }

// components/feedback/Spinner.jsx
try { (() => {
__ds_scope.injectStyle('q-spinner', `
.q-spinner { display: inline-flex; align-items: center; gap: var(--space-2); font-family: var(--font-mono); font-size: var(--text-sm); color: var(--text-muted); }
.q-spinner__glyph {
  width: 1em; height: 1em; border-radius: 50%;
  border: 2px solid var(--border-strong);
  border-top-color: var(--accent);
  animation: q-spin 0.7s var(--ease-linear) infinite;
}
.q-spinner--alive .q-spinner__glyph { border-top-color: var(--alive); }
@keyframes q-spin { to { transform: rotate(360deg); } }

/* the alternative: a terminal scan dots loader */
.q-dots::after { content: ""; animation: q-dots 1.2s steps(4, end) infinite; }
@keyframes q-dots { 0% { content: ""; } 25% { content: "."; } 50% { content: ".."; } 75% { content: "..."; } }
`);

/** Spinner — a quiet ring loader with optional label. `dots` uses a terminal "…" tick. */
function Spinner({
  label,
  tone = 'ember',
  dots = false,
  className = ''
}) {
  return /*#__PURE__*/React.createElement("span", {
    className: ['q-spinner', tone === 'alive' ? 'q-spinner--alive' : '', className].filter(Boolean).join(' '),
    role: "status"
  }, !dots && /*#__PURE__*/React.createElement("span", {
    className: "q-spinner__glyph"
  }), label && /*#__PURE__*/React.createElement("span", null, label, dots && /*#__PURE__*/React.createElement("span", {
    className: "q-dots"
  })));
}
Object.assign(__ds_scope, { Spinner });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/Spinner.jsx", error: String((e && e.message) || e) }); }

// components/feedback/Tooltip.jsx
try { (() => {
__ds_scope.injectStyle('q-tooltip', `
.q-tooltip { position: relative; display: inline-flex; }
.q-tooltip__pop {
  position: absolute; z-index: 50;
  bottom: calc(100% + 8px); left: 50%; transform: translateX(-50%) translateY(4px);
  background: var(--void-1000);
  color: var(--text-strong);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  line-height: 1.4;
  padding: var(--space-2) var(--space-3);
  border: var(--border-hair) solid var(--border-strong);
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow-2);
  white-space: nowrap;
  opacity: 0; pointer-events: none;
  transition: opacity var(--dur-fast) var(--ease-out), transform var(--dur-fast) var(--ease-out);
}
.q-tooltip:hover .q-tooltip__pop, .q-tooltip:focus-within .q-tooltip__pop { opacity: 1; transform: translateX(-50%) translateY(0); }
.q-tooltip__pop::after {
  content: ""; position: absolute; top: 100%; left: 50%; transform: translateX(-50%);
  border: 5px solid transparent; border-top-color: var(--border-strong);
}
`);

/** Tooltip — hover/focus hint. Wraps a single trigger element. */
function Tooltip({
  label,
  children,
  className = ''
}) {
  return /*#__PURE__*/React.createElement("span", {
    className: ['q-tooltip', className].filter(Boolean).join(' '),
    tabIndex: 0
  }, children, /*#__PURE__*/React.createElement("span", {
    className: "q-tooltip__pop",
    role: "tooltip"
  }, label));
}
Object.assign(__ds_scope, { Tooltip });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/feedback/Tooltip.jsx", error: String((e && e.message) || e) }); }

// components/forms/Checkbox.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
__ds_scope.injectStyle('q-check', `
.q-check { display: inline-flex; align-items: center; gap: var(--space-3); cursor: pointer; user-select: none; }
.q-check input { position: absolute; opacity: 0; width: 0; height: 0; }
.q-check__box {
  width: 18px; height: 18px; flex: none;
  display: flex; align-items: center; justify-content: center;
  background: var(--surface-inset);
  border: var(--border-hair) solid var(--border-strong);
  border-radius: var(--radius-sm);
  color: var(--text-on-ember);
  transition: background var(--dur-fast) var(--ease-out), border-color var(--dur-fast) var(--ease-out);
}
.q-check__box svg { width: 12px; height: 12px; opacity: 0; transform: scale(0.6); transition: all var(--dur-fast) var(--ease-spring); }
.q-check input:checked + .q-check__box { background: var(--accent); border-color: transparent; }
.q-check input:checked + .q-check__box svg { opacity: 1; transform: scale(1); }
.q-check--radio .q-check__box { border-radius: var(--radius-full); }
.q-check--radio .q-check__dot { width: 8px; height: 8px; border-radius: 50%; background: var(--text-on-ember); opacity: 0; transform: scale(0.4); transition: all var(--dur-fast) var(--ease-spring); }
.q-check--radio input:checked + .q-check__box .q-check__dot { opacity: 1; transform: scale(1); }
.q-check input:focus-visible + .q-check__box { box-shadow: 0 0 0 3px var(--accent-soft); }
.q-check input:disabled + .q-check__box { opacity: 0.45; }
.q-check__label { font-family: var(--font-mono); font-size: var(--text-sm); color: var(--text-body); }
`);
const CHECK = /*#__PURE__*/React.createElement("svg", {
  viewBox: "0 0 12 12",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: "2.4",
  strokeLinecap: "round",
  strokeLinejoin: "round",
  "aria-hidden": "true"
}, /*#__PURE__*/React.createElement("path", {
  d: "M2 6.5 L5 9.5 L10 2.5"
}));

/** Checkbox / Radio — square check or round dot. Set `radio` for single-select. */
function Checkbox({
  radio = false,
  label,
  className = '',
  ...rest
}) {
  return /*#__PURE__*/React.createElement("label", {
    className: ['q-check', radio ? 'q-check--radio' : '', className].filter(Boolean).join(' ')
  }, /*#__PURE__*/React.createElement("input", _extends({
    type: radio ? 'radio' : 'checkbox'
  }, rest)), /*#__PURE__*/React.createElement("span", {
    className: "q-check__box"
  }, radio ? /*#__PURE__*/React.createElement("span", {
    className: "q-check__dot"
  }) : CHECK), label && /*#__PURE__*/React.createElement("span", {
    className: "q-check__label"
  }, label));
}
Object.assign(__ds_scope, { Checkbox });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Checkbox.jsx", error: String((e && e.message) || e) }); }

// components/forms/Input.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
__ds_scope.injectStyle('q-input', `
.q-field { display: flex; flex-direction: column; gap: var(--space-2); }
.q-field__label { font-family: var(--font-mono); font-size: var(--text-2xs); text-transform: uppercase; letter-spacing: var(--tracking-caps); color: var(--text-muted); }
.q-field__hint { font-family: var(--font-mono); font-size: var(--text-xs); color: var(--text-faint); }
.q-field__hint--error { color: var(--danger); }
.q-input-wrap { position: relative; display: flex; align-items: center; }
.q-input-wrap__affix { position: absolute; color: var(--text-faint); font-family: var(--font-mono); font-size: var(--text-sm); pointer-events: none; }
.q-input-wrap__affix--lead { left: var(--space-3); }
.q-input {
  width: 100%;
  height: var(--control-h);
  padding: 0 var(--space-3);
  background: var(--surface-inset);
  color: var(--text-strong);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  border: var(--border-hair) solid var(--border-subtle);
  border-radius: var(--radius-md);
  transition: border-color var(--dur-fast) var(--ease-out), box-shadow var(--dur-fast) var(--ease-out);
}
.q-input:hover:not(:disabled) { border-color: var(--border-strong); }
.q-input:focus { outline: none; border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
.q-input::placeholder { color: var(--text-faint); }
.q-input:disabled { opacity: 0.5; cursor: not-allowed; }
.q-input--lead { padding-left: calc(var(--space-3) * 2 + 0.8em); }
.q-input--error { border-color: var(--danger); }
.q-input--error:focus { box-shadow: 0 0 0 3px color-mix(in oklab, var(--danger) 22%, transparent); }
textarea.q-input { height: auto; padding: var(--space-3); line-height: var(--leading-normal); resize: vertical; }
`);

/** Input — text field with optional label, hint, error, and leading affix. */
function Input({
  label,
  hint,
  error,
  lead,
  id,
  className = '',
  textarea = false,
  rows = 4,
  ...rest
}) {
  const fieldId = id || `q-in-${Math.random().toString(36).slice(2, 8)}`;
  const inputCls = ['q-input', lead ? 'q-input--lead' : '', error ? 'q-input--error' : '', className].filter(Boolean).join(' ');
  const control = textarea ? /*#__PURE__*/React.createElement("textarea", _extends({
    id: fieldId,
    className: inputCls,
    rows: rows
  }, rest)) : /*#__PURE__*/React.createElement("input", _extends({
    id: fieldId,
    className: inputCls
  }, rest));
  return /*#__PURE__*/React.createElement("div", {
    className: "q-field"
  }, label && /*#__PURE__*/React.createElement("label", {
    className: "q-field__label",
    htmlFor: fieldId
  }, label), /*#__PURE__*/React.createElement("div", {
    className: "q-input-wrap"
  }, lead && /*#__PURE__*/React.createElement("span", {
    className: "q-input-wrap__affix q-input-wrap__affix--lead"
  }, lead), control), (hint || error) && /*#__PURE__*/React.createElement("span", {
    className: `q-field__hint ${error ? 'q-field__hint--error' : ''}`
  }, error || hint));
}
Object.assign(__ds_scope, { Input });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Input.jsx", error: String((e && e.message) || e) }); }

// components/forms/Select.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
__ds_scope.injectStyle('q-select', `
.q-select-wrap { position: relative; display: inline-flex; width: 100%; }
.q-select {
  appearance: none; width: 100%;
  height: var(--control-h);
  padding: 0 calc(var(--space-5) + var(--space-2)) 0 var(--space-3);
  background: var(--surface-inset);
  color: var(--text-strong);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  border: var(--border-hair) solid var(--border-subtle);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: border-color var(--dur-fast) var(--ease-out), box-shadow var(--dur-fast) var(--ease-out);
}
.q-select:hover:not(:disabled) { border-color: var(--border-strong); }
.q-select:focus { outline: none; border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
.q-select:disabled { opacity: 0.5; cursor: not-allowed; }
.q-select-wrap__chev { position: absolute; right: var(--space-3); top: 50%; transform: translateY(-50%); pointer-events: none; color: var(--text-muted); font-family: var(--font-mono); font-size: 11px; }
`);

/** Select — native dropdown styled to the system, with a mono chevron. */
function Select({
  options = [],
  placeholder,
  className = '',
  children,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("div", {
    className: "q-select-wrap"
  }, /*#__PURE__*/React.createElement("select", _extends({
    className: ['q-select', className].filter(Boolean).join(' ')
  }, rest), placeholder && /*#__PURE__*/React.createElement("option", {
    value: "",
    disabled: true
  }, placeholder), children || options.map(o => {
    const opt = typeof o === 'string' ? {
      value: o,
      label: o
    } : o;
    return /*#__PURE__*/React.createElement("option", {
      key: opt.value,
      value: opt.value
    }, opt.label);
  })), /*#__PURE__*/React.createElement("span", {
    className: "q-select-wrap__chev",
    "aria-hidden": "true"
  }, "\u25BE"));
}
Object.assign(__ds_scope, { Select });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Select.jsx", error: String((e && e.message) || e) }); }

// components/forms/Switch.jsx
try { (() => {
function _extends() { return _extends = Object.assign ? Object.assign.bind() : function (n) { for (var e = 1; e < arguments.length; e++) { var t = arguments[e]; for (var r in t) ({}).hasOwnProperty.call(t, r) && (n[r] = t[r]); } return n; }, _extends.apply(null, arguments); }
__ds_scope.injectStyle('q-switch', `
.q-switch { display: inline-flex; align-items: center; gap: var(--space-3); cursor: pointer; user-select: none; }
.q-switch input { position: absolute; opacity: 0; width: 0; height: 0; }
.q-switch__track {
  position: relative; width: 38px; height: 22px; flex: none;
  background: var(--surface-inset);
  border: var(--border-hair) solid var(--border-strong);
  border-radius: var(--radius-full);
  transition: background var(--dur-normal) var(--ease-out), border-color var(--dur-normal) var(--ease-out);
}
.q-switch__thumb {
  position: absolute; top: 2px; left: 2px; width: 16px; height: 16px;
  background: var(--text-muted); border-radius: var(--radius-full);
  transition: transform var(--dur-normal) var(--ease-spring), background var(--dur-normal) var(--ease-out);
}
.q-switch input:checked + .q-switch__track { background: var(--accent); border-color: transparent; }
.q-switch input:checked + .q-switch__track .q-switch__thumb { transform: translateX(16px); background: var(--text-on-ember); }
.q-switch input:focus-visible + .q-switch__track { box-shadow: 0 0 0 3px var(--accent-soft); }
.q-switch input:disabled + .q-switch__track { opacity: 0.45; }
.q-switch__label { font-family: var(--font-mono); font-size: var(--text-sm); color: var(--text-body); }
`);

/** Switch — a binary toggle. Track turns ember when on; thumb springs across. */
function Switch({
  checked,
  defaultChecked,
  onChange,
  disabled,
  label,
  ...rest
}) {
  return /*#__PURE__*/React.createElement("label", {
    className: "q-switch"
  }, /*#__PURE__*/React.createElement("input", _extends({
    type: "checkbox",
    checked: checked,
    defaultChecked: defaultChecked,
    onChange: onChange,
    disabled: disabled
  }, rest)), /*#__PURE__*/React.createElement("span", {
    className: "q-switch__track"
  }, /*#__PURE__*/React.createElement("span", {
    className: "q-switch__thumb"
  })), label && /*#__PURE__*/React.createElement("span", {
    className: "q-switch__label"
  }, label));
}
Object.assign(__ds_scope, { Switch });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/forms/Switch.jsx", error: String((e && e.message) || e) }); }

// components/navigation/Tabs.jsx
try { (() => {
__ds_scope.injectStyle('q-tabs', `
.q-tabs { display: flex; gap: 2px; border-bottom: var(--border-hair) solid var(--border-subtle); }
.q-tab {
  position: relative;
  padding: var(--space-3) var(--space-4);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  letter-spacing: 0.01em;
  color: var(--text-muted);
  background: none; border: none; cursor: pointer;
  display: inline-flex; align-items: center; gap: var(--space-2);
  transition: color var(--dur-fast) var(--ease-out);
}
.q-tab:hover { color: var(--text-strong); }
.q-tab[aria-selected="true"] { color: var(--text-strong); }
.q-tab__underline {
  position: absolute; left: 0; right: 0; bottom: -1px; height: 2px;
  background: var(--accent); border-radius: 2px 2px 0 0;
  transform: scaleX(0); transform-origin: center;
  transition: transform var(--dur-normal) var(--ease-out);
}
.q-tab[aria-selected="true"] .q-tab__underline { transform: scaleX(1); }
.q-tab__count { font-size: var(--text-2xs); color: var(--text-faint); }
`);

/** Tabs — underline navigation. Controlled via `value`/`onChange`, or uncontrolled. */
function Tabs({
  tabs = [],
  value,
  defaultValue,
  onChange,
  className = ''
}) {
  const [internal, setInternal] = React.useState(defaultValue ?? (tabs[0] && (tabs[0].id ?? tabs[0])));
  const active = value !== undefined ? value : internal;
  const select = id => {
    if (value === undefined) setInternal(id);
    onChange && onChange(id);
  };
  return /*#__PURE__*/React.createElement("div", {
    className: ['q-tabs', className].filter(Boolean).join(' '),
    role: "tablist"
  }, tabs.map(t => {
    const tab = typeof t === 'string' ? {
      id: t,
      label: t
    } : t;
    return /*#__PURE__*/React.createElement("button", {
      key: tab.id,
      role: "tab",
      "aria-selected": active === tab.id,
      className: "q-tab",
      onClick: () => select(tab.id)
    }, tab.label, tab.count != null && /*#__PURE__*/React.createElement("span", {
      className: "q-tab__count"
    }, tab.count), /*#__PURE__*/React.createElement("span", {
      className: "q-tab__underline"
    }));
  }));
}
Object.assign(__ds_scope, { Tabs });
})(); } catch (e) { __ds_ns.__errors.push({ path: "components/navigation/Tabs.jsx", error: String((e && e.message) || e) }); }

// q-loader.js
try { (() => {
/* ════════════════════════════════════════════════════════════════════
   QUINE — q-loader.js  ·  preview-time module loader
   ────────────────────────────────────────────────────────────────────
   The design-system compiler emits a single `_ds_bundle.js` that consuming
   projects load. Inside THIS authoring project's file-preview, that virtual
   artifact isn't served, so cards/screens would render blank.

   This loader makes the @dsCard HTML and UI-kit screens self-sufficient:
   it fetches the real ES-module component source (.jsx), transpiles it with
   the already-loaded @babel/standalone, runs it through a tiny CommonJS
   shim, and merges the exports onto window.QUINEDesignSystem_b59e1b — the
   same namespace the compiled bundle would populate.

   Requires React + ReactDOM (+ optional) and @babel/standalone loaded first.

       QLoad(['./LandingPage.jsx'])
         .then(ns => ReactDOM.createRoot(el).render(React.createElement(ns.LandingPage)));
   ════════════════════════════════════════════════════════════════════ */
(function () {
  var NS = 'QUINEDesignSystem_b59e1b';
  function specsOf(src) {
    var out = [];
    var re = /(?:import|export)[\s\S]*?from\s*['"]([^'"]+)['"]/g,
      m;
    while (m = re.exec(src)) out.push(m[1]);
    var re2 = /import\s+['"]([^'"]+)['"]/g;
    while (m = re2.exec(src)) out.push(m[1]);
    return out;
  }
  function isBare(spec) {
    return !(spec[0] === '.' || spec[0] === '/');
  }
  window.QLoad = async function QLoad(entryRels) {
    if (!window.Babel) throw new Error('q-loader: @babel/standalone must load first');
    var mods = {}; // absURL -> { code, deps:{spec:absURL}, exports, done }

    async function gather(absURL) {
      if (mods[absURL]) return;
      mods[absURL] = {
        pending: true
      };
      var res = await fetch(absURL);
      if (!res.ok) throw new Error('q-loader: cannot fetch ' + absURL + ' (' + res.status + ')');
      var src = await res.text();
      var specs = specsOf(src);
      var deps = {};
      for (var i = 0; i < specs.length; i++) {
        if (!isBare(specs[i])) deps[specs[i]] = new URL(specs[i], absURL).href;
      }
      var out = window.Babel.transform(src, {
        filename: absURL,
        presets: [['react', {
          runtime: 'classic'
        }], ['env', {
          modules: 'commonjs',
          targets: {
            esmodules: true
          }
        }]]
      });
      mods[absURL] = {
        code: out.code,
        deps: deps,
        exports: {},
        done: false
      };
      for (var k in deps) {
        await gather(deps[k]);
      }
    }
    function evalMod(absURL) {
      var m = mods[absURL];
      if (m.done) return m.exports;
      m.done = true;
      var module = {
        exports: m.exports
      };
      var require = function (spec) {
        if (spec === 'react') return window.React;
        if (spec === 'react-dom' || spec === 'react-dom/client') return window.ReactDOM;
        if (m.deps[spec]) return evalMod(m.deps[spec]);
        return {};
      };
      var fn = new Function('require', 'module', 'exports', 'React', 'ReactDOM', m.code);
      fn(require, module, module.exports, window.React, window.ReactDOM);
      m.exports = module.exports;
      return m.exports;
    }
    var entryURLs = entryRels.map(function (e) {
      return new URL(e, document.baseURI).href;
    });
    for (var i = 0; i < entryURLs.length; i++) await gather(entryURLs[i]);
    var ns = window[NS] || (window[NS] = {});
    for (var j = 0; j < entryURLs.length; j++) {
      var ex = evalMod(entryURLs[j]);
      for (var name in ex) {
        if (name !== '__esModule' && name !== 'default') ns[name] = ex[name];
      }
    }
    return ns;
  };
})();
})(); } catch (e) { __ds_ns.__errors.push({ path: "q-loader.js", error: String((e && e.message) || e) }); }

// ui_kits/console/ConsoleApp.jsx
try { (() => {
__ds_scope.injectStyle('q-console', `
.cs { display: grid; grid-template-columns: 232px 1fr; height: 760px; background: var(--bg-base); color: var(--text-body); font-family: var(--font-sans); }

/* sidebar */
.cs-side { background: var(--bg-sunken); border-right: 1px solid var(--border-subtle); display: flex; flex-direction: column; padding: 16px 12px; gap: 6px; }
.cs-side__brand { display: flex; align-items: center; gap: 10px; padding: 6px 8px 16px; font-family: var(--font-display); font-weight: 600; font-size: 17px; letter-spacing: -0.03em; color: var(--text-strong); }
.cs-side__brand img { width: 24px; height: 24px; }
.cs-side__sec { font-family: var(--font-mono); font-size: 10px; letter-spacing: 0.14em; text-transform: uppercase; color: var(--text-faint); padding: 14px 10px 6px; }
.cs-nav { display: flex; align-items: center; gap: 10px; padding: 8px 10px; border-radius: var(--radius-md); font-family: var(--font-mono); font-size: 13px; color: var(--text-muted); cursor: pointer; border: 1px solid transparent; }
.cs-nav:hover { background: var(--surface-hover); color: var(--text-strong); }
.cs-nav--on { background: var(--accent-soft); color: var(--text-strong); border-color: color-mix(in oklab, var(--accent) 30%, transparent); }
.cs-nav__glyph { width: 18px; text-align: center; color: var(--text-faint); }
.cs-nav--on .cs-nav__glyph { color: var(--accent); }
.cs-nav__count { margin-left: auto; font-size: 11px; color: var(--text-faint); }
.cs-side__foot { margin-top: auto; display: flex; align-items: center; gap: 10px; padding: 10px 8px; border-top: 1px solid var(--border-subtle); font-family: var(--font-mono); font-size: 12px; color: var(--text-muted); }

/* main */
.cs-main { display: flex; flex-direction: column; min-width: 0; }
.cs-top { display: flex; align-items: center; gap: 14px; height: 60px; padding: 0 24px; border-bottom: 1px solid var(--border-subtle); }
.cs-top__title { font-family: var(--font-display); font-weight: 600; font-size: 19px; color: var(--text-strong); letter-spacing: -0.02em; }
.cs-top__sp { flex: 1; }
.cs-search { width: 240px; }
.cs-body { flex: 1; overflow: auto; padding: 24px; }

/* overview */
.cs-statgrid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px; margin-bottom: 18px; }
.cs-stat__lab { font-family: var(--font-mono); font-size: 11px; letter-spacing: 0.08em; text-transform: uppercase; color: var(--text-muted); }
.cs-stat__num { font-family: var(--font-display); font-weight: 600; font-size: 30px; color: var(--text-strong); letter-spacing: -0.02em; margin-top: 8px; }
.cs-stat__num small { font-size: 15px; color: var(--text-muted); }
.cs-stat__delta { font-family: var(--font-mono); font-size: 11px; margin-top: 6px; }
.cs-cols { display: grid; grid-template-columns: 1.3fr 1fr; gap: 14px; }

/* topology */
.cs-topo { position: relative; height: 280px; background-image: var(--bg-grid); background-color: var(--surface-card); border-radius: var(--radius-md); overflow: hidden; }
.cs-node { position: absolute; transform: translate(-50%, -50%); display: flex; flex-direction: column; align-items: center; gap: 5px; cursor: pointer; }
.cs-node__dot { width: 16px; height: 16px; border-radius: 50%; background: var(--alive); box-shadow: var(--glow-phosphor); border: 2px solid var(--bg-base); transition: transform var(--dur-fast) var(--ease-spring); }
.cs-node--leader .cs-node__dot { background: var(--accent); box-shadow: var(--glow-ember); }
.cs-node--down .cs-node__dot { background: var(--text-faint); box-shadow: none; }
.cs-node--sel .cs-node__dot { transform: scale(1.5); }
.cs-node__lab { font-family: var(--font-mono); font-size: 10px; color: var(--text-muted); }

/* log */
.cs-log { font-family: var(--font-mono); font-size: 12.5px; line-height: 1.9; }
.cs-log__row { display: flex; gap: 14px; padding: 2px 0; border-bottom: 1px solid color-mix(in oklab, var(--border-subtle) 50%, transparent); }
.cs-log__ts { color: var(--text-faint); flex: none; }
.cs-log__lvl { flex: none; width: 52px; }
.cs-log__lvl--info { color: var(--sky-500); } .cs-log__lvl--ok { color: var(--alive); } .cs-log__lvl--warn { color: var(--amber-500); } .cs-log__lvl--err { color: var(--danger); }
.cs-log__msg { color: var(--text-body); }

/* table */
.cs-table { width: 100%; border-collapse: collapse; font-family: var(--font-mono); font-size: 13px; }
.cs-table th { text-align: left; font-size: 10.5px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--text-faint); font-weight: 500; padding: 10px 12px; border-bottom: 1px solid var(--border-subtle); }
.cs-table td { padding: 11px 12px; border-bottom: 1px solid color-mix(in oklab, var(--border-subtle) 60%, transparent); color: var(--text-body); }
.cs-table tr:hover td { background: var(--surface-hover); }
.cs-table__id { color: var(--text-strong); }

.cs-cfg { max-width: 460px; display: flex; flex-direction: column; gap: 22px; }
.cs-cfg__group { display: flex; flex-direction: column; gap: 12px; }
.cs-cfg__h { font-family: var(--font-mono); font-size: 11px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--accent); }
`);
const NODES = [{
  id: 'n0',
  region: 'us-east',
  x: 50,
  y: 22,
  role: 'leader',
  lat: 0
}, {
  id: 'n1',
  region: 'us-west',
  x: 22,
  y: 52,
  role: 'follower',
  lat: 31
}, {
  id: 'n2',
  region: 'eu-central',
  x: 78,
  y: 48,
  role: 'follower',
  lat: 84
}, {
  id: 'n3',
  region: 'ap-south',
  x: 36,
  y: 82,
  role: 'follower',
  lat: 142
}, {
  id: 'n4',
  region: 'sa-east',
  x: 70,
  y: 80,
  role: 'down',
  lat: null
}];
const LINKS = [[0, 1], [0, 2], [0, 3], [0, 4], [1, 3], [2, 4]];
const BASE_LOG = [{
  ts: '14:22:01.382',
  lvl: 'ok',
  l: 'commit',
  m: 'round 4182 committed · 5/5 ack · 38ms'
}, {
  ts: '14:22:01.344',
  lvl: 'info',
  l: 'gossip',
  m: 'n2 ← heartbeat ← n0 (rtt 84ms)'
}, {
  ts: '14:21:59.901',
  lvl: 'warn',
  l: 'lag',
  m: 'n3 applying behind by 2 entries'
}, {
  ts: '14:21:58.220',
  lvl: 'err',
  l: 'partition',
  m: 'n4 unreachable · marked down · quorum holds (4/5)'
}, {
  ts: '14:21:57.118',
  lvl: 'info',
  l: 'propose',
  m: 'n0 proposes entry 4182 ("∀ peers: agree")'
}, {
  ts: '14:21:55.640',
  lvl: 'ok',
  l: 'elect',
  m: 'n0 won election · term 7 · ∴ leader'
}];
const NAV = [{
  id: 'overview',
  label: 'Overview',
  g: '◫'
}, {
  id: 'nodes',
  label: 'Nodes',
  g: '⬡',
  count: '5'
}, {
  id: 'log',
  label: 'Consensus log',
  g: '≣'
}, {
  id: 'config',
  label: 'Config',
  g: 'λ'
}];
function StatCard({
  lab,
  num,
  unit,
  delta,
  deltaTone
}) {
  return /*#__PURE__*/React.createElement(__ds_scope.Card, {
    pad: true
  }, /*#__PURE__*/React.createElement("div", {
    className: "cs-stat__lab"
  }, lab), /*#__PURE__*/React.createElement("div", {
    className: "cs-stat__num"
  }, num, unit && /*#__PURE__*/React.createElement("small", null, unit)), delta && /*#__PURE__*/React.createElement("div", {
    className: "cs-stat__delta",
    style: {
      color: deltaTone === 'up' ? 'var(--alive)' : deltaTone === 'down' ? 'var(--danger)' : 'var(--text-faint)'
    }
  }, delta));
}
function Topology({
  sel,
  setSel
}) {
  return /*#__PURE__*/React.createElement("div", {
    className: "cs-topo"
  }, /*#__PURE__*/React.createElement("svg", {
    width: "100%",
    height: "100%",
    style: {
      position: 'absolute',
      inset: 0
    },
    preserveAspectRatio: "none"
  }, LINKS.map(([a, b], i) => {
    const A = NODES[a],
      B = NODES[b];
    const down = A.role === 'down' || B.role === 'down';
    return /*#__PURE__*/React.createElement("line", {
      key: i,
      x1: `${A.x}%`,
      y1: `${A.y}%`,
      x2: `${B.x}%`,
      y2: `${B.y}%`,
      stroke: down ? 'var(--border-subtle)' : 'color-mix(in oklab, var(--phosphor-500) 35%, transparent)',
      strokeWidth: "1.5",
      strokeDasharray: down ? '4 4' : undefined
    });
  })), NODES.map(n => /*#__PURE__*/React.createElement("div", {
    key: n.id,
    className: ['cs-node', n.role === 'leader' ? 'cs-node--leader' : '', n.role === 'down' ? 'cs-node--down' : '', sel === n.id ? 'cs-node--sel' : ''].filter(Boolean).join(' '),
    style: {
      left: `${n.x}%`,
      top: `${n.y}%`
    },
    onClick: () => setSel(n.id)
  }, /*#__PURE__*/React.createElement("span", {
    className: "cs-node__dot"
  }), /*#__PURE__*/React.createElement("span", {
    className: "cs-node__lab"
  }, n.id, " \xB7 ", n.region))));
}
function Overview({
  sel,
  setSel,
  onForce
}) {
  return /*#__PURE__*/React.createElement(React.Fragment, null, /*#__PURE__*/React.createElement("div", {
    className: "cs-statgrid"
  }, /*#__PURE__*/React.createElement(StatCard, {
    lab: "Nodes alive",
    num: "4",
    unit: "/5",
    delta: "\u2193 n4 down",
    deltaTone: "down"
  }), /*#__PURE__*/React.createElement(StatCard, {
    lab: "Commit p50",
    num: "38",
    unit: "ms",
    delta: "\u25B2 12% slower",
    deltaTone: "down"
  }), /*#__PURE__*/React.createElement(StatCard, {
    lab: "Term",
    num: "7",
    delta: "elected 14:21:55",
    deltaTone: "flat"
  }), /*#__PURE__*/React.createElement(StatCard, {
    lab: "Throughput",
    num: "2.4",
    unit: "k/s",
    delta: "\u25B2 steady",
    deltaTone: "up"
  })), /*#__PURE__*/React.createElement("div", {
    className: "cs-cols"
  }, /*#__PURE__*/React.createElement(__ds_scope.Card, null, /*#__PURE__*/React.createElement(__ds_scope.Card.Header, {
    title: "Cluster topology",
    actions: /*#__PURE__*/React.createElement(__ds_scope.Badge, {
      tone: "alive",
      dot: true
    }, "quorum 4/5")
  }), /*#__PURE__*/React.createElement(__ds_scope.Card.Body, null, /*#__PURE__*/React.createElement(Topology, {
    sel: sel,
    setSel: setSel
  }))), /*#__PURE__*/React.createElement(__ds_scope.Card, null, /*#__PURE__*/React.createElement(__ds_scope.Card.Header, {
    title: "Selected peer",
    actions: /*#__PURE__*/React.createElement(__ds_scope.Badge, {
      tone: sel === 'n4' ? 'danger' : 'alive',
      dot: true
    }, sel === 'n4' ? 'down' : 'healthy')
  }), /*#__PURE__*/React.createElement(__ds_scope.Card.Body, null, (() => {
    const n = NODES.find(x => x.id === sel) || NODES[0];
    return /*#__PURE__*/React.createElement("div", {
      style: {
        fontFamily: 'var(--font-mono)',
        fontSize: 13,
        lineHeight: 2
      }
    }, /*#__PURE__*/React.createElement("div", {
      style: {
        color: 'var(--text-strong)',
        fontSize: 16,
        marginBottom: 8
      }
    }, n.id, " \xB7 ", n.region), /*#__PURE__*/React.createElement("div", {
      style: {
        color: 'var(--text-muted)'
      }
    }, "role ", /*#__PURE__*/React.createElement("span", {
      style: {
        color: n.role === 'leader' ? 'var(--accent)' : 'var(--text-body)'
      }
    }, n.role === 'down' ? 'unreachable' : n.role)), /*#__PURE__*/React.createElement("div", {
      style: {
        color: 'var(--text-muted)'
      }
    }, "latency ", /*#__PURE__*/React.createElement("span", {
      style: {
        color: 'var(--text-body)'
      }
    }, n.lat == null ? '—' : n.lat + 'ms')), /*#__PURE__*/React.createElement("div", {
      style: {
        color: 'var(--text-muted)'
      }
    }, "entries ", /*#__PURE__*/React.createElement("span", {
      style: {
        color: 'var(--text-body)'
      }
    }, n.role === 'down' ? '4180' : '4182')), /*#__PURE__*/React.createElement("div", {
      style: {
        marginTop: 16,
        display: 'flex',
        gap: 8
      }
    }, /*#__PURE__*/React.createElement(__ds_scope.Button, {
      size: "sm",
      variant: "secondary",
      onClick: onForce
    }, "Force election"), /*#__PURE__*/React.createElement(__ds_scope.Button, {
      size: "sm",
      variant: "danger"
    }, "Evict")));
  })()))));
}
function NodesTable() {
  return /*#__PURE__*/React.createElement(__ds_scope.Card, null, /*#__PURE__*/React.createElement("table", {
    className: "cs-table"
  }, /*#__PURE__*/React.createElement("thead", null, /*#__PURE__*/React.createElement("tr", null, /*#__PURE__*/React.createElement("th", null, "node"), /*#__PURE__*/React.createElement("th", null, "region"), /*#__PURE__*/React.createElement("th", null, "role"), /*#__PURE__*/React.createElement("th", null, "latency"), /*#__PURE__*/React.createElement("th", null, "entries"), /*#__PURE__*/React.createElement("th", null, "status"))), /*#__PURE__*/React.createElement("tbody", null, NODES.map(n => /*#__PURE__*/React.createElement("tr", {
    key: n.id
  }, /*#__PURE__*/React.createElement("td", {
    className: "cs-table__id"
  }, n.id), /*#__PURE__*/React.createElement("td", {
    style: {
      color: 'var(--text-muted)'
    }
  }, n.region), /*#__PURE__*/React.createElement("td", null, n.role === 'leader' ? /*#__PURE__*/React.createElement(__ds_scope.Badge, {
    tone: "ember"
  }, "leader") : n.role === 'down' ? /*#__PURE__*/React.createElement(__ds_scope.Badge, {
    tone: "neutral"
  }, "\u2014") : 'follower'), /*#__PURE__*/React.createElement("td", null, n.lat == null ? '—' : n.lat + 'ms'), /*#__PURE__*/React.createElement("td", null, n.role === 'down' ? '4180' : '4182'), /*#__PURE__*/React.createElement("td", null, n.role === 'down' ? /*#__PURE__*/React.createElement(__ds_scope.Badge, {
    tone: "danger",
    dot: true
  }, "down") : /*#__PURE__*/React.createElement(__ds_scope.Badge, {
    tone: "alive",
    dot: true
  }, "live")))))));
}
function LogView({
  log
}) {
  return /*#__PURE__*/React.createElement(__ds_scope.Card, {
    pad: true
  }, /*#__PURE__*/React.createElement("div", {
    className: "cs-log"
  }, log.map((e, i) => /*#__PURE__*/React.createElement("div", {
    className: "cs-log__row",
    key: i
  }, /*#__PURE__*/React.createElement("span", {
    className: "cs-log__ts"
  }, e.ts), /*#__PURE__*/React.createElement("span", {
    className: `cs-log__lvl cs-log__lvl--${e.lvl}`
  }, e.l), /*#__PURE__*/React.createElement("span", {
    className: "cs-log__msg"
  }, e.m)))));
}
function ConfigView() {
  return /*#__PURE__*/React.createElement(__ds_scope.Card, {
    pad: true
  }, /*#__PURE__*/React.createElement("div", {
    className: "cs-cfg"
  }, /*#__PURE__*/React.createElement("div", {
    className: "cs-cfg__group"
  }, /*#__PURE__*/React.createElement("span", {
    className: "cs-cfg__h"
  }, "// consensus"), /*#__PURE__*/React.createElement(__ds_scope.Select, {
    options: ['Raft', 'Paxos', 'PBFT'],
    defaultValue: "Raft"
  }), /*#__PURE__*/React.createElement(__ds_scope.Input, {
    label: "Quorum size",
    type: "number",
    defaultValue: 5,
    hint: "odd counts avoid split-brain"
  }), /*#__PURE__*/React.createElement(__ds_scope.Input, {
    label: "Heartbeat (ms)",
    type: "number",
    defaultValue: 200
  })), /*#__PURE__*/React.createElement("div", {
    className: "cs-cfg__group"
  }, /*#__PURE__*/React.createElement("span", {
    className: "cs-cfg__h"
  }, "// runtime flags"), /*#__PURE__*/React.createElement(__ds_scope.Switch, {
    label: "Deterministic replay",
    defaultChecked: true
  }), /*#__PURE__*/React.createElement(__ds_scope.Switch, {
    label: "Auto-gossip discovery",
    defaultChecked: true
  }), /*#__PURE__*/React.createElement(__ds_scope.Checkbox, {
    label: "Verify peer signatures",
    defaultChecked: true
  }), /*#__PURE__*/React.createElement(__ds_scope.Checkbox, {
    label: "Compress write-ahead log"
  })), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      gap: 10
    }
  }, /*#__PURE__*/React.createElement(__ds_scope.Button, {
    variant: "primary"
  }, "Apply & restart"), /*#__PURE__*/React.createElement(__ds_scope.Button, {
    variant: "ghost"
  }, "Reset"))));
}

/** The dhilipsiva Console — a distributed-cluster monitoring app (overview, nodes, log, config). */
function ConsoleApp() {
  const [view, setView] = React.useState('overview');
  const [sel, setSel] = React.useState('n0');
  const [log, setLog] = React.useState(BASE_LOG);
  const title = NAV.find(n => n.id === view).label;
  const forceElection = () => {
    const ts = new Date().toTimeString().slice(0, 8) + '.' + String(Math.floor(Math.random() * 900) + 100);
    setLog(l => [{
      ts,
      lvl: 'ok',
      l: 'elect',
      m: `manual election triggered · ${sel} → candidate · term 8`
    }, ...l]);
    setView('log');
  };
  return /*#__PURE__*/React.createElement("div", {
    className: "cs"
  }, /*#__PURE__*/React.createElement("aside", {
    className: "cs-side"
  }, /*#__PURE__*/React.createElement("div", {
    className: "cs-side__brand"
  }, /*#__PURE__*/React.createElement("img", {
    src: "../../assets/mark.svg",
    alt: ""
  }), "dhilipsiva"), /*#__PURE__*/React.createElement("div", {
    className: "cs-side__sec"
  }, "cluster \xB7 prod-1"), NAV.map(n => /*#__PURE__*/React.createElement("div", {
    key: n.id,
    className: ['cs-nav', view === n.id ? 'cs-nav--on' : ''].filter(Boolean).join(' '),
    onClick: () => setView(n.id)
  }, /*#__PURE__*/React.createElement("span", {
    className: "cs-nav__glyph"
  }, n.g), n.label, n.count && /*#__PURE__*/React.createElement("span", {
    className: "cs-nav__count"
  }, n.count))), /*#__PURE__*/React.createElement("div", {
    className: "cs-side__sec"
  }, "peers"), /*#__PURE__*/React.createElement("div", {
    style: {
      padding: '0 4px'
    }
  }, /*#__PURE__*/React.createElement(__ds_scope.Spinner, {
    tone: "alive",
    label: "gossiping"
  })), /*#__PURE__*/React.createElement("div", {
    className: "cs-side__foot"
  }, /*#__PURE__*/React.createElement(__ds_scope.Avatar, {
    name: "Ada Lovelace",
    size: "sm",
    status: "online"
  }), " ada@dhilipsiva.dev")), /*#__PURE__*/React.createElement("main", {
    className: "cs-main"
  }, /*#__PURE__*/React.createElement("header", {
    className: "cs-top"
  }, /*#__PURE__*/React.createElement("span", {
    className: "cs-top__title"
  }, title), /*#__PURE__*/React.createElement(__ds_scope.Badge, {
    tone: "alive",
    dot: true
  }, "5-node \xB7 raft \xB7 term 7"), /*#__PURE__*/React.createElement("span", {
    className: "cs-top__sp"
  }), /*#__PURE__*/React.createElement("div", {
    className: "cs-search"
  }, /*#__PURE__*/React.createElement(__ds_scope.Input, {
    lead: "\u2315",
    placeholder: "filter peers, entries\u2026"
  })), /*#__PURE__*/React.createElement(__ds_scope.IconButton, {
    label: "Docs"
  }, "?"), /*#__PURE__*/React.createElement(__ds_scope.Button, {
    size: "sm",
    variant: "primary",
    onClick: forceElection
  }, "Force election")), /*#__PURE__*/React.createElement("div", {
    className: "cs-body"
  }, view === 'overview' && /*#__PURE__*/React.createElement(Overview, {
    sel: sel,
    setSel: setSel,
    onForce: forceElection
  }), view === 'nodes' && /*#__PURE__*/React.createElement(NodesTable, null), view === 'log' && /*#__PURE__*/React.createElement(LogView, {
    log: log
  }), view === 'config' && /*#__PURE__*/React.createElement(ConfigView, null))));
}
Object.assign(__ds_scope, { ConsoleApp });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/console/ConsoleApp.jsx", error: String((e && e.message) || e) }); }

// ui_kits/landing/LandingPage.jsx
try { (() => {
__ds_scope.injectStyle('q-landing', `
.lp { background: var(--bg-base); color: var(--text-body); min-height: 100%; }
.lp__container { max-width: 1120px; margin: 0 auto; padding: 0 28px; }

/* nav */
.lp-nav { position: sticky; top: 0; z-index: 30; display: flex; align-items: center; gap: 24px; height: 64px; border-bottom: 1px solid var(--border-subtle); background: color-mix(in oklab, var(--bg-base) 82%, transparent); backdrop-filter: var(--blur-glass); }
.lp-nav__brand { display: flex; align-items: center; gap: 10px; font-family: var(--font-display); font-weight: 600; font-size: 19px; letter-spacing: -0.03em; color: var(--text-strong); }
.lp-nav__brand img { width: 28px; height: 28px; }
.lp-nav__links { display: flex; gap: 4px; margin-left: 8px; }
.lp-nav__link { font-family: var(--font-mono); font-size: 13px; color: var(--text-muted); padding: 6px 10px; border-radius: var(--radius-sm); }
.lp-nav__link:hover { color: var(--text-strong); background: var(--surface-hover); }
.lp-nav__spacer { flex: 1; }
.lp-nav__stars { display: flex; align-items: center; gap: 7px; font-family: var(--font-mono); font-size: 13px; color: var(--text-muted); padding: 6px 11px; border: 1px solid var(--border-subtle); border-radius: var(--radius-md); }
.lp-nav__stars b { color: var(--text-strong); }

/* hero */
.lp-hero { position: relative; padding: 84px 0 64px; overflow: hidden; }
.lp-hero__grid { position: absolute; inset: 0; background-image: var(--bg-grid); opacity: 0.5; mask-image: radial-gradient(ellipse 80% 70% at 30% 0%, #000 30%, transparent 75%); }
.lp-hero__inner { position: relative; display: grid; grid-template-columns: 1.05fr 0.95fr; gap: 48px; align-items: center; }
.lp-eyebrow { display: inline-flex; align-items: center; gap: 10px; font-family: var(--font-mono); font-size: 12px; letter-spacing: 0.04em; color: var(--text-muted); margin-bottom: 22px; }
.lp-h1 { font-family: var(--font-display); font-weight: 600; font-size: 60px; line-height: 1.02; letter-spacing: -0.035em; color: var(--text-strong); margin: 0 0 20px; }
.lp-h1 em { font-style: normal; color: var(--accent); }
.lp-lede { font-family: var(--font-sans); font-size: 18px; line-height: 1.55; color: var(--text-muted); max-width: 42ch; margin: 0 0 28px; }
.lp-actions { display: flex; gap: 12px; align-items: center; margin-bottom: 24px; }
.lp-install { display: flex; align-items: center; gap: 12px; height: 44px; padding: 0 6px 0 16px; background: var(--surface-inset); border: 1px solid var(--border-subtle); border-radius: var(--radius-md); font-family: var(--font-mono); font-size: 14px; color: var(--text-strong); max-width: 340px; }
.lp-install__prompt { color: var(--accent); }
.lp-install__copy { margin-left: auto; }

/* terminal window */
.term { background: var(--void-1000); border: 1px solid var(--border-strong); border-radius: var(--radius-lg); box-shadow: var(--shadow-3); overflow: hidden; }
.term__bar { display: flex; align-items: center; gap: 8px; padding: 11px 14px; border-bottom: 1px solid var(--border-subtle); }
.term__dot { width: 11px; height: 11px; border-radius: 50%; }
.term__title { margin-left: 8px; font-family: var(--font-mono); font-size: 12px; color: var(--text-faint); }
.term__body { padding: 18px 20px; font-family: var(--font-mono); font-size: 13.5px; line-height: 1.75; white-space: pre; overflow-x: auto; }
.c-key { color: var(--ember-300); } .c-fn { color: var(--sky-500); } .c-str { color: var(--phosphor-300); } .c-num { color: var(--phosphor-300); } .c-com { color: var(--text-faint); } .c-type { color: var(--quanta-300); } .c-pun { color: var(--text-muted); }

/* stat strip */
.lp-stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 1px; background: var(--border-subtle); border-top: 1px solid var(--border-subtle); border-bottom: 1px solid var(--border-subtle); }
.lp-stat { background: var(--bg-base); padding: 28px 24px; }
.lp-stat__num { font-family: var(--font-display); font-weight: 600; font-size: 38px; color: var(--text-strong); letter-spacing: -0.03em; }
.lp-stat__num span { color: var(--accent); }
.lp-stat__lab { font-family: var(--font-mono); font-size: 12px; color: var(--text-muted); margin-top: 4px; letter-spacing: 0.02em; }

/* section */
.lp-section { padding: 80px 0; }
.lp-sec-head { margin-bottom: 40px; max-width: 56ch; }
.lp-sec-eyebrow { font-family: var(--font-mono); font-size: 12px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--accent); }
.lp-sec-title { font-family: var(--font-display); font-weight: 600; font-size: 36px; letter-spacing: -0.03em; color: var(--text-strong); margin: 12px 0 0; }
.lp-features { display: grid; grid-template-columns: repeat(2, 1fr); gap: 16px; }
.lp-feat__glyph { font-family: var(--font-mono); font-size: 26px; color: var(--accent); margin-bottom: 14px; display: block; }
.lp-feat__title { font-family: var(--font-display); font-weight: 600; font-size: 19px; color: var(--text-strong); margin: 0 0 8px; }
.lp-feat__body { font-family: var(--font-sans); font-size: 14.5px; line-height: 1.6; color: var(--text-muted); margin: 0; }

/* code section */
.lp-code { display: grid; grid-template-columns: 0.85fr 1.15fr; gap: 40px; align-items: center; }

/* cta + footer */
.lp-cta { text-align: center; padding: 96px 0; position: relative; }
.lp-cta__grid { position: absolute; inset: 0; background-image: var(--bg-grid); opacity: 0.35; mask-image: radial-gradient(ellipse 60% 80% at 50% 50%, #000, transparent 70%); }
.lp-cta__h { position: relative; font-family: var(--font-serif); font-size: 40px; font-style: italic; color: var(--text-strong); margin: 0 0 8px; }
.lp-cta__sub { position: relative; font-family: var(--font-mono); font-size: 14px; color: var(--text-muted); margin: 0 0 28px; }
.lp-footer { border-top: 1px solid var(--border-subtle); padding: 40px 0; display: flex; align-items: center; gap: 16px; font-family: var(--font-mono); font-size: 12.5px; color: var(--text-faint); }
.lp-footer__sp { flex: 1; }
.lp-footer a { color: var(--text-muted); }

@media (max-width: 900px) {
  .lp-hero__inner, .lp-code { grid-template-columns: 1fr; }
  .lp-h1 { font-size: 44px; }
  .lp-stats { grid-template-columns: repeat(2, 1fr); }
  .lp-features { grid-template-columns: 1fr; }
  .lp-nav__links { display: none; }
}
`);
function Term({
  title,
  children
}) {
  return /*#__PURE__*/React.createElement("div", {
    className: "term"
  }, /*#__PURE__*/React.createElement("div", {
    className: "term__bar"
  }, /*#__PURE__*/React.createElement("span", {
    className: "term__dot",
    style: {
      background: 'var(--crimson-500)'
    }
  }), /*#__PURE__*/React.createElement("span", {
    className: "term__dot",
    style: {
      background: 'var(--amber-500)'
    }
  }), /*#__PURE__*/React.createElement("span", {
    className: "term__dot",
    style: {
      background: 'var(--phosphor-500)'
    }
  }), /*#__PURE__*/React.createElement("span", {
    className: "term__title"
  }, title)), /*#__PURE__*/React.createElement("div", {
    className: "term__body"
  }, children));
}
const SAMPLES = {
  rust: /*#__PURE__*/React.createElement("code", {
    dangerouslySetInnerHTML: {
      __html: `<span class="c-com">// a node is a fixed point of the network</span>
<span class="c-key">use</span> fixpoint<span class="c-pun">::{</span>Node<span class="c-pun">,</span> Quorum<span class="c-pun">};</span>

<span class="c-pun">#[</span><span class="c-fn">fixpoint::main</span><span class="c-pun">]</span>
<span class="c-key">async fn</span> <span class="c-fn">main</span><span class="c-pun">()</span> -> <span class="c-type">Result</span><span class="c-pun"><()></span> <span class="c-pun">{</span>
    <span class="c-key">let</span> node = <span class="c-type">Node</span><span class="c-pun">::</span><span class="c-fn">spawn</span><span class="c-pun">(</span><span class="c-str">"fixpoint/core"</span><span class="c-pun">).</span><span class="c-fn">await</span><span class="c-pun">?;</span>
    node.<span class="c-fn">join</span><span class="c-pun">(</span><span class="c-type">Quorum</span><span class="c-pun">::</span><span class="c-fn">of</span><span class="c-pun">(</span><span class="c-num">5</span><span class="c-pun">)).</span><span class="c-fn">await</span><span class="c-pun">?;</span>
    node.<span class="c-fn">commit</span><span class="c-pun">(</span><span class="c-str">"∀ peers: agree"</span><span class="c-pun">).</span><span class="c-fn">await</span>
<span class="c-pun">}</span>`
    }
  }),
  python: /*#__PURE__*/React.createElement("code", {
    dangerouslySetInnerHTML: {
      __html: `<span class="c-key">from</span> fixpoint <span class="c-key">import</span> Node<span class="c-pun">,</span> Quorum

<span class="c-key">async def</span> <span class="c-fn">main</span><span class="c-pun">():</span>
    node = <span class="c-key">await</span> <span class="c-type">Node</span><span class="c-pun">.</span><span class="c-fn">spawn</span><span class="c-pun">(</span><span class="c-str">"fixpoint/core"</span><span class="c-pun">)</span>
    <span class="c-key">await</span> node<span class="c-pun">.</span><span class="c-fn">join</span><span class="c-pun">(</span><span class="c-type">Quorum</span><span class="c-pun">.</span><span class="c-fn">of</span><span class="c-pun">(</span><span class="c-num">5</span><span class="c-pun">))</span>
    <span class="c-key">await</span> node<span class="c-pun">.</span><span class="c-fn">commit</span><span class="c-pun">(</span><span class="c-str">"∀ peers: agree"</span><span class="c-pun">)</span>`
    }
  }),
  wasm: /*#__PURE__*/React.createElement("code", {
    dangerouslySetInnerHTML: {
      __html: `<span class="c-com">;; 12kB portable core — runs in any browser</span>
<span class="c-pun">(</span><span class="c-key">module</span> <span class="c-str">$fixpoint</span>
  <span class="c-pun">(</span><span class="c-key">import</span> <span class="c-str">"net"</span> <span class="c-str">"gossip"</span> <span class="c-pun">(</span><span class="c-key">func</span> <span class="c-str">$gossip</span><span class="c-pun">))</span>
  <span class="c-pun">(</span><span class="c-key">func</span> <span class="c-pun">(</span><span class="c-key">export</span> <span class="c-str">"commit"</span><span class="c-pun">)</span> <span class="c-pun">(</span><span class="c-key">param</span> <span class="c-type">i32</span><span class="c-pun">)</span>
    <span class="c-fn">local.get</span> <span class="c-num">0</span>
    <span class="c-fn">call</span> <span class="c-str">$gossip</span><span class="c-pun">))</span>`
    }
  })
};
function CopyInstall() {
  const [copied, setCopied] = React.useState(false);
  const cmd = 'cargo add fixpoint';
  const copy = () => {
    try {
      navigator.clipboard.writeText(cmd);
    } catch (e) {}
    setCopied(true);
    setTimeout(() => setCopied(false), 1400);
  };
  return /*#__PURE__*/React.createElement("div", {
    className: "lp-install"
  }, /*#__PURE__*/React.createElement("span", {
    className: "lp-install__prompt"
  }, "$"), /*#__PURE__*/React.createElement("span", null, cmd), /*#__PURE__*/React.createElement("span", {
    className: "lp-install__copy"
  }, /*#__PURE__*/React.createElement(__ds_scope.Button, {
    size: "sm",
    variant: "ghost",
    onClick: copy
  }, copied ? '✓ copied' : 'copy')));
}
const FEATURES = [{
  g: '⊢',
  t: 'Deterministic by default',
  b: 'Same inputs, same history, every node. Replay any partition from the write-ahead log — no heisenbugs, no surprises.'
}, {
  g: '⇄',
  t: 'Peer-to-peer over WebRTC',
  b: 'Nodes find each other and gossip directly. No broker, no single point of failure, no rent.'
}, {
  g: 'λ',
  t: 'Symbolic configuration',
  b: 'Express invariants as logic, not YAML. The planner proves your topology before it ever runs.'
}, {
  g: '◫',
  t: '12kB Wasm core',
  b: 'The whole runtime compiles to portable WebAssembly. Ship it to a server, a browser, or a toaster.'
}];

/** The dhilipsiva open-source project landing page (the fixpoint runtime) — marketing surface, brand voice. */
function LandingPage() {
  const [tab, setTab] = React.useState('rust');
  return /*#__PURE__*/React.createElement("div", {
    className: "lp"
  }, /*#__PURE__*/React.createElement("nav", {
    className: "lp-nav"
  }, /*#__PURE__*/React.createElement("div", {
    className: "lp__container",
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 18,
      width: '100%'
    }
  }, /*#__PURE__*/React.createElement("div", {
    className: "lp-nav__brand"
  }, /*#__PURE__*/React.createElement("img", {
    src: "../../assets/mark.svg",
    alt: ""
  }), "dhilipsiva"), /*#__PURE__*/React.createElement("div", {
    className: "lp-nav__links"
  }, /*#__PURE__*/React.createElement("a", {
    className: "lp-nav__link",
    href: "#"
  }, "Docs"), /*#__PURE__*/React.createElement("a", {
    className: "lp-nav__link",
    href: "#"
  }, "Spec"), /*#__PURE__*/React.createElement("a", {
    className: "lp-nav__link",
    href: "#"
  }, "Blog"), /*#__PURE__*/React.createElement("a", {
    className: "lp-nav__link",
    href: "#"
  }, "Community")), /*#__PURE__*/React.createElement("span", {
    className: "lp-nav__spacer"
  }), /*#__PURE__*/React.createElement("span", {
    className: "lp-nav__stars"
  }, "\u2605 ", /*#__PURE__*/React.createElement("b", null, "14.2k")), /*#__PURE__*/React.createElement(__ds_scope.Button, {
    size: "sm",
    variant: "primary"
  }, "Get started"))), /*#__PURE__*/React.createElement("header", {
    className: "lp-hero"
  }, /*#__PURE__*/React.createElement("div", {
    className: "lp-hero__grid"
  }), /*#__PURE__*/React.createElement("div", {
    className: "lp__container lp-hero__inner"
  }, /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    className: "lp-eyebrow"
  }, /*#__PURE__*/React.createElement(__ds_scope.Badge, {
    tone: "ember"
  }, "v0.4.2"), " MIT licensed \xB7 \u2200 platforms"), /*#__PURE__*/React.createElement("h1", {
    className: "lp-h1"
  }, "Build systems that", /*#__PURE__*/React.createElement("br", null), "outlive their ", /*#__PURE__*/React.createElement("em", null, "meaning.")), /*#__PURE__*/React.createElement("p", {
    className: "lp-lede"
  }, "A deterministic, peer-to-peer runtime for distributed programs. Nothing is owed to your nodes \u2014 so ", /*#__PURE__*/React.createElement("em", {
    style: {
      color: 'var(--text-strong)',
      fontStyle: 'normal'
    }
  }, "fixpoint"), " makes them agree anyway."), /*#__PURE__*/React.createElement("div", {
    className: "lp-actions"
  }, /*#__PURE__*/React.createElement(__ds_scope.Button, {
    variant: "primary",
    size: "lg"
  }, "Read the docs"), /*#__PURE__*/React.createElement(__ds_scope.Button, {
    variant: "secondary",
    size: "lg"
  }, "\u2605 Star on GitHub")), /*#__PURE__*/React.createElement(CopyInstall, null)), /*#__PURE__*/React.createElement(Term, {
    title: "~/fixpoint/examples/quorum.rs"
  }, SAMPLES.rust))), /*#__PURE__*/React.createElement("section", {
    className: "lp-stats"
  }, /*#__PURE__*/React.createElement("div", {
    className: "lp-stat"
  }, /*#__PURE__*/React.createElement("div", {
    className: "lp-stat__num"
  }, /*#__PURE__*/React.createElement("span", null, "38"), "ms"), /*#__PURE__*/React.createElement("div", {
    className: "lp-stat__lab"
  }, "// 5-node commit latency")), /*#__PURE__*/React.createElement("div", {
    className: "lp-stat"
  }, /*#__PURE__*/React.createElement("div", {
    className: "lp-stat__num"
  }, "0"), /*#__PURE__*/React.createElement("div", {
    className: "lp-stat__lab"
  }, "// GC pauses, by design")), /*#__PURE__*/React.createElement("div", {
    className: "lp-stat"
  }, /*#__PURE__*/React.createElement("div", {
    className: "lp-stat__num"
  }, /*#__PURE__*/React.createElement("span", null, "12"), "kB"), /*#__PURE__*/React.createElement("div", {
    className: "lp-stat__lab"
  }, "// portable wasm core")), /*#__PURE__*/React.createElement("div", {
    className: "lp-stat"
  }, /*#__PURE__*/React.createElement("div", {
    className: "lp-stat__num"
  }, "\u221E"), /*#__PURE__*/React.createElement("div", {
    className: "lp-stat__lab"
  }, "// deterministic replays"))), /*#__PURE__*/React.createElement("section", {
    className: "lp-section"
  }, /*#__PURE__*/React.createElement("div", {
    className: "lp__container"
  }, /*#__PURE__*/React.createElement("div", {
    className: "lp-sec-head"
  }, /*#__PURE__*/React.createElement("span", {
    className: "lp-sec-eyebrow"
  }, "// why it exists"), /*#__PURE__*/React.createElement("h2", {
    className: "lp-sec-title"
  }, "Consensus is a fixed point. We just compute it faster.")), /*#__PURE__*/React.createElement("div", {
    className: "lp-features"
  }, FEATURES.map(f => /*#__PURE__*/React.createElement(__ds_scope.Card, {
    key: f.t,
    pad: true,
    interactive: true
  }, /*#__PURE__*/React.createElement("span", {
    className: "lp-feat__glyph"
  }, f.g), /*#__PURE__*/React.createElement("h3", {
    className: "lp-feat__title"
  }, f.t), /*#__PURE__*/React.createElement("p", {
    className: "lp-feat__body"
  }, f.b)))))), /*#__PURE__*/React.createElement("section", {
    className: "lp-section",
    style: {
      paddingTop: 0
    }
  }, /*#__PURE__*/React.createElement("div", {
    className: "lp__container lp-code"
  }, /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    className: "lp-sec-eyebrow"
  }, "// one API, three runtimes"), /*#__PURE__*/React.createElement("h2", {
    className: "lp-sec-title"
  }, "Write it once.", /*#__PURE__*/React.createElement("br", null), "Run it everywhere it shouldn't."), /*#__PURE__*/React.createElement("p", {
    className: "lp-feat__body",
    style: {
      marginTop: 16,
      maxWidth: '36ch'
    }
  }, "The same node logic compiles to native Rust, drives a Python service, or ships as a Wasm module to the browser. The network doesn't care where you run.")), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement(__ds_scope.Tabs, {
    tabs: [{
      id: 'rust',
      label: 'Rust'
    }, {
      id: 'python',
      label: 'Python'
    }, {
      id: 'wasm',
      label: 'Wasm'
    }],
    value: tab,
    onChange: setTab
  }), /*#__PURE__*/React.createElement("div", {
    style: {
      height: 14
    }
  }), /*#__PURE__*/React.createElement(Term, {
    title: tab === 'wasm' ? 'core.wat' : tab === 'python' ? 'node.py' : 'main.rs'
  }, SAMPLES[tab])))), /*#__PURE__*/React.createElement("section", {
    className: "lp-cta"
  }, /*#__PURE__*/React.createElement("div", {
    className: "lp-cta__grid"
  }), /*#__PURE__*/React.createElement("h2", {
    className: "lp-cta__h"
  }, "\u201COne must imagine the cluster happy.\u201D"), /*#__PURE__*/React.createElement("p", {
    className: "lp-cta__sub"
  }, "// free forever \xB7 self-hostable \xB7 no telemetry"), /*#__PURE__*/React.createElement("div", {
    style: {
      display: 'flex',
      gap: 12,
      justifyContent: 'center',
      position: 'relative'
    }
  }, /*#__PURE__*/React.createElement(__ds_scope.Button, {
    variant: "primary",
    size: "lg"
  }, "Start building"), /*#__PURE__*/React.createElement(__ds_scope.Button, {
    variant: "ghost",
    size: "lg"
  }, "Read the spec \u2192"))), /*#__PURE__*/React.createElement("footer", {
    className: "lp-footer"
  }, /*#__PURE__*/React.createElement("div", {
    className: "lp__container",
    style: {
      display: 'flex',
      alignItems: 'center',
      gap: 16,
      width: '100%'
    }
  }, /*#__PURE__*/React.createElement("img", {
    src: "../../assets/mark.svg",
    alt: "",
    style: {
      width: 22,
      height: 22
    }
  }), /*#__PURE__*/React.createElement("span", null, "dhilipsiva \xB7 ", new Date().getFullYear(), " \xB7 the void, organized"), /*#__PURE__*/React.createElement("span", {
    className: "lp-footer__sp"
  }), /*#__PURE__*/React.createElement("a", {
    href: "#"
  }, "GitHub"), /*#__PURE__*/React.createElement("a", {
    href: "#"
  }, "Discord"), /*#__PURE__*/React.createElement("a", {
    href: "#"
  }, "RSS"), /*#__PURE__*/React.createElement("a", {
    href: "#"
  }, "MIT"))));
}
Object.assign(__ds_scope, { LandingPage });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/landing/LandingPage.jsx", error: String((e && e.message) || e) }); }

// ui_kits/nibli/NibliApp.jsx
try { (() => {
__ds_scope.injectStyle('q-nibli', `
.nb { background: var(--bg-base); color: var(--text-body); min-height: 100%; font-family: var(--font-sans); }
.nb__wrap { max-width: 1000px; margin: 0 auto; padding: 0 28px; }

/* top bar */
.nb-top { position: sticky; top: 0; z-index: 30; display: flex; align-items: center; height: 60px; border-bottom: 1px solid var(--border-subtle); background: color-mix(in oklab, var(--bg-base) 84%, transparent); backdrop-filter: var(--blur-glass); }
.nb-top__row { display: flex; align-items: center; gap: 16px; width: 100%; }
.nb-brand { display: flex; align-items: center; gap: 9px; font-family: var(--font-display); font-weight: 600; font-size: 18px; letter-spacing: -0.03em; color: var(--text-strong); }
.nb-brand img { width: 24px; height: 24px; }
.nb-brand small { font-family: var(--font-mono); font-size: 10px; letter-spacing: .12em; color: var(--text-faint); text-transform: uppercase; align-self: center; margin-left: 2px; }
.nb-top__sp { flex: 1; }
.nb-tavla { display: flex; align-items: center; gap: 8px; font-family: var(--font-mono); font-size: 12px; color: var(--text-muted); padding: 6px 11px; border: 1px solid var(--border-subtle); border-radius: var(--radius-md); }
.nb-tavla__dot { width: 7px; height: 7px; border-radius: 50%; background: var(--alive); box-shadow: var(--glow-phosphor); }

/* composer */
.nb-main { padding: 32px 0 64px; }
.nb-eyebrow { font-family: var(--font-mono); font-size: 12px; letter-spacing: .04em; color: var(--text-muted); }
.nb-eyebrow b { color: var(--symbol); }
.nb-pills { display: flex; gap: 8px; margin: 14px 0 16px; flex-wrap: wrap; }
.nb-pill { font-family: var(--font-mono); font-size: 12.5px; color: var(--text-muted); padding: 6px 12px; border: 1px solid var(--border-subtle); border-radius: var(--radius-full); cursor: pointer; transition: all var(--dur-fast) var(--ease-out); }
.nb-pill:hover { color: var(--text-strong); border-color: var(--border-strong); }
.nb-pill--on { color: var(--text-on-ember); background: var(--accent); border-color: transparent; }
.nb-composer { display: flex; gap: 10px; align-items: stretch; }
.nb-input { flex: 1; display: flex; align-items: center; gap: 10px; height: 48px; padding: 0 16px; background: var(--surface-inset); border: 1px solid var(--border-subtle); border-radius: var(--radius-md); font-family: var(--font-mono); font-size: 15px; color: var(--text-strong); }
.nb-input__q { color: var(--accent); }

/* pipeline */
.nb-pipe { margin-top: 30px; }
.nb-triad-label { display: flex; align-items: center; gap: 10px; margin: 4px 0 4px; font-family: var(--font-mono); font-size: 11px; letter-spacing: .08em; text-transform: uppercase; color: var(--symbol); }
.nb-triad-label::after { content: ""; flex: 1; height: 1px; background: linear-gradient(90deg, color-mix(in oklab, var(--symbol) 50%, transparent), transparent); }

.nb-stage { background: var(--surface-card); border: 1px solid var(--border-subtle); border-radius: var(--radius-lg); overflow: clip; }
.nb-stage--triad { border-color: color-mix(in oklab, var(--symbol) 26%, var(--border-subtle)); }
.nb-stage__head { display: flex; align-items: center; gap: 12px; padding: 13px 18px; border-bottom: 1px solid var(--border-subtle); }
.nb-stage__num { font-family: var(--font-mono); font-size: 11px; color: var(--text-on-ember); background: var(--text-faint); width: 22px; height: 22px; border-radius: var(--radius-sm); display: grid; place-items: center; flex: none; }
.nb-stage--triad .nb-stage__num { background: var(--symbol); }
.nb-stage__title { font-family: var(--font-display); font-weight: 600; font-size: 15px; color: var(--text-strong); }
.nb-stage__sub { font-family: var(--font-mono); font-size: 11px; color: var(--text-faint); margin-left: auto; }
.nb-stage__body { padding: 18px; }

.nb-arrow { display: flex; align-items: center; gap: 12px; padding: 8px 0 8px 30px; }
.nb-arrow__g { color: var(--text-faint); font-family: var(--font-mono); font-size: 18px; line-height: 1; }
.nb-arrow__v { font-family: var(--font-mono); font-size: 11.5px; color: var(--text-faint); letter-spacing: .04em; }
.nb-arrow__v b { color: var(--symbol); }

/* source / nl rows */
.nb-nl { font-family: var(--font-sans); font-size: 15px; line-height: 1.65; color: var(--text-body); }
.nb-nl--strong { color: var(--text-strong); }
.nb-prem { display: flex; gap: 10px; align-items: baseline; padding: 3px 0; }
.nb-prem__tag { font-family: var(--font-mono); font-size: 11px; color: var(--text-faint); flex: none; width: 26px; }

/* lojban */
.nb-lojban { font-family: var(--font-mono); font-size: 15px; line-height: 1.9; color: var(--quanta-300); }
.nb-lojban .q { color: var(--accent); }
.nb-fol { margin-top: 14px; padding-top: 14px; border-top: 1px dashed var(--border-subtle); font-family: var(--font-mono); font-size: 13px; line-height: 1.8; color: var(--text-muted); }
.nb-fol__lab { font-size: 10.5px; letter-spacing: .1em; text-transform: uppercase; color: var(--text-faint); margin-bottom: 6px; }

/* back-translation confirm */
.nb-confirm { display: flex; align-items: center; gap: 12px; margin-top: 16px; padding: 12px 14px; border-radius: var(--radius-md); background: color-mix(in oklab, var(--phosphor-500) 8%, var(--surface-inset)); border: 1px solid color-mix(in oklab, var(--phosphor-500) 24%, transparent); }
.nb-confirm__txt { font-family: var(--font-mono); font-size: 13px; color: var(--phosphor-300); flex: 1; }
.nb-confirm--off { background: var(--surface-inset); border-color: var(--border-subtle); }
.nb-confirm--off .nb-confirm__txt { color: var(--text-muted); }

/* proof tree */
.nb-proof { background-image: var(--bg-grid); background-color: var(--void-1000); border-radius: var(--radius-md); padding: 18px 16px; font-family: var(--font-mono); }
.pt { margin-left: 0; }
.pt-node { position: relative; }
.pt-children { margin-left: 18px; border-left: 1px solid var(--border-subtle); padding-left: 16px; }
.pt-line { display: flex; align-items: center; gap: 10px; padding: 5px 0; flex-wrap: wrap; }
.pt-formula { font-size: 13.5px; color: var(--text-body); }
.pt-formula--goal { color: var(--text-strong); font-weight: 600; }
.pt-formula--open { color: var(--amber-500); }
.pt-rule { font-size: 10px; letter-spacing: .06em; text-transform: uppercase; padding: 2px 7px; border-radius: var(--radius-sm); }
.pt-rule--derived { color: var(--phosphor-300); background: color-mix(in oklab, var(--phosphor-500) 14%, transparent); }
.pt-rule--premise { color: var(--text-muted); background: var(--surface-inset); }
.pt-rule--open { color: var(--amber-500); background: color-mix(in oklab, var(--amber-500) 14%, transparent); }
.pt-prov { font-size: 11px; color: var(--text-faint); display: inline-flex; gap: 8px; align-items: center; }
.pt-prov b { color: var(--sky-500); font-weight: 500; }
.pt-note { font-size: 11.5px; color: var(--text-faint); font-style: italic; }

/* verdict */
.nb-verdict { display: flex; align-items: center; gap: 16px; margin-top: 16px; padding: 16px 18px; border-radius: var(--radius-md); border: 1px solid var(--border-subtle); }
.nb-verdict--ok { border-color: color-mix(in oklab, var(--phosphor-500) 36%, transparent); background: color-mix(in oklab, var(--phosphor-500) 8%, transparent); }
.nb-verdict--open { border-color: color-mix(in oklab, var(--amber-500) 34%, transparent); background: color-mix(in oklab, var(--amber-500) 8%, transparent); }
.nb-verdict__glyph { font-family: var(--font-mono); font-size: 30px; line-height: 1; }
.nb-verdict__main { font-family: var(--font-display); font-weight: 600; font-size: 20px; letter-spacing: -0.01em; color: var(--text-strong); }
.nb-verdict__sub { font-family: var(--font-mono); font-size: 12.5px; color: var(--text-muted); margin-top: 3px; }

.nb-foot { margin-top: 28px; padding: 16px 18px; border: 1px dashed var(--border-subtle); border-radius: var(--radius-md); font-family: var(--font-mono); font-size: 12.5px; line-height: 1.7; color: var(--text-muted); }
.nb-foot b { color: var(--text-strong); }

@media (max-width: 720px) { .nb-composer { flex-direction: column; } }
`);
const QUERIES = {
  socrates: {
    label: 'Socrates · mortal',
    query: 'Is Socrates mortal?',
    premises: ['Every human is mortal.', 'Socrates is a human.'],
    lojban: {
      lines: ['ro remna cu morsi', 'la .sokrates. cu remna'],
      query: 'xu la .sokrates. cu morsi'
    },
    fol: ['∀x. remna(x) → morsi(x)', 'remna(sokrates)', '⊢ ?  morsi(sokrates)'],
    back: ['For every x, if x is human then x is mortal.', 'Socrates is human.', 'Asking: is Socrates mortal?'],
    verdict: 'ok',
    timing: 'derived in 3.8ms · 6 nodes · sound',
    proof: {
      formula: 'morsi(sokrates)',
      rule: '→-elim',
      status: 'goal',
      children: [{
        formula: 'remna(sokrates) → morsi(sokrates)',
        rule: '∀-elim',
        status: 'derived',
        children: [{
          formula: '∀x. remna(x) → morsi(x)',
          rule: 'premise',
          tag: 'P1',
          peer: 'tavla:7f3a',
          evid: "ju'a · asserted",
          status: 'fact'
        }]
      }, {
        formula: 'remna(sokrates)',
        rule: 'premise',
        tag: 'P2',
        peer: 'tavla:9c1d',
        evid: "ti'e · reported",
        status: 'fact'
      }]
    },
    result: {
      glyph: '⊤',
      color: 'var(--alive)',
      main: 'ENTAILED',
      sub: '∴ morsi(sokrates) — the conclusion follows from P1, P2 by ∀-elim then →-elim.'
    }
  },
  plato: {
    label: 'Plato · mortal',
    query: 'Is Plato mortal?',
    premises: ['Every human is mortal.', 'Socrates is a human.'],
    lojban: {
      lines: ['ro remna cu morsi', 'la .sokrates. cu remna'],
      query: 'xu la .platon. cu morsi'
    },
    fol: ['∀x. remna(x) → morsi(x)', 'remna(sokrates)', '⊢ ?  morsi(platon)'],
    back: ['For every x, if x is human then x is mortal.', 'Socrates is human.', 'Asking: is Plato mortal?'],
    verdict: 'open',
    timing: 'halted in 1.2ms · open subgoal · sound',
    proof: {
      formula: 'morsi(platon)',
      rule: '→-elim?',
      status: 'goal',
      children: [{
        formula: 'remna(platon) → morsi(platon)',
        rule: '∀-elim',
        status: 'derived',
        children: [{
          formula: '∀x. remna(x) → morsi(x)',
          rule: 'premise',
          tag: 'P1',
          peer: 'tavla:7f3a',
          evid: "ju'a · asserted",
          status: 'fact'
        }]
      }, {
        formula: 'remna(platon)',
        rule: 'open',
        status: 'open',
        note: 'no fact, no rule — nothing entails remna(platon)'
      }]
    },
    result: {
      glyph: '⊥',
      color: 'var(--amber-500)',
      main: 'NOT ENTAILED',
      sub: 'remna(platon) is unknown, so morsi(platon) cannot be derived. nibli will not assert what its rules do not license.'
    }
  }
};
function ProofNode({
  node,
  depth = 0
}) {
  const ruleCls = node.status === 'open' ? 'pt-rule--open' : node.rule === 'premise' || node.status === 'fact' ? 'pt-rule--premise' : 'pt-rule--derived';
  const fCls = node.status === 'goal' ? 'pt-formula--goal' : node.status === 'open' ? 'pt-formula--open' : '';
  return /*#__PURE__*/React.createElement("div", {
    className: "pt-node"
  }, /*#__PURE__*/React.createElement("div", {
    className: "pt-line"
  }, /*#__PURE__*/React.createElement("span", {
    className: `pt-formula ${fCls}`
  }, depth > 0 ? '⊢ ' : '', node.formula), /*#__PURE__*/React.createElement("span", {
    className: `pt-rule ${ruleCls}`
  }, node.tag ? node.tag + ' · ' : '', node.rule), node.peer && /*#__PURE__*/React.createElement("span", {
    className: "pt-prov"
  }, "\xB7 ", /*#__PURE__*/React.createElement("b", null, node.peer), " ", /*#__PURE__*/React.createElement("span", null, node.evid)), node.note && /*#__PURE__*/React.createElement("span", {
    className: "pt-note"
  }, "\u2014 ", node.note)), node.children && node.children.length > 0 && /*#__PURE__*/React.createElement("div", {
    className: "pt-children"
  }, node.children.map((c, i) => /*#__PURE__*/React.createElement(ProofNode, {
    key: i,
    node: c,
    depth: depth + 1
  }))));
}
function Stage({
  num,
  title,
  sub,
  triad,
  children
}) {
  return /*#__PURE__*/React.createElement("div", {
    className: `nb-stage ${triad ? 'nb-stage--triad' : ''}`
  }, /*#__PURE__*/React.createElement("div", {
    className: "nb-stage__head"
  }, /*#__PURE__*/React.createElement("span", {
    className: "nb-stage__num"
  }, num), /*#__PURE__*/React.createElement("span", {
    className: "nb-stage__title"
  }, title), sub && /*#__PURE__*/React.createElement("span", {
    className: "nb-stage__sub"
  }, sub)), /*#__PURE__*/React.createElement("div", {
    className: "nb-stage__body"
  }, children));
}
function Arrow({
  verb
}) {
  return /*#__PURE__*/React.createElement("div", {
    className: "nb-arrow"
  }, /*#__PURE__*/React.createElement("span", {
    className: "nb-arrow__g"
  }, "\u21E3"), /*#__PURE__*/React.createElement("span", {
    className: "nb-arrow__v"
  }, /*#__PURE__*/React.createElement("b", null, "logji"), " ", verb));
}

/** nibli — the Transparency Triad reasoning view: Source → Lojban → back-translation → proof. */
function NibliApp() {
  const [key, setKey] = React.useState('socrates');
  const [confirmed, setConfirmed] = React.useState(false);
  const q = QUERIES[key];
  const select = k => {
    setKey(k);
    setConfirmed(false);
  };
  return /*#__PURE__*/React.createElement("div", {
    className: "nb"
  }, /*#__PURE__*/React.createElement("header", {
    className: "nb-top"
  }, /*#__PURE__*/React.createElement("div", {
    className: "nb__wrap nb-top__row"
  }, /*#__PURE__*/React.createElement("div", {
    className: "nb-brand"
  }, /*#__PURE__*/React.createElement("img", {
    src: "../../assets/mark.svg",
    alt: ""
  }), "nibli ", /*#__PURE__*/React.createElement("small", null, "logji \xB7 tavla")), /*#__PURE__*/React.createElement("span", {
    className: "nb-top__sp"
  }), /*#__PURE__*/React.createElement("span", {
    className: "nb-tavla"
  }, /*#__PURE__*/React.createElement("span", {
    className: "nb-tavla__dot"
  }), "tavla \xB7 3 peers \xB7 synced"), /*#__PURE__*/React.createElement(__ds_scope.IconButton, {
    label: "Knowledge base"
  }, "\u2263"))), /*#__PURE__*/React.createElement("main", {
    className: "nb__wrap nb-main"
  }, /*#__PURE__*/React.createElement("div", {
    className: "nb-eyebrow"
  }, "// a symbolic reasoning engine \xB7 Lojban \u2192 first-order logic \xB7 ", /*#__PURE__*/React.createElement("b", null, "backward chaining")), /*#__PURE__*/React.createElement("div", {
    className: "nb-pills"
  }, Object.keys(QUERIES).map(k => /*#__PURE__*/React.createElement("span", {
    key: k,
    className: `nb-pill ${k === key ? 'nb-pill--on' : ''}`,
    onClick: () => select(k)
  }, QUERIES[k].label))), /*#__PURE__*/React.createElement("div", {
    className: "nb-composer"
  }, /*#__PURE__*/React.createElement("div", {
    className: "nb-input"
  }, /*#__PURE__*/React.createElement("span", {
    className: "nb-input__q"
  }, "xu"), q.query), /*#__PURE__*/React.createElement(__ds_scope.Button, {
    variant: "primary",
    size: "lg"
  }, "\u22A2 Derive")), /*#__PURE__*/React.createElement("div", {
    className: "nb-pipe"
  }, /*#__PURE__*/React.createElement(Stage, {
    num: "00",
    title: "Source",
    sub: "what you wrote"
  }, /*#__PURE__*/React.createElement("div", {
    className: "nb-nl nb-nl--strong",
    style: {
      marginBottom: 10
    }
  }, q.query), q.premises.map((p, i) => /*#__PURE__*/React.createElement("div", {
    className: "nb-prem",
    key: i
  }, /*#__PURE__*/React.createElement("span", {
    className: "nb-prem__tag"
  }, "P", i + 1), /*#__PURE__*/React.createElement("span", {
    className: "nb-nl"
  }, p)))), /*#__PURE__*/React.createElement(Arrow, {
    verb: "compiles to"
  }), /*#__PURE__*/React.createElement("div", {
    className: "nb-triad-label"
  }, "the transparency triad \u2014 nowhere for an unexamined step to hide"), /*#__PURE__*/React.createElement(Stage, {
    num: "01",
    title: "Lojban form",
    sub: "logji IR",
    triad: true
  }, /*#__PURE__*/React.createElement("div", {
    className: "nb-lojban"
  }, q.lojban.lines.map((l, i) => /*#__PURE__*/React.createElement("div", {
    key: i
  }, l)), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("span", {
    className: "q"
  }, q.lojban.query))), /*#__PURE__*/React.createElement("div", {
    className: "nb-fol"
  }, /*#__PURE__*/React.createElement("div", {
    className: "nb-fol__lab"
  }, "\u21B3 first-order logic"), q.fol.map((f, i) => /*#__PURE__*/React.createElement("div", {
    key: i
  }, f)))), /*#__PURE__*/React.createElement(Arrow, {
    verb: "back-translates to"
  }), /*#__PURE__*/React.createElement(Stage, {
    num: "02",
    title: "Back-translation",
    sub: "did we understand you?",
    triad: true
  }, q.back.map((b, i) => /*#__PURE__*/React.createElement("div", {
    className: "nb-nl",
    key: i,
    style: {
      padding: '2px 0'
    }
  }, b)), /*#__PURE__*/React.createElement("div", {
    className: `nb-confirm ${confirmed ? '' : 'nb-confirm--off'}`
  }, /*#__PURE__*/React.createElement("span", {
    className: "nb-confirm__txt"
  }, confirmed ? '✓ confirmed — this matches my intent' : 'Does this round-trip match what you meant?'), !confirmed && /*#__PURE__*/React.createElement(__ds_scope.Button, {
    size: "sm",
    variant: "secondary",
    onClick: () => setConfirmed(true)
  }, "\u2713 Confirm meaning"))), /*#__PURE__*/React.createElement(Arrow, {
    verb: "derives"
  }), /*#__PURE__*/React.createElement(Stage, {
    num: "03",
    title: "Proof tree",
    sub: q.timing,
    triad: true
  }, /*#__PURE__*/React.createElement("div", {
    className: "nb-proof"
  }, /*#__PURE__*/React.createElement("div", {
    className: "pt"
  }, /*#__PURE__*/React.createElement(ProofNode, {
    node: q.proof
  }))), /*#__PURE__*/React.createElement("div", {
    className: `nb-verdict nb-verdict--${q.verdict}`
  }, /*#__PURE__*/React.createElement("span", {
    className: "nb-verdict__glyph",
    style: {
      color: q.result.color
    }
  }, q.result.glyph), /*#__PURE__*/React.createElement("div", null, /*#__PURE__*/React.createElement("div", {
    className: "nb-verdict__main"
  }, q.result.main), /*#__PURE__*/React.createElement("div", {
    className: "nb-verdict__sub"
  }, q.result.sub))))), /*#__PURE__*/React.createElement("div", {
    className: "nb-foot"
  }, /*#__PURE__*/React.createElement("b", null, "Soundness, stated honestly:"), " nibli is sound with respect to ", /*#__PURE__*/React.createElement("b", null, "inference"), ", not ", /*#__PURE__*/React.createElement("b", null, "premises"), ". It will never derive a conclusion its rules don't license \u2014 but it has no view on whether the premises you fed it are true. The same guarantee Lean or Coq gives: the proof is checkable; the axioms are your problem.")));
}
Object.assign(__ds_scope, { NibliApp });
})(); } catch (e) { __ds_ns.__errors.push({ path: "ui_kits/nibli/NibliApp.jsx", error: String((e && e.message) || e) }); }

__ds_ns.Avatar = __ds_scope.Avatar;

__ds_ns.Badge = __ds_scope.Badge;

__ds_ns.Button = __ds_scope.Button;

__ds_ns.Callout = __ds_scope.Callout;

__ds_ns.Card = __ds_scope.Card;

__ds_ns.IconButton = __ds_scope.IconButton;

__ds_ns.Spinner = __ds_scope.Spinner;

__ds_ns.Tooltip = __ds_scope.Tooltip;

__ds_ns.Checkbox = __ds_scope.Checkbox;

__ds_ns.Input = __ds_scope.Input;

__ds_ns.Select = __ds_scope.Select;

__ds_ns.Switch = __ds_scope.Switch;

__ds_ns.Tabs = __ds_scope.Tabs;

__ds_ns.ConsoleApp = __ds_scope.ConsoleApp;

__ds_ns.LandingPage = __ds_scope.LandingPage;

__ds_ns.NibliApp = __ds_scope.NibliApp;

})();
