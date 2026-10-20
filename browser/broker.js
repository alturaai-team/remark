(() => {
  if (globalThis.__remarkBroker) return;
  globalThis.__remarkBroker = true;
  addEventListener("message", (event) => {
    if (event.source !== window || event.origin !== location.origin || event.data?.type !== "remark-selection") return;
    const selection = event.data.selection;
    const report = (ok) => window.postMessage({type:"remark-selection-result",id:selection.id,ok},location.origin);
    chrome.runtime.sendMessage({type:"remark-selection",t:Date.now(),target:event.data.target,selection})
      .then((result) => report(result?.ok === true)).catch(() => report(false));
  });
})();
