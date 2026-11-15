/* Additive GPU finish for the existing v1.4 perspective deck. CSS still owns
 * composition and materials; measured corner projections keep the GPU coat
 * aligned with the original cards throughout their existing transitions. */
(function(){
  'use strict';
  const VERTEX=`#version 300 es
in vec3 aClip;
in vec2 aUV;
out vec2 vUV;
void main(){gl_Position=vec4(aClip.xy,0.,aClip.z);vUV=aUV;}`;
  const HEADER=`#version 300 es
precision highp float;
in vec2 vUV;
out vec4 fragmentColor;
uniform vec2 uResolution,uViewport,uPointer,uCoatOrigin,uCoatSize,uCardSize;
uniform vec3 uNormal;
uniform float uTime,uReduced,uRadius,uOpacity,uGlassOn;
uniform vec4 uPalette[5],uAtmos,uBackdropShape,uBackdropLayers,uBackdropLight,uBackdropDetail;
uniform vec4 uSection;
uniform vec4 uSceneNeutral[3],uEnvironment,uLight,uMaterial,uSurface,uMotion,uGlassModel,uGlassOptics;
`;
  const CORE=`
void main(){
  vec2 local=(vUV-.5)*uCardSize;
  float edge=sround(local,vec2(0.),uCardSize,uRadius);
  float mask=1.-smoothstep(-.65,.6,edge);
  if(mask<=0.)discard;
  vec2 screen=vec2(gl_FragCoord.x/uResolution.x,1.-gl_FragCoord.y/uResolution.y)*uCoatSize+uCoatOrigin;
  vec2 uv=screen/uViewport;
  vec2 cursor=uPointer/uViewport-.5;
  float angle=radians(uLight.x);
  vec3 L=normalize(vec3(cos(angle)*.87+cursor.x*uMotion.z,sin(angle)*.87-cursor.y*uMotion.z,uLight.y+.26));
  float bevel=smoothstep(-7.,-.5,edge);
  vec3 N=normalize(uNormal+vec3((vUV-.5)*bevel*uMaterial.w*.65,0.));
  vec3 V=vec3(0.,0.,1.),H=normalize(V+L);
  float nl=max(dot(N,L),.01),nv=max(dot(N,V),.01),nh=max(dot(N,H),0.);
  float rough=clamp(uMaterial.x,.075,.96);
  float spec=ggxD(nh,rough)*ggxG(nv,nl,rough)*.04/(4.*nv*nl);
  float rim=1.-smoothstep(.5,2.0,abs(edge+1.));
  float time=uTime*uMotion.x*(1.-uReduced);
  vec2 envUV=clamp(uv+N.xy*.065,vec2(0.),vec2(1.));
  vec3 env=renderBackdrop(envUV,screen,time,uPointer/uViewport);
  vec3 pearl=mix(uSceneNeutral[2].rgb,vec3(1.),.22);
  vec3 reflection=mix(pearl,env,.34+.25*uMaterial.y);
  float highlight=min(spec,.75)*uMaterial.z*uLight.z;
  float fresnel=pow(1.-nv,3.)*uGlassOptics.z;
  float transmission=uGlassOn*uGlassModel.w*(1.-uGlassModel.y*.76);
  float alpha=highlight*.20+rim*(.025+.075*nl)*uSurface.z+fresnel*.13;
  alpha+=transmission*.035;
  vec3 tint=mix(reflection,env,transmission*.35);
  // This is a transparent reflective finish over the original material,
  // not a replacement surface or a second opaque panel.
  fragmentColor=vec4(tint,clamp(alpha*mask*uOpacity,0.,.28));
}`;
  // A unit-square homography from four observed CSS-projected corners.
  function homography(p){
    const [a,b,c,d]=p,dx1=b[0]-c[0],dx2=d[0]-c[0],dx3=a[0]-b[0]+c[0]-d[0];
    const dy1=b[1]-c[1],dy2=d[1]-c[1],dy3=a[1]-b[1]+c[1]-d[1];
    const det=dx1*dy2-dx2*dy1;
    if(Math.abs(det)<1e-8)return null;
    const g=(dx3*dy2-dx2*dy3)/det,h=(dx1*dy3-dx3*dy1)/det;
    return [b[0]-a[0]+g*b[0],d[0]-a[0]+h*d[0],a[0],b[1]-a[1]+g*b[1],d[1]-a[1]+h*d[1],a[1],g,h,1];
  }
  function project(m,u,v){const w=m[6]*u+m[7]*v+m[8];return[(m[0]*u+m[1]*v+m[2])/w,(m[3]*u+m[4]*v+m[5])/w];}
  function shaderSource(){const s=window.JadeRenderer.fragmentSource;return HEADER+s.slice(s.indexOf('float hash12'),s.indexOf('// Blur the ACTUAL'))+CORE;}
  let lastFrame=null;
  const markerCache=new WeakMap();
  let canvas,gl,program,buffer,uniform={},tiles=[],cards=[],source='',ready=false,error='',until=0,lastRead=0,lastLight=0,reads=0;
  let bounds={x:0,y:0,w:1,h:1},lastViewport='';
  const colors=h=>{const n=parseInt(String(h).replace('#',''),16)||0;return[(n>>16&255)/255,(n>>8&255)/255,(n&255)/255,1];};
  function init(){
    if(ready)return true;
    canvas=document.getElementById('deck-coat');if(!canvas)return false;
    try{
      gl=canvas.getContext('webgl2',{alpha:true,premultipliedAlpha:false,depth:false,stencil:false,antialias:false,preserveDrawingBuffer:false});
      if(!gl)throw Error('WebGL2 indisponível para acabamento');
      source=shaderSource();
      const compile=(type,s)=>{const sh=gl.createShader(type);gl.shaderSource(sh,s);gl.compileShader(sh);if(!gl.getShaderParameter(sh,gl.COMPILE_STATUS))throw Error(gl.getShaderInfoLog(sh));return sh;};
      const vs=compile(gl.VERTEX_SHADER,VERTEX),fs=compile(gl.FRAGMENT_SHADER,source);
      program=gl.createProgram();gl.attachShader(program,vs);gl.attachShader(program,fs);gl.bindAttribLocation(program,0,'aClip');gl.bindAttribLocation(program,1,'aUV');gl.linkProgram(program);gl.deleteShader(vs);gl.deleteShader(fs);
      if(!gl.getProgramParameter(program,gl.LINK_STATUS))throw Error(gl.getProgramInfoLog(program));
      buffer=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,buffer);gl.bufferData(gl.ARRAY_BUFFER,120,gl.DYNAMIC_DRAW);
      for(const k of ['uResolution','uViewport','uPointer','uCoatOrigin','uCoatSize','uCardSize','uNormal','uTime','uReduced','uRadius','uOpacity','uGlassOn','uPalette[0]','uSceneNeutral[0]','uAtmos','uBackdropShape','uBackdropLayers','uBackdropLight','uBackdropDetail','uEnvironment','uSection','uLight','uMaterial','uSurface','uMotion','uGlassModel','uGlassOptics'])uniform[k]=gl.getUniformLocation(program,k);
      register();
      document.addEventListener('scroll',()=>{until=performance.now()+200;},{capture:true,passive:true});
      window.addEventListener('resize',()=>{until=performance.now()+750;});
      canvas.addEventListener('webglcontextlost',e=>{e.preventDefault();ready=false;error='Contexto de acabamento perdido';canvas.hidden=true;});
      ready=true;canvas.hidden=false;until=performance.now()+750;return true;
    }catch(e){error=String(e.message||e);canvas.hidden=true;return false;}
  }
  function register(){
    tiles=Array.from(document.querySelectorAll('[data-spatial-surface]')).map(el=>{
      let markers=markerCache.get(el);
      if(!markers){markers=Array.from({length:4},(_,i)=>{const m=document.createElement('i');m.className='deck-corner';m.setAttribute('aria-hidden','true');m.style.cssText='position:absolute;width:0;height:0;pointer-events:none;'+(i===0||i===3?'left:0;':'right:0;')+(i<2?'top:0;':'bottom:0;');el.appendChild(m);return m;});markerCache.set(el,markers);}
      return {el,markers};
    });
    until=performance.now()+760;
  }
  function measure(){
    const order={rear:0,middle:1,front:2};
    cards=tiles.map(o=>{
      const style=getComputedStyle(o.el),rect=o.el.getBoundingClientRect();
      if(style.visibility==='hidden'||style.display==='none'||rect.width<2||rect.height<2)return null;
      if(o.el.dataset.stack&&!(o.el.dataset.stack in order))return null;
      let opacity=Number(style.opacity),matrix=new DOMMatrixReadOnly(style.transform),clip=[0,0,innerWidth,innerHeight];
      for(let parent=o.el.parentElement;parent&&parent!==document.body;parent=parent.parentElement){
        const ps=getComputedStyle(parent);opacity*=Number(ps.opacity);if(ps.display==='none'||ps.visibility==='hidden')return null;
        if(ps.transform!=='none')matrix=new DOMMatrixReadOnly(ps.transform).multiply(matrix);
        if(['auto','scroll','hidden','clip'].includes(ps.overflowX)||['auto','scroll','hidden','clip'].includes(ps.overflowY)){
          const r=parent.getBoundingClientRect();clip=[Math.max(clip[0],r.left),Math.max(clip[1],r.top),Math.min(clip[2],r.right),Math.min(clip[3],r.bottom)];
        }
      }
      if(opacity<.005||clip[2]<=clip[0]||clip[3]<=clip[1])return null;
      const p=o.markers.map(m=>{const r=m.getBoundingClientRect();return[r.x,r.y];}),homographyMatrix=homography(p);if(!homographyMatrix)return null;
      const normal=[matrix.m31,matrix.m32,matrix.m33],norm=Math.hypot(...normal)||1;
      const kind=o.el.dataset.spatialSurface,level=kind==='button'?30:kind==='card'?10:0;
      return {id:o.el.dataset.page||o.el.id||kind,stack:o.el.dataset.stack||kind,order:level+(order[o.el.dataset.stack]||0),p,matrix:homographyMatrix,w:o.el.offsetWidth,h:o.el.offsetHeight,radius:parseFloat(style.borderTopLeftRadius)||9,opacity,normal:normal.map(n=>n/norm),clip};
    }).filter(Boolean).sort((a,b)=>a.order-b.order);
    const points=cards.flatMap(c=>c.p),xs=points.map(p=>p[0]),ys=points.map(p=>p[1]);
    if(points.length){const x=Math.max(0,Math.floor(Math.min(...xs))-2),y=Math.max(0,Math.floor(Math.min(...ys))-2);bounds={x,y,w:Math.max(1,Math.min(innerWidth,Math.ceil(Math.max(...xs))+2)-x),h:Math.max(1,Math.min(innerHeight,Math.ceil(Math.max(...ys))+2)-y)};}
    reads++;
  }
  function frame(p,time,pointer,paused,scale){
    if(!ready)return;
    lastFrame=[p,time,pointer,paused,scale];
    const now=performance.now(),vp=innerWidth+'x'+innerHeight;
    if(now<until||now-lastRead>400||vp!==lastViewport){measure();lastRead=now;lastViewport=vp;}
    const w=Math.max(2,Math.round(bounds.w*scale)),h=Math.max(2,Math.round(bounds.h*scale));
    if(canvas.width!==w||canvas.height!==h){canvas.width=w;canvas.height=h;}
    Object.assign(canvas.style,{left:bounds.x+'px',top:bounds.y+'px',width:bounds.w+'px',height:bounds.h+'px'});
    gl.viewport(0,0,w,h);gl.disable(gl.SCISSOR_TEST);gl.clearColor(0,0,0,0);gl.clear(gl.COLOR_BUFFER_BIT);gl.useProgram(program);
    gl.bindBuffer(gl.ARRAY_BUFFER,buffer);gl.enableVertexAttribArray(0);gl.vertexAttribPointer(0,3,gl.FLOAT,false,20,0);gl.enableVertexAttribArray(1);gl.vertexAttribPointer(1,2,gl.FLOAT,false,20,12);
    // Front geometry replaces rear coating in its footprint; alpha blending
    // here would leak the hidden rear card's highlight through the front.
    gl.disable(gl.BLEND);
    const v4=(k,a)=>gl.uniform4fv(uniform[k],a);
    gl.uniform2f(uniform.uResolution,w,h);gl.uniform2f(uniform.uViewport,innerWidth,innerHeight);gl.uniform2f(uniform.uPointer,...pointer);gl.uniform2f(uniform.uCoatOrigin,bounds.x,bounds.y);gl.uniform2f(uniform.uCoatSize,bounds.w,bounds.h);gl.uniform1f(uniform.uTime,time);gl.uniform1f(uniform.uReduced,p.reducedMotion?1:0);gl.uniform1f(uniform.uGlassOn,p.glassCards?1:0);
    gl.uniform4fv(uniform['uPalette[0]'],['bg','surface','accent','cool','warm'].flatMap(k=>colors(p[k])));gl.uniform4fv(uniform['uSceneNeutral[0]'],['sceneNeutral','sceneMetal','sceneHighlight'].flatMap(k=>colors(p[k])));
    v4('uSection',window.JadeNavigation?.environment()||[0,0,0,0]);
    v4('uAtmos',[p.fog,p.waves,p.noise,p.vignette]);v4('uLight',[p.azimuth,p.elevation,p.intensity,p.ambient]);v4('uMaterial',[p.roughness,p.metallic,p.reflectivity,p.bevel]);v4('uSurface',[p.depth,p.ao,p.edgeLight,p.heroGlow]);v4('uMotion',[p.speed,p.pulse,p.parallax,p.bloom]);v4('uEnvironment',[p.environmentLight,1,p.environmentWarmth,1]);
    v4('uBackdropShape',[Math.max(0,['ribbons','aurora','satin','halo','minimal'].indexOf(p.backgroundStyle)),p.backgroundIntensity,p.backgroundScale,p.ribbonCurve]);v4('uBackdropLayers',[p.ribbonLayers,p.ribbonWidth,p.ribbonSoftness,p.backgroundFlow]);v4('uBackdropLight',[p.backgroundGlow,p.gradientDepth,p.backgroundCenterX,p.backgroundCenterY]);v4('uBackdropDetail',[p.backgroundGrain,p.backgroundParallax,p.ribbonGleam,p.backgroundContrast]);v4('uGlassModel',[0,p.glassOpacity,p.glassBlur,p.glassTransmission]);v4('uGlassOptics',[p.glassTint,p.glassScatter,p.glassFresnel,p.glassRim]);
    gl.enable(gl.SCISSOR_TEST);
    for(const card of cards){
      const left=Math.max(0,Math.floor((card.clip[0]-bounds.x)*scale)),right=Math.min(w,Math.ceil((card.clip[2]-bounds.x)*scale));
      const top=Math.max(0,Math.floor((card.clip[1]-bounds.y)*scale)),bottom=Math.min(h,Math.ceil((card.clip[3]-bounds.y)*scale));
      if(right<=left||bottom<=top)continue;gl.scissor(left,h-bottom,right-left,bottom-top);
      const vertices=[];for(const [u,v] of [[0,0],[1,0],[0,1],[0,1],[1,0],[1,1]]){
        const projected=project(card.matrix,u,v),ww=card.matrix[6]*u+card.matrix[7]*v+1;
        vertices.push(((projected[0]-bounds.x)/bounds.w*2-1)*ww,(1-(projected[1]-bounds.y)/bounds.h*2)*ww,ww,u,v);
      }
      gl.bufferSubData(gl.ARRAY_BUFFER,0,new Float32Array(vertices));gl.uniform2f(uniform.uCardSize,card.w,card.h);gl.uniform3f(uniform.uNormal,...card.normal);gl.uniform1f(uniform.uRadius,card.radius);gl.uniform1f(uniform.uOpacity,card.opacity);gl.drawArrays(gl.TRIANGLES,0,6);
    }
    if(now-lastLight>45){
      const angle=p.azimuth*Math.PI/180,px=50+Math.cos(angle)*22+(pointer[0]/innerWidth-.5)*p.parallax*30,py=45+Math.sin(angle)*22+(pointer[1]/innerHeight-.5)*p.parallax*30;
      const style=document.documentElement.style;style.setProperty('--light-pos-x',px.toFixed(2)+'%');style.setProperty('--light-pos-y',py.toFixed(2)+'%');
      style.setProperty('--avatar-light',String(Math.max(.84,Math.min(1.13,.90+p.intensity*.08+px*.0006))));lastLight=now;
    }
    document.body.dataset.paused=paused?'true':'false';
  }
  window.JadeDeckFinish={init,register,frame,homography,project,changed(){until=performance.now()+760;},state:()=>({ready,error,time:lastFrame?.[1]||0,pointer:lastFrame?.[2]||[],cards:cards.map(c=>({id:c.id,stack:c.stack,corners:c.p})),bounds,reads}),screenshot(){if(!ready||!lastFrame)return null;frame(...lastFrame);return canvas.toDataURL('image/png');},vertexSource:VERTEX,fragmentSource:()=>source||shaderSource()};
})();
