/* Original Nexo light-trace introduction, synchronized with the actual 3D scene. */
(function(){
 'use strict';
 const dialog=document.getElementById('intro-dialog'),root=document.getElementById('nexo-app'),word=document.getElementById('intro-word'),mark=document.getElementById('intro-mark'),duration=1850;
 let active=false,waiting=false,started=0,pausedAt=0,pausedTotal=0,animations=[],generation=0,inModule=false,completed=false,visible=!document.hidden;
 const clamp=n=>Math.max(0,Math.min(1,n)),smooth=n=>{n=clamp(n);return n*n*(3-2*n);};
 function finish(skipped=false){
  generation++;active=false;waiting=false;completed=true;for(const a of animations)a.cancel();animations=[];
  document.body.classList.remove('intro-active');root.removeAttribute('aria-busy');if(dialog.open)dialog.close();
  window.NexoScene?.resetMotionHistory();window.NexoScene?.refresh();root.dataset.intro=skipped?'skipped':'complete';
 }
 function animate(node,frames,options){const a=node.animate(frames,{fill:'both',...options});animations.push(a);return a;}
 async function start({replay=false}={}){
  const prefs=window.NexoApp?.store.get().preferences,reduced=matchMedia('(prefers-reduced-motion: reduce)').matches||prefs?.reducedMotion||!!window.NexoApp?.reducedMotion?.();
  if(reduced||(!replay&&(prefs?.introEnabled===false||window.HorizonNative?.skipIntro?.()))){finish();return;}
  for(const a of animations)a.cancel();animations=[];const token=++generation;active=true;waiting=true;completed=false;pausedAt=0;pausedTotal=0;inModule=!!window.NexoScene?.getState().open;
  document.body.classList.add('intro-active');root.setAttribute('aria-busy','true');root.dataset.intro='playing';if(!dialog.open)dialog.showModal();dialog.setAttribute('tabindex','-1');dialog.focus({preventScroll:true});
  await Promise.race([document.fonts.load('48px Michroma'),new Promise(resolve=>setTimeout(resolve,300))]);if(token!==generation||!active)return;
  waiting=false;started=performance.now();word.style.transform='';mark.style.transform='';
  const target=document.querySelector('.wordmark').getBoundingClientRect(),from=word.getBoundingClientRect(),scale=target.width/Math.max(1,from.width),dx=target.x+target.width/2-from.x-from.width/2,dy=target.y+target.height/2-from.y-from.height/2;
  animate(dialog,[{opacity:1},{opacity:1,offset:.7},{opacity:0}],{duration});
  animate(dialog.querySelector('.intro-stroke'),[{strokeDashoffset:1},{strokeDashoffset:0}],{duration:750,delay:100,easing:'cubic-bezier(.25,.65,.2,1)'});
  animate(dialog.querySelector('.intro-orbit'),[{strokeDashoffset:1,opacity:0},{strokeDashoffset:.15,opacity:.65,offset:.65},{strokeDashoffset:0,opacity:.2}],{duration:1100,easing:'ease-out'});
  animate(mark,[{opacity:0,transform:'translateY(14px) rotate(-8deg) scale(.9)'},{opacity:1,transform:'translateY(0) rotate(0) scale(1)',offset:.6},{opacity:0,transform:'translateY(-10px) scale(.8)'}],{duration:1350,easing:'cubic-bezier(.2,.7,.2,1)'});
  animate(word,[{opacity:0,clipPath:'inset(0 100% 0 0)',letterSpacing:'.36em'},{opacity:1,clipPath:'inset(0 0 0 0)',letterSpacing:'.2em'}],{duration:650,delay:360,easing:'ease-out'});
  animate(word,inModule?[{transform:'none',opacity:1},{transform:'scale(.92)',opacity:0}]:[{transform:'none',opacity:1},{transform:`translate(${dx}px,${dy}px) scale(${scale})`,opacity:1}],{duration:600,delay:1200,easing:'cubic-bezier(.65,0,.2,1)'});
  for(const [i,node] of [...dialog.querySelectorAll('.intro-ray')].entries())animate(node,[{opacity:0,transform:`translateX(${i?-90:90}px) rotate(${i?-16:16}deg)`},{opacity:.75,offset:.45},{opacity:0,transform:`translateX(${i?90:-90}px) rotate(${i?-16:16}deg)`}],{duration:1250,easing:'ease-in-out'});
  if(!visible){pausedAt=started;for(const a of animations)a.pause();}
  window.NexoScene?.refresh();
 }
 function getState(now=performance.now()){
  if(!active)return {active:false,completed,progress:1,sceneReveal:1};if(waiting)return {active:true,completed:false,progress:0,sceneReveal:inModule?1:0};
  const progress=clamp(((pausedAt||now)-started-pausedTotal)/duration);if(progress>=1){finish();return {active:false,completed:true,progress:1,sceneReveal:1};}
  return {active:true,completed:false,progress,sceneReveal:inModule?1:smooth((progress-.52)/.43)};
 }
 function setVisible(value){visible=!!value;if(!active||waiting)return;if(!visible&&!pausedAt){pausedAt=performance.now();for(const a of animations)a.pause();}else if(visible&&pausedAt){pausedTotal+=performance.now()-pausedAt;pausedAt=0;for(const a of animations)a.play();}}
 dialog.addEventListener('click',e=>{e.preventDefault();e.stopPropagation();finish(true);});dialog.addEventListener('cancel',e=>{e.preventDefault();finish(true);});
 window.NexoIntro={start,getState,setVisible,skip:()=>finish(true),isActive:()=>active};
})();
