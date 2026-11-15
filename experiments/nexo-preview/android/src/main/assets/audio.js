/* Original player for bundled Kenney CC0 recordings; no synthesis or remote audio. */
(function(){
  'use strict';
  const clips={select:['select_001.ogg',.58],open:['maximize_007.ogg',.8],back:['minimize_007.ogg',.72],confirm:['confirmation_001.ogg',.7],error:['error_003.ogg',.6],toggle:['toggle_001.ogg',.6],click:['rollover2.ogg',.55]};
  const players=new Map(),played={},failures={};let visible=!document.hidden,volume=.25,lastKind='',lastAt=-1000;
  const clamp=n=>Math.max(0,Math.min(1,Number(n)||0));
  for(const [kind,[file,gain]] of Object.entries(clips)){
    const audio=new Audio('sounds/'+file);audio.preload='auto';audio.volume=volume*gain;
    audio.addEventListener('error',()=>{failures[kind]=audio.error?.code||'load';});
    audio.load();players.set(kind,audio);
  }
  function stop(){for(const a of players.values()){a.pause();try{a.currentTime=0;}catch(_){}}}
  function setVolume(value){volume=clamp(value);for(const [kind,a] of players)a.volume=volume*clips[kind][1];if(!volume)stop();}
  function play(kind='click',value=volume){
    setVolume(value);if(!visible||document.hidden||!volume)return;
    const a=players.get(kind);if(!a)return;
    const now=performance.now();if(lastKind===kind&&now-lastAt<90)return;
    lastKind=kind;lastAt=now;
    if(['open','back','error'].includes(kind))stop();
    a.pause();try{a.currentTime=0;}catch(_){}
    // play() stays in the actual touch/click handler, including the first cue.
    a.play().then(()=>{played[kind]=(played[kind]||0)+1;delete failures[kind];}).catch(e=>{if(e.name!=='AbortError')failures[kind]=e.name;});
  }
  window.NexoAudio={play,setVolume,setVisible(value){visible=!!value;if(!visible)stop();},
    getState(){return {visible,volume,played:{...played},failures:{...failures},clips:[...players].map(([kind,a])=>({kind,src:a.currentSrc||a.src,ready:a.readyState,time:a.currentTime,duration:a.duration,paused:a.paused,volume:a.volume}))};}};
})();
