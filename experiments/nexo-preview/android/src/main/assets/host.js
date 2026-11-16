/* Local Android transport. Only opaque controls exposed by the current native view can run. */
(function () {
  'use strict';
  if (!window.HorizonNative) return;
  let serial=0, snapshot=null;
  const pending=new Map(),listeners=new Set();
  function request(op,body={}) {
    const id=++serial;
    return new Promise((resolve,reject)=>{
      const timer=setTimeout(()=>{pending.delete(id);reject(Error('O Horizon não respondeu. Tente novamente.'));},20000);
      pending.set(id,{resolve,reject,timer});
      try { if(!HorizonNative.request(JSON.stringify({id,op,...body})))throw Error('Este controle não está disponível agora.'); }
      catch(error){clearTimeout(timer);pending.delete(id);reject(error);}
    });
  }
  window.HorizonHost={
    request,get:()=>snapshot,
    subscribe(fn){listeners.add(fn);if(snapshot)fn(snapshot);return()=>listeners.delete(fn);},
    receive(json){
      let value;try{value=JSON.parse(json);}catch(_){return;}
      if((value.schema!==1&&value.schema!==2)||value.live!==true||(value.schema===1&&!Array.isArray(value.controls)))return;
      snapshot=value;
      for(const reply of value.replies||[]){const p=pending.get(reply.id);if(!p)continue;clearTimeout(p.timer);pending.delete(reply.id);reply.error?p.reject(Error(reply.error)):p.resolve(value);}
      for(const listener of listeners)listener(value);
    },
    back(){window.Nexo?.back(true);}
  };
  window.NexoNativeHost={launchGame:()=>request('play')};
})();
