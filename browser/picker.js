(() => {
  if (globalThis.__remarkPicker) { globalThis.__remarkPicker.toggle(); return; }
  let enabled = false;
  let count = 0;
  const markers = new Map();
  const host = document.createElement("div");
  host.setAttribute("aria-label", "Označi element");
  host.style.cssText = "position:fixed;inset:0;pointer-events:none;z-index:2147483647";
  const root = host.attachShadow({mode:"open"});
  const hover = document.createElement("div");
  hover.style.cssText = "display:none;position:fixed;border:2px solid #e53935;background:#e5393514;box-sizing:border-box;pointer-events:none";
  root.append(hover);
  document.documentElement.append(host);
  const hit = (event) => {
    const first = event.composedPath()[0];
    return first instanceof Element && !host.contains(first) ? first : null;
  };
  const move = (event) => {
    if (!enabled) return;
    const element = hit(event);
    if (!element) return;
    const described = RemarkContext.describe(element);
    if (!described) return;
    const [x,y,w,h] = described.selection.rectViewport;
    hover.style.display = "block";
    Object.assign(hover.style,{left:x+"px",top:y+"px",width:w+"px",height:h+"px"});
  };
  const click = (event) => {
    if (!enabled || event.button !== 0) return;
    const element = hit(event);
    if (!element) return;
    const described = RemarkContext.describe(element);
    if (!described) return;
    event.preventDefault();event.stopImmediatePropagation();
    const {selection,...target} = described;
    selection.id = crypto.randomUUID();
    selection.number = ++count;
    if (markers.size === 300) {
      const oldest = markers.keys().next().value;
      markers.get(oldest).marker.remove();markers.delete(oldest);
    }
    const marker = document.createElement("div");
    const [x,y] = selection.rectViewport;
    marker.textContent = String(selection.number);
    marker.style.cssText = "position:fixed;background:#e53935;color:white;font:600 12px system-ui;border-radius:50%;width:20px;height:20px;text-align:center;line-height:20px;pointer-events:none";
    Object.assign(marker.style,{left:x+"px",top:y+"px",opacity:"0.5"});root.append(marker);
    markers.set(selection.id,{element,marker});
    window.postMessage({type:"remark-selection", target, selection}, location.origin);
  };
  const toggle = () => {
    enabled = !enabled;
    if (enabled) {
      for (const {marker} of markers.values()) marker.remove();
      markers.clear();
    }
    if (!enabled) hover.style.display = "none";
    if (enabled) document.documentElement.style.setProperty("cursor","crosshair");
    else document.documentElement.style.removeProperty("cursor");
  };
  addEventListener("pointermove",move,true);
  addEventListener("pointerdown",click,true);
  addEventListener("click",(event) => {if (enabled) {event.preventDefault();event.stopImmediatePropagation();}},true);
  addEventListener("message",(event) => {
    if (event.source !== window || event.origin !== location.origin || event.data?.type !== "remark-selection-result") return;
    const saved = markers.get(event.data.id);
    if (!saved) return;
    saved.marker.style.opacity = "1";
    if (!event.data.ok) {saved.marker.remove();markers.delete(event.data.id);}
  });
  const update = () => {
    hover.style.display="none";
    for (const [id,{element,marker}] of markers) {
      if (!element.isConnected) {marker.remove();markers.delete(id);continue;}
      marker.style.display="block";
      const rect=element.getBoundingClientRect();
      Object.assign(marker.style,{left:rect.x+"px",top:rect.y+"px"});
    }
  };
  addEventListener("scroll",update,true);addEventListener("resize",update);
  addEventListener("keydown",(event) => {if (event.key === "Escape" && enabled) toggle();},true);
  globalThis.__remarkPicker = {toggle};toggle();
})();
