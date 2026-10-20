(() => {
    const clip = (text, max) => {
      const s = String(text == null ? "" : text).replace(/\s+/g, " ").trim();
      return s ? s.slice(0, max) : undefined;
    };

    const selectorInRoot = (element) => {
      const root = element.getRootNode();
      const parts = [];
      for (let node = element; node; node = node.parentElement) {
        let part = node.tagName.toLowerCase();
        const candidates = [];
        if (node.id) candidates.push("#" + CSS.escape(node.id));
        for (const name of ["data-testid", "data-test", "data-qa", "data-cy", "data-component"]) {
          const value = node.getAttribute(name);
          if (value) candidates.push("[" + name + "=" + CSS.escape(value) + "]");
        }
        for (const candidate of candidates) {
          if (root.querySelectorAll(candidate).length === 1) return [candidate, ...parts].join(" > ");
        }
        const parent = node.parentNode;
        if (parent && parent.children) {
          const siblings = [...parent.children].filter((child) => child.tagName === node.tagName);
          if (siblings.length > 1) part += ":nth-of-type(" + (siblings.indexOf(node) + 1) + ")";
        }
        parts.unshift(part);
        if (root.querySelectorAll(parts.join(" > ")).length === 1) return parts.join(" > ");
      }
      return parts.join(" > ");
    };
    const selectorPath = (element) => {
      const path = [];
      for (let node = element; node;) {
        path.unshift(selectorInRoot(node));
        const root = node.getRootNode();
        node = root instanceof ShadowRoot ? root.host : null;
      }
      return path;
    };
    const ignored = /\/node_modules\/|\/\.vite\/deps\/|react-dom|jsx-dev-runtime|jsx-runtime|\/sdk\.js/;
    // `src/ui/Pill.tsx:142` from the first stack frame in the project's own code.
    const stackSource = (stack) => {
      for (const line of String(stack || "").split("\n")) {
        const m = line.match(/((?:https?|file):\/\/[^\s()]+?):(\d+):\d+\)?\s*$/);
        if (!m || ignored.test(m[1])) continue;
        try {
          const path = decodeURIComponent(new URL(m[1]).pathname).replace(/^\/(@fs\/)?/, "");
          return path + ":" + m[2];
        } catch {}
      }
      return undefined;
    };
    const short = (file, line) => {
      try {
        return decodeURIComponent(new URL(file, location.href).pathname).replace(/^\//, "") + (line ? ":" + line : "");
      } catch {
        return undefined;
      }
    };
    // Source locators some dev setups add (solid-devtools, react-dev-inspector, Sentry).
    const located = (el) => {
      const d = el.dataset || {};
      if (d.sourceLoc) return { source: d.sourceLoc.replace(/^(.*:\d+):\d+$/, "$1") };
      if (d.inspectorRelativePath) return { source: d.inspectorRelativePath + (d.inspectorLine ? ":" + d.inspectorLine : "") };
      if (d.sentrySourceFile) return { component: d.sentryComponent, source: d.sentrySourceFile };
      return null;
    };
    const fiberOf = (el) => {
      for (const key in el) {
        if (key.startsWith("__reactFiber$") || key.startsWith("__reactInternalInstance$")) return el[key];
      }
      return null;
    };
    const reactInfo = (fiber) => {
      const out = {};
      const src = fiber._debugSource;
      if (src && src.fileName) out.source = short(src.fileName, src.lineNumber);
      else if (fiber._debugStack) out.source = stackSource(fiber._debugStack.stack);
      const names = [];
      for (let f = fiber, depth = 0; f && depth < 64; f = f.return, depth++) {
        const type = f.type;
        const fn = typeof type === "function" ? type : type && typeof type === "object" ? type.render || type.type : null;
        const name = fn && (fn.displayName || fn.name);
        if (name) {
          if (!names.includes(name)) names.unshift(name);
        }
      }
      if (names.length) out.component = names.slice(-8).join(" > ");
      return out;
    };
    const component = (el) => {
      for (let node = el, depth = 0; node && depth < 30; node = node.parentElement, depth++) {
        const found = located(node);
        if (found) return found;
        const fiber = fiberOf(node);
        if (fiber) return reactInfo(fiber);
      }
      return {};
    };
    const typed = "input, textarea, select, [contenteditable=''], [contenteditable='true']";
    const describe = (el) => {
      if (!(el instanceof Element)) return undefined;
      el = el.closest("button, a, [role=button], [role=tab], [role=menuitem], [role=link], label, summary") || el;
      const target = { tag: el.tagName.toLowerCase() };
      if (el.id) target.id = clip(el.id, 60);
      const cls = [...el.classList].filter((c) => c.length <= 30 && !/[:[\]/]/.test(c)).slice(0, 2);
      if (cls.length) target.cls = cls.join(".");
      const field = el.closest(typed);
      let text =
        el.getAttribute("aria-label") || el.getAttribute("title") || el.getAttribute("alt") || el.getAttribute("data-testid");
      if (!text && field) {
        if (field.getAttribute("type") !== "password") text = field.getAttribute("placeholder") || field.getAttribute("name");
      } else if (!text) {
        text = (el.textContent || "").slice(0, 200);
      }
      const label = clip(text, 60);
      if (label) target.text = label;
      const rect = el.getBoundingClientRect();
      const attributes = {};
      for (const name of ["id", "data-testid", "data-test", "data-qa", "data-cy", "data-component", "role", "aria-label"]) {
        const value = el.getAttribute(name);
        if (value) attributes[name] = value.slice(0, 500);
      }
      const styles = {};
      const computed = getComputedStyle(el);
      for (const name of ["display", "position", "color", "background-color", "font-family", "font-size", "font-weight", "line-height", "padding", "margin", "gap", "border-radius", "width", "height"]) {
        styles[name] = computed.getPropertyValue(name);
      }
      const url = new URL(location.href);
      return Object.assign(target, component(el), { selection: {
        url: url.origin + url.pathname,
        selectorPath: selectorPath(el), rectViewport: [rect.x, rect.y, rect.width, rect.height],
        scroll: [scrollX, scrollY], attributes, styles
      }});
    };

    globalThis.RemarkContext = { describe, stackSource };
})();
