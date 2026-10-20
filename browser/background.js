async function connection() {
  const response = await fetch(chrome.runtime.getURL("connection.json"));
  if (!response.ok) throw new Error("Odpri Remark");
  return response.json();
}
chrome.action.onClicked.addListener(async (tab) => {
  try {
    await connection();
    await chrome.scripting.executeScript({target:{tabId:tab.id,allFrames:true},files:["broker.js"],world:"ISOLATED"});
    await chrome.scripting.executeScript({target:{tabId:tab.id,allFrames:true},files:["context.js","picker.js"],world:"MAIN"});
    await chrome.action.setBadgeText({tabId:tab.id,text:""});
  } catch {
    await chrome.action.setBadgeText({tabId:tab.id,text:"!"});
    await chrome.action.setTitle({tabId:tab.id,title:"Odpri Remark"});
  }
});
chrome.runtime.onMessage.addListener((message,sender,reply) => {
  if (message.type !== "remark-selection" || !sender.tab) return;
  (async () => {
    const config = await connection();
    const window = await chrome.windows.get(sender.tab.windowId);
    const tab = await chrome.tabs.get(sender.tab.id);
    if (!window.focused || !tab.active) throw new Error("Zavihek ni aktiven");
    const url = new URL(sender.url);
    message.selection.url = url.origin + url.pathname;
    message.selection.frameId = sender.frameId;
    const body = {tab:String(sender.tab.id)+":"+sender.frameId,events:[{
      t:message.t,kind:"annotation",focused:true,target:message.target,selection:message.selection
    }]};
    const response = await fetch("http://127.0.0.1:"+config.port+"/web/annotations",{
      method:"POST",headers:{"Content-Type":"application/json","Authorization":"Bearer "+config.token},body:JSON.stringify(body)
    });
    if (!response.ok || (await response.json()).ok !== true) throw new Error("Remark ni povezan");
    reply({ok:true});
  })().catch(() => {
    chrome.action.setBadgeText({tabId:sender.tab.id,text:"!"});
    chrome.action.setTitle({tabId:sender.tab.id,title:"Odpri Remark"});
    reply({ok:false});
  });
  return true;
});
