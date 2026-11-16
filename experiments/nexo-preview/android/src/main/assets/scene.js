/* Approved Nexo 3D carousel, retained geometry and motion; DOM content follows its projected face. */

(()=>{
'use strict';
const root=document.getElementById('nexo-app'),canvas=root.querySelector('canvas'),backButton=root.querySelector('.nexo-back'),status=root.querySelector('.nexo-state'),buttons=[...root.querySelectorAll('[data-card]')],mediaReduced=matchMedia('(prefers-reduced-motion: reduce)'),reduced={get matches(){return mediaReduced.matches||NexoApp.store.get().preferences.reducedMotion||!!NexoApp.reducedMotion?.();},addEventListener:(...args)=>mediaReduced.addEventListener(...args)};
const names=['Jogar','Mods','Pacotes','Shaders','Skins'],sub=['Seu próximo mundo','Sua experiência','Camadas e texturas','Luz e atmosfera','Sua identidade'],N=names.length,TAU=Math.PI*2,STEP=TAU/N,W=3.6,H=2.35;
const gl=canvas.getContext('webgl',{alpha:false,antialias:true,powerPreference:'low-power'});
if(!gl){root.querySelector('.nexo-error').textContent='O estudo de movimento precisa de WebGL neste visualizador.';root.querySelector('.nexo-error').hidden=false;return;}
const I=()=>new Float32Array([1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1]);
function mul(a,b){const c=new Float32Array(16);for(let i=0;i<4;i++)for(let j=0;j<4;j++)for(let k=0;k<4;k++)c[i*4+j]+=a[k*4+j]*b[i*4+k];return c;}
function trans(x,y,z){const m=I();m[12]=x;m[13]=y;m[14]=z;return m;}
function scale(x,y=x,z=x){const m=I();m[0]=x;m[5]=y;m[10]=z;return m;}
function ry(a){const m=I(),c=Math.cos(a),s=Math.sin(a);m[0]=c;m[2]=-s;m[8]=s;m[10]=c;return m;}
function rx(a){const m=I(),c=Math.cos(a),s=Math.sin(a);m[5]=c;m[6]=s;m[9]=-s;m[10]=c;return m;}
function rz(a){const m=I(),c=Math.cos(a),s=Math.sin(a);m[0]=c;m[1]=s;m[4]=-s;m[5]=c;return m;}
function normal(m){const a=m[0],b=m[4],c=m[8],d=m[1],e=m[5],f=m[9],g=m[2],h=m[6],i=m[10],det=a*(e*i-f*h)-b*(d*i-f*g)+c*(d*h-e*g);return new Float32Array([(e*i-f*h)/det,(b*i-c*h)/-det,(b*f-c*e)/det,(d*i-f*g)/-det,(a*i-c*g)/det,(a*f-c*d)/-det,(d*h-e*g)/det,(a*h-b*g)/-det,(a*e-b*d)/det]);}
function perspective(aspect,eye){const f=1/Math.tan(42*Math.PI/360),near=.1,far=50,m=new Float32Array(16);m[0]=f/aspect;m[5]=f;m[10]=(far+near)/(near-far);m[11]=-1;m[14]=2*far*near/(near-far);return mul(m,trans(0,-.08,-eye));}
function program(vs,fs){function sh(type,text){const s=gl.createShader(type);gl.shaderSource(s,text);gl.compileShader(s);if(!gl.getShaderParameter(s,gl.COMPILE_STATUS))throw Error(gl.getShaderInfoLog(s));return s;}const p=gl.createProgram(),v=sh(gl.VERTEX_SHADER,vs),f=sh(gl.FRAGMENT_SHADER,fs);gl.attachShader(p,v);gl.attachShader(p,f);gl.linkProgram(p);if(!gl.getProgramParameter(p,gl.LINK_STATUS))throw Error(gl.getProgramInfoLog(p));gl.deleteShader(v);gl.deleteShader(f);return p;}
const vs=`attribute vec3 aP,aN;attribute vec2 aUV;
uniform mat4 uModel,uVP,uMirror,uLightVP;uniform mat3 uNormal;
varying vec3 vN,vP;varying vec2 vUV;varying vec4 vShadow,vView;
void main(){vec4 p=uModel*vec4(aP,1.);vP=p.xyz;vN=normalize(uNormal*aN);vUV=aUV;vShadow=uLightVP*p;vView=uVP*p;gl_Position=uVP*uMirror*p;}`;
// Original stylized material model: shared analytic softboxes, Fresnel, a coat
// lobe, microtexture, actual scene shadow map and planar reflected geometry.
const fs=`precision highp float;
varying vec3 vN,vP;varying vec2 vUV;varying vec4 vShadow,vView;
uniform vec3 uColor,uEye,uCool,uWarm;
uniform float uMaterial;
uniform float uAlpha,uTex,uBlend,uMetal,uWarmth,uTime,uSurface;
uniform sampler2D uPoster,uModule,uGrain,uShadowMap,uReflection;
uniform vec2 uShadowTexel,uReflectionTexel;
uniform float uLighting,uReflectPass,uReflectGain;uniform vec4 uClipPlane;
float visibleLight(vec3 n,vec3 l){
  if(uLighting<.5)return 1.;
  vec3 q=vShadow.xyz/vShadow.w*.5+.5;
  if(q.x<0.||q.x>1.||q.y<0.||q.y>1.||q.z<0.||q.z>1.)return 1.;
  float bias=max(.0013,.0022*(1.-max(dot(n,l),0.))),sum=0.;
  for(int k=0;k<4;k++){
    vec2 offset=vec2(mod(float(k),2.)-.5,floor(float(k)/2.)-.5)*uShadowTexel*2.5;
    vec4 d=texture2D(uShadowMap,q.xy+offset);
    sum+=mix(1.,step(q.z-bias,d.r+d.g/255.),d.b);
  }
  return mix(.28,1.,sum*.25);
}
vec3 environment(vec3 r,float rough){
  float bend=.10*sin(r.x*3.4+uTime*.17);
  float strip=exp(-pow((r.y-.07-bend)/(.045+rough*.22),2.));
  float high=exp(-pow((r.y-.79)/(.09+rough*.12),2.));
  float warm=exp(-pow((r.x-.64)/(.16+rough*.2),2.))*exp(-pow((r.y+.12)/.7,2.));
  return mix(vec3(.055,.085,.12),uCool*.14,.65)+uCool*.95*strip+vec3(.44,.50,.54)*high
    +uWarm*.68*warm*(.18+uWarmth*.95);
}
void main(){
  if(uReflectPass>.5&&dot(vec4(vP,1.),uClipPlane)<.005)discard;
  vec3 micro=texture2D(uGrain,vUV*vec2(7.,4.)).rgb;
  vec3 n=normalize(vN+vec3((micro.rg-.5)*.019,0.));
  vec3 v=normalize(uEye-vP),l=normalize(vec3(-.7,.9,1.6)),h=normalize(l+v);
  float nv=max(dot(n,v),0.),ndl=max(dot(n,l),0.);
  float visibility=visibleLight(n,l);
  float rough=clamp(mix(.28,.13,uMetal)+(micro.b-.5)*.035+uMaterial,.045,.75);
  vec3 base=uColor*(.98+micro.b*.04);
  // A shallow convex polish makes the broad front face catch the same room
  // reflection as its bevel, without changing the approved card geometry.
  vec3 coatN=normalize(n+vec3((vUV-.5)*vec2(.13,.085)*uSurface,0.));
  vec3 reflection=environment(reflect(-v,coatN),rough);
  float f=pow(1.-nv,5.);
  vec3 f0=mix(vec3(.055),base*.76,uMetal),fresnel=f0+(1.-f0)*f;
  float spec=pow(max(dot(n,h),0.),mix(150.,24.,rough));
  vec3 color=base*(.34+.64*ndl*visibility)*(1.-uMetal*.47)+reflection*fresnel*(.72+uMetal*.65);
  color+=vec3(.68,.84,.91)*spec*(.12+.30*uMetal)*visibility;
  color+=environment(reflect(-v,coatN),.07)*(.025+.23*f)*(1.-uMetal*.5);
  if(uSurface>.5){
    vec2 q=abs(vUV-.5)-vec2(.447,.419);
    float edge=length(max(q,0.))+min(max(q.x,q.y),0.)-.035;
    float etch=1.-smoothstep(.001,.0035,abs(edge));
    color+=vec3(.14,.25,.28)*etch*.32;
    float sweep=exp(-pow((vUV.y-.75-.055*sin(vUV.x*4.+uTime*.17))/.18,2.));
    color+=vec3(.012,.035,.043)*sweep;
  }
  float reveal=smoothstep(.18,.92,uBlend);
  vec4 t=mix(texture2D(uPoster,vUV),texture2D(uModule,vUV),reveal);
  color=mix(color,t.rgb,t.a*uTex);
  color*=.72+.28*visibility;
  if(uReflectGain>0.){
    vec2 uv=vView.xy/vView.w*.5+.5,spread=uReflectionTexel*3.;
    vec4 image=texture2D(uReflection,uv)*.4;
    image+=texture2D(uReflection,uv+vec2(spread.x,0.))*.15;
    image+=texture2D(uReflection,uv-vec2(spread.x,0.))*.15;
    image+=texture2D(uReflection,uv+vec2(0.,spread.y))*.15;
    image+=texture2D(uReflection,uv-vec2(0.,spread.y))*.15;
    color+=image.rgb*image.a*uReflectGain;
  }
  gl_FragColor=vec4(color,uAlpha);
}`;
let p;try{p=program(vs,fs);}catch(e){root.querySelector('.nexo-error').textContent='Não foi possível iniciar o estudo de movimento.';root.querySelector('.nexo-error').hidden=false;return;}
const U={};for(const n of ['uModel','uVP','uMirror','uLightVP','uNormal','uColor','uEye','uAlpha','uTex','uBlend','uMetal','uWarmth','uTime','uSurface','uPoster','uModule','uGrain','uShadowMap','uReflection','uShadowTexel','uReflectionTexel','uLighting','uReflectPass','uReflectGain','uClipPlane','uCool','uWarm','uMaterial'])U[n]=gl.getUniformLocation(p,n);
const A={p:gl.getAttribLocation(p,'aP'),n:gl.getAttribLocation(p,'aN'),uv:gl.getAttribLocation(p,'aUV')};
const bgvs='attribute vec2 aP;varying vec2 vUV;void main(){vUV=(aP+1.)*.5;gl_Position=vec4(aP,.999,1.);}',bgfs=`precision mediump float;
varying vec2 vUV;uniform float uOpen,uTime,uAspect,uWarmth;uniform vec3 uColdTint,uWarmTint,uBase;
void main(){
  vec2 uv=vUV,p=(uv-vec2(.44,.46))*vec2(min(uAspect,1.6),.8);
  float haze=exp(-dot(p,p)*5.);
  vec3 c=uBase+uColdTint*.06*haze;
  float drift=uTime*.16;
  for(int k=0;k<3;k++){
    float layer=float(k),phase=layer*.59;
    float line=.25+layer*.061+.10*sin(uv.x*4.7+drift+phase)
      +.035*sin(uv.x*9.4-drift*.63+phase);
    float d=uv.y-line;
    float veil=exp(-pow((d-.014)/(.032+layer*.018),2.));
    float lip=exp(-pow(d/(.0035+layer*.001),2.));
    float fold=exp(-pow((d+.009)/.014,2.));
    vec3 cold=uColdTint*.52,warm=uWarmTint*.42;
    vec3 tint=mix(cold,warm,smoothstep(.48,.98,uv.x)*uWarmth*.8);
    float fade=(.55+.45*sin(uv.x*2.7+phase))*(1.-layer*.19);
    c+=tint*(veil*.29+lip*.44+fold*.09)*fade*(1.-uOpen*.45);
  }
  float glow=exp(-dot((uv-vec2(.81,.66))*vec2(1.5,2.),(uv-vec2(.81,.66))*vec2(1.5,2.))*5.);
  c+=uWarmTint*.055*glow*uWarmth;
  c*=.80+.20*smoothstep(0.,.16,uv.y)*(1.-smoothstep(.80,1.,uv.y));
  gl_FragColor=vec4(c,1.);
}`;
const bgp=program(bgvs,bgfs);
const bgb=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,bgb);gl.bufferData(gl.ARRAY_BUFFER,new Float32Array([-1,-1,3,-1,-1,3]),gl.STATIC_DRAW);const bgA=gl.getAttribLocation(bgp,'aP'),bgU={};for(const n of ['uOpen','uTime','uAspect','uWarmth','uColdTint','uWarmTint','uBase'])bgU[n]=gl.getUniformLocation(bgp,n);
const grain=gl.createTexture(),grainData=new Uint8Array(128*128*4);let grainSeed=198;
for(let i=0;i<grainData.length;i+=4){grainSeed=(Math.imul(grainSeed,1664525)+1013904223)>>>0;grainData[i]=grainSeed>>>24;grainData[i+1]=(grainSeed>>>16)&255;grainData[i+2]=112+((grainSeed>>>8)&31);grainData[i+3]=255;}
gl.bindTexture(gl.TEXTURE_2D,grain);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,128,128,0,gl.RGBA,gl.UNSIGNED_BYTE,grainData);gl.generateMipmap(gl.TEXTURE_2D);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR_MIPMAP_LINEAR);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.LINEAR);
let meshNumber=0;function mesh(data){const b=gl.createBuffer(),bounds=[Infinity,Infinity,Infinity,-Infinity,-Infinity,-Infinity];for(let i=0;i<data.length;i+=8)for(let j=0;j<3;j++){bounds[j]=Math.min(bounds[j],data[i+j]);bounds[j+3]=Math.max(bounds[j+3],data[i+j]);}gl.bindBuffer(gl.ARRAY_BUFFER,b);gl.bufferData(gl.ARRAY_BUFFER,new Float32Array(data),gl.STATIC_DRAW);return {buffer:b,count:data.length/8,bounds,id:meshNumber++};}
function rounded(w,h,d,r=.1,bevel=.025){const rings=[],pts=8,body=[],front=[];for(const [z,inset] of [[-d/2,bevel],[-d/2+bevel,0],[d/2-bevel,0],[d/2,bevel]]){const ring=[];for(let k=0;k<4;k++){const sx=k===0||k===3?1:-1,sy=k<2?1:-1,cx=sx*(w/2-r),cy=sy*(h/2-r);for(let j=0;j<=pts;j++){const a=(k*Math.PI/2)+(j/pts)*Math.PI/2;ring.push([cx+(r-inset)*Math.cos(a),cy+(r-inset)*Math.sin(a),z,Math.cos(a),Math.sin(a)]);}}rings.push(ring);}
 function vertex(arr,v,n){arr.push(v[0],v[1],v[2],...n,v[0]/w+.5,v[1]/h+.5);}
 const total=rings[0].length;
 for(let ring=0;ring<3;ring++)for(let j=0;j<total;j++){const k=(j+1)%total,a=rings[ring][j],b=rings[ring][k],c=rings[ring+1][k],d1=rings[ring+1][j],z=ring===0?-.65:ring===2?.65:0;for(const v of [a,b,c,a,c,d1]){const length=Math.hypot(v[3],v[4],z);vertex(body,v,[v[3]/length,v[4]/length,z/length]);}}
 for(let j=0;j<total;j++){const a=rings[3][j],b=rings[3][(j+1)%total];for(const v of [[0,0,d/2],a,b])vertex(front,v,[0,0,1]);const x=rings[0][j],y=rings[0][(j+1)%total];for(const v of [[0,0,-d/2],y,x])vertex(body,v,[0,0,-1]);}
 return {body:mesh(body),front:mesh(front)};}
function prism(points,depth){
 const data=[],bevel=.045,cx=points.reduce((s,p)=>s+p[0],0)/points.length,cy=points.reduce((s,p)=>s+p[1],0)/points.length;
 const inset=points.map(p=>[cx+(p[0]-cx)*.83,cy+(p[1]-cy)*.83]);
 function v(p,n){data.push(...p,...n,p[0]+.5,p[1]+.5);}
 for(const side of [-1,1])for(let i=1;i<points.length-1;i++)for(const p0 of [inset[0],inset[i],inset[i+1]])v([...p0,side*depth/2],[0,0,side]);
 for(let i=0;i<points.length;i++){
  const j=(i+1)%points.length,a=points[i],b=points[j],dx=b[0]-a[0],dy=b[1]-a[1],length=Math.hypot(dx,dy),n=[dy/length,-dx/length,0];
  for(const q of [[...a,-depth/2+bevel],[...b,-depth/2+bevel],[...b,depth/2-bevel],[...a,-depth/2+bevel],[...b,depth/2-bevel],[...a,depth/2-bevel]])v(q,n);
  for(const side of [-1,1]){
   const a0=[...a,side*(depth/2-bevel)],b0=[...b,side*(depth/2-bevel)],a1=[...inset[i],side*depth/2],b1=[...inset[j],side*depth/2];
   for(const q of [a0,b0,b1,a0,b1,a1])v(q,[n[0]*.6,n[1]*.6,side*.8]);
  }
 }
 return mesh(data);
}
const rim=rounded(W+.045,H+.045,.15,.075,.018),panel=rounded(W,H,.16,.065,.018),block=rounded(.53,.53,.34,.10,.03),plate=rounded(.88,.63,.10,.12,.025),head=rounded(.40,.43,.35,.11,.03),shoulders=rounded(.83,.28,.34,.12,.03),triangle=prism([[-.34,-.44],[.48,0],[-.34,.44]],.24),diamond=prism([[0,.50],[-.37,0],[0,-.50],[.37,0]],.28);
// Original voxel silhouettes, with the existing real-time console materials.
const voxelMesh=rounded(1,1,1,.018,.012),crystalMesh=prism([[0,.62],[-.22,.25],[-.16,-.38],[.16,-.38],[.22,.25]],.25);
function voxel(parent,x,y,z,w,h,d,color,alpha,metal=.7){object(voxelMesh,mul(parent,mul(trans(x,y,z),scale(w,h,d))),color,alpha,metal);}
function minecraftIcon(i,m,cool,warm,alpha){
 const edge=[.30,.43,.47],dark=[.12,.19,.22],light=[.77,.86,.86];
 if(i===0){const pick=mul(m,rz(-.67));voxel(pick,0,-.12,0,.13,1.02,.14,warm,alpha,.48);voxel(pick,0,.34,0,.88,.17,.25,cool,alpha,.94);voxel(pick,-.35,.18,0,.18,.28,.25,cool,alpha,.94);voxel(pick,.35,.23,0,.16,.19,.25,cool,alpha,.94);voxel(pick,0,-.52,.01,.16,.14,.17,edge,alpha,.85);}
 if(i===1){voxel(m,0,0,0,.82,.69,.65,warm,alpha,.48);voxel(m,0,.37,.015,.92,.14,.74,cool,alpha,.86);for(const x of[-.25,0,.25])for(const z of[-.22,.02,.26])voxel(m,x,.448,z,.19,.012,.18,dark,alpha,.55);for(const x of[-.32,.32])voxel(m,x,-.14,.34,.09,.46,.03,edge,alpha,.8);voxel(m,0,-.08,.342,.30,.12,.025,cool,alpha,.7);voxel(m,0,-.25,.344,.12,.22,.028,cool,alpha,.7);}
 if(i===2){for(let k=0;k<3;k++){const layer=mul(m,mul(trans((k-1)*.085,(k-1)*.25,(k-1)*-.04),ry(-.12)));voxel(layer,0,0,0,.81,.22,.62,k===2?warm:cool,alpha,.8);voxel(layer,0,-.07,.315,.72,.035,.018,edge,alpha,.6);}voxel(m,.085,.405,-.04,.68,.08,.52,edge,alpha,.7);}
 if(i===3){for(const [x,y,z,k]of[[-.28,-.10,0,.66],[.04,.05,0,1],[.31,-.18,.08,.55]])drawMesh(crystalMesh,mul(m,mul(trans(x,y,z),mul(rz(-x*.7),scale(k)))),x>0?warm:cool,alpha,0,.92);voxel(m,0,-.44,0,.72,.12,.48,edge,alpha,.8);}
 if(i===4){voxel(m,0,.18,0,.64,.64,.57,cool,alpha,.8);voxel(m,0,-.35,-.01,.9,.28,.43,warm,alpha,.65);voxel(m,0,.43,.30,.64,.14,.035,edge,alpha,.6);for(const x of[-.16,.16]){voxel(m,x,.18,.302,.11,.09,.025,dark,alpha,.7);voxel(m,x-.02,.19,.322,.035,.04,.012,light,alpha,.8);}voxel(m,0,-.02,.304,.18,.055,.03,edge,alpha,.6);}
}
function utilityIcon(kind,m,cool,warm,alpha){
 const dark=[.12,.19,.22];
 if(kind==='files'){voxel(m,-.2,.25,0,.40,.20,.32,cool,alpha,.7);voxel(m,0,-.03,-.1,.91,.65,.27,cool,alpha,.65);voxel(m,0,-.12,.14,.96,.47,.13,warm,alpha,.72);voxel(m,0,.14,.03,.74,.06,.30,dark,alpha,.6);}
 if(kind==='settings'){for(let k=0;k<8;k++){const a=k*Math.PI/4,base=mul(m,mul(rz(a),trans(0,.41,0)));voxel(base,0,0,0,.22,.19,.27,cool,alpha,.85);}voxel(m,0,0,0,.64,.64,.24,cool,alpha,.83);voxel(m,0,0,.14,.31,.31,.06,dark,alpha,.55);voxel(m,0,0,.19,.13,.13,.07,warm,alpha,.8);}
}
function texture(i,internal){const c=document.createElement('canvas');c.width=1024;c.height=668;const x=c.getContext('2d');x.clearRect(0,0,c.width,c.height);
 if(!internal){x.fillStyle='#9ebbd4';x.font='500 30px system-ui';x.fillText(String(i+1).padStart(2,'0')+(NexoApp.productOnly?' / NEXO':' / HORIZON'),65,85);x.fillStyle='#e3ebf2';const textScale=NexoApp.store.get().preferences.textScale;x.font='400 '+(86*textScale)+'px Michroma, system-ui';x.fillText(names[i],65,462+86*textScale*.86);x.font='400 '+(35*textScale)+'px system-ui';x.fillStyle='#b7cbdc';const p=NexoApp.profile();x.fillText(i===0?p.name+' · '+p.version:sub[i],65,564+(textScale-1)*80+35*textScale*.82);}
 const t=gl.createTexture();gl.bindTexture(gl.TEXTURE_2D,t);gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL,true);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,gl.RGBA,gl.UNSIGNED_BYTE,c);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.LINEAR);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_S,gl.CLAMP_TO_EDGE);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_T,gl.CLAMP_TO_EDGE);return t;}
const torso=rounded(.56,.72,.30,.08,.025),limb=rounded(.18,.68,.24,.07,.025),leg=rounded(.23,.58,.25,.06,.02);
const posters=names.map((_,i)=>texture(i,false)),internals=names.map((_,i)=>texture(i,true));
let position=NexoApp.store.get().selected,velocity=0,target=position,opening=0,openVelocity=0,openTarget=0,pulseStart=-10000,pulseIndex=-1,pulsePending=false,drag=null,last=0,raf=0,vp=I(),eye=8,hitFaces=[],settled=true,launching=0,launchVelocity=0,launchTarget=0,visible=!document.hidden,forcePaint=true,lastPaint=0,tapGuard=null,idleTimer=0,ambientTime=0,renderCount=0,outsidePress=null,collecting=false,drawQueue=[],drawInfo=null,frontModel=null,blurAmount=0;
const mod=(n,m)=>(n%m+m)%m,clamp=(n,a,b)=>Math.max(a,Math.min(b,n)),mix=(a,b,t)=>a+(b-a)*t;
function spring(x,v,to,dt,w=17){const e=Math.exp(-w*dt),d=x-to,c=v+w*d;return [to+(d+c*dt)*e,(v-w*c*dt)*e];}
function updateUI(){const ix=mod(Math.round(target),N),busy=!!openTarget||!!launchTarget;buttons.forEach((b,i)=>{b.setAttribute('aria-pressed',String(i===ix));b.setAttribute('aria-label',i===ix?(i===0?'Jogar Minecraft':'Abrir '+names[i]):'Selecionar '+names[i]);b.disabled=busy;});const profile=NexoApp.profile();status.textContent=launchTarget?'Preparando sessão':openTarget?names[ix]+(window.HorizonHost?'':' · prévia local'):profile.name+' · '+profile.version;NexoApp.selectionChanged(ix,!!openTarget);}
function ambientEnabled(){return NexoApp.store.get().preferences.ambientMotion&&!reduced.matches;}
function wake(){if(!visible)return;if(idleTimer){clearTimeout(idleTimer);idleTimer=0;}if(!raf)raf=requestAnimationFrame(frame);}
function choose(index){if(openTarget||launchTarget||opening>.001)return;const current=mod(Math.round(target),N);let d=mod(index-current+N/2,N)-N/2;target=Math.round(target)+d;pulsePending=true;pulseStart=-10000;settled=false;updateUI();wake();}
function activate(index){if(root.classList.contains('nx-compatible'))return;if(NexoApp.beforeOpen?.()===false)return;if(openTarget||launchTarget||opening>.001)return;const current=mod(Math.round(target),N);if(index!==current||Math.abs(position-target)>.10){choose(index);return;}pulseStart=-10000;pulsePending=false;if(index===0){NexoApp.play();return;}openTarget=1;NexoApp.sound('open');updateUI();wake();}
function closeSection(force=false){if(!openTarget||launchTarget)return;if(force!==true&&NexoApp.beforeClose?.()===false)return;NexoApp.sound('back');openTarget=0;launchTarget=0;forcePaint=true;updateUI();buttons[mod(Math.round(target),N)].focus({preventScroll:true});wake();}
function drawMesh(m,model,color,alpha,tex=0,metal=.1,surface=0){if(collecting){const key=drawInfo.page+':'+drawInfo.counter++ +':'+m.id;drawQueue.push({m,model,color,alpha,tex,metal,surface,...drawInfo,key});return;}gl.uniform1f(U.uSurface,surface);gl.uniformMatrix4fv(U.uModel,false,model);gl.uniformMatrix3fv(U.uNormal,false,normal(model));gl.uniform3fv(U.uColor,color);gl.uniform1f(U.uAlpha,alpha);gl.uniform1f(U.uTex,tex);gl.uniform1f(U.uMetal,metal);gl.bindBuffer(gl.ARRAY_BUFFER,m.buffer);gl.enableVertexAttribArray(A.p);gl.vertexAttribPointer(A.p,3,gl.FLOAT,false,32,0);gl.enableVertexAttribArray(A.n);gl.vertexAttribPointer(A.n,3,gl.FLOAT,false,32,12);gl.enableVertexAttribArray(A.uv);gl.vertexAttribPointer(A.uv,2,gl.FLOAT,false,32,24);gl.drawArrays(gl.TRIANGLES,0,m.count);}
function object(m,model,color,alpha=1,metal=.1){drawMesh(m.body,model,color,alpha,0,metal);drawMesh(m.front,model,color,alpha,0,metal);}
function project(m,x,y,z){const p=mul(vp,m),w=p[3]*x+p[7]*y+p[11]*z+p[15];return [(p[0]*x+p[4]*y+p[8]*z+p[12])/w,(p[1]*x+p[5]*y+p[9]*z+p[13])/w];}
// Original scene passes. Both maps render the same mesh commands as the visible
// scene; the reflection is clipped to the front side of the selected panel.
const shadowVS='attribute vec3 aP;uniform mat4 uModel,uLightVP;void main(){gl_Position=uLightVP*uModel*vec4(aP,1.);}';
const shadowFS='precision highp float;uniform float uAlpha;void main(){vec2 d=fract(vec2(1.,255.)*gl_FragCoord.z);d.x-=d.y/255.;gl_FragColor=vec4(d,uAlpha,1.);}';
const shadowProgram=program(shadowVS,shadowFS),shadowA=gl.getAttribLocation(shadowProgram,'aP'),shadowU={};
for(const name of ['uModel','uLightVP','uAlpha'])shadowU[name]=gl.getUniformLocation(shadowProgram,name);
function lightView(){
 const normalize=a=>{const d=Math.hypot(...a);return a.map(v=>v/d);},cross=(a,b)=>[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]],dot=(a,b)=>a.reduce((s,v,i)=>s+v*b[i],0);
 const e=[-7,9,16],z=normalize(e),x=normalize(cross([0,1,0],z)),y=cross(z,x),view=new Float32Array([x[0],y[0],z[0],0,x[1],y[1],z[1],0,x[2],y[2],z[2],0,-dot(x,e),-dot(y,e),-dot(z,e),1]),ortho=I();
 ortho[0]=1/9;ortho[5]=1/7;ortho[10]=-2/35;ortho[14]=-1.01;return mul(ortho,view);
}
const lightVP=lightView(),emptyTexture=gl.createTexture();gl.activeTexture(gl.TEXTURE5);gl.bindTexture(gl.TEXTURE_2D,emptyTexture);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,1,1,0,gl.RGBA,gl.UNSIGNED_BYTE,new Uint8Array(4));gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.NEAREST);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.NEAREST);
let shadowTarget=null,reflectionTarget=null,lightingStats={enabled:false,shadowDraws:0,reflectionDraws:0},shadowKey=null,reflectionKey=null,reflectionTime=-1;
function mapTarget(width,height,linear=false){
 const texture=gl.createTexture(),buffer=gl.createFramebuffer(),depth=gl.createRenderbuffer();gl.activeTexture(gl.TEXTURE5);gl.bindTexture(gl.TEXTURE_2D,texture);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,width,height,0,gl.RGBA,gl.UNSIGNED_BYTE,null);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,linear?gl.LINEAR:gl.NEAREST);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,linear?gl.LINEAR:gl.NEAREST);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_S,gl.CLAMP_TO_EDGE);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_T,gl.CLAMP_TO_EDGE);gl.bindRenderbuffer(gl.RENDERBUFFER,depth);gl.renderbufferStorage(gl.RENDERBUFFER,gl.DEPTH_COMPONENT16,width,height);gl.bindFramebuffer(gl.FRAMEBUFFER,buffer);gl.framebufferTexture2D(gl.FRAMEBUFFER,gl.COLOR_ATTACHMENT0,gl.TEXTURE_2D,texture,0);gl.framebufferRenderbuffer(gl.FRAMEBUFFER,gl.DEPTH_ATTACHMENT,gl.RENDERBUFFER,depth);
 const ready=gl.checkFramebufferStatus(gl.FRAMEBUFFER)===gl.FRAMEBUFFER_COMPLETE;gl.bindFramebuffer(gl.FRAMEBUFFER,null);return {width,height,texture,buffer,depth,ready};
}
function resizeTarget(old,width,height,linear){if(old&&old.width===width&&old.height===height)return old;if(old){gl.deleteTexture(old.texture);gl.deleteRenderbuffer(old.depth);gl.deleteFramebuffer(old.buffer);}return mapTarget(width,height,linear);}
function planeMirror(model){
 const length=Math.hypot(model[8],model[9],model[10]),n=[model[8]/length,model[9]/length,model[10]/length],point=[model[12]+model[8]*.08,model[13]+model[9]*.08,model[14]+model[10]*.08],d=-n.reduce((s,v,i)=>s+v*point[i],0),matrix=I();
 for(let col=0;col<3;col++)for(let row=0;row<3;row++)matrix[col*4+row]-=2*n[row]*n[col];for(let row=0;row<3;row++)matrix[12+row]=-2*d*n[row];return {matrix,plane:[...n,d]};
}
function issue(c,gain=0){gl.activeTexture(gl.TEXTURE0);gl.bindTexture(gl.TEXTURE_2D,c.poster);gl.activeTexture(gl.TEXTURE1);gl.bindTexture(gl.TEXTURE_2D,c.internal);gl.uniform1f(U.uBlend,c.blend);gl.uniform1f(U.uReflectGain,gain);drawMesh(c.m,c.model,c.color,c.alpha,c.tex,c.metal,c.surface);}
function paintScene(commands,selected,viewport,width,height){
 const atmosphere=NexoApp.atmosphere?.()||{cool:[.62,.76,.8],warm:[.64,.49,.29]};
 const enabled=NexoApp.store.get().preferences.sceneLighting&&NexoApp.nativeLighting?.()!==false&&frontModel;
 lightingStats={enabled:false,shadowDraws:0,reflectionDraws:0};
 if(enabled){const size=NexoApp.store.get().preferences.renderScale>.8?768:384;shadowTarget=resizeTarget(shadowTarget,size,size,false);const ratio=Math.min(.42,512/width,512/height);reflectionTarget=resizeTarget(reflectionTarget,Math.max(64,Math.ceil(width*ratio/16)*16),Math.max(64,Math.ceil(height*ratio/16)*16),true);}
 const ready=!!enabled&&shadowTarget.ready&&reflectionTarget.ready;
 let hash=2166136261;for(const c of commands){for(const value of c.model)hash=Math.imul(hash^Math.round(value*100000),16777619);hash=Math.imul(hash^Math.round(c.alpha*10000),16777619);}
 const key=[hash,width,height,NexoApp.store.get().preferences.renderScale,NexoApp.store.get().preferences.warmth,NexoApp.profile().name,JSON.stringify(NexoApp.themeColors?.()),NexoApp.material?.()].join('|'),updateShadow=ready&&key!==shadowKey,updateReflection=ready&&(key!==reflectionKey||Math.abs(ambientTime-reflectionTime)>1/12);
 lightingStats.shadowReused=ready&&!updateShadow;lightingStats.reflectionReused=ready&&!updateReflection;
 if(updateShadow){
  gl.bindFramebuffer(gl.FRAMEBUFFER,shadowTarget.buffer);gl.viewport(0,0,shadowTarget.width,shadowTarget.height);gl.clearColor(1,1,1,1);gl.clear(gl.COLOR_BUFFER_BIT|gl.DEPTH_BUFFER_BIT);gl.enable(gl.DEPTH_TEST);gl.disable(gl.BLEND);gl.enable(gl.CULL_FACE);gl.cullFace(gl.FRONT);gl.useProgram(shadowProgram);gl.uniformMatrix4fv(shadowU.uLightVP,false,lightVP);
  for(const c of commands){gl.uniformMatrix4fv(shadowU.uModel,false,c.model);gl.uniform1f(shadowU.uAlpha,c.alpha);gl.bindBuffer(gl.ARRAY_BUFFER,c.m.buffer);gl.enableVertexAttribArray(shadowA);gl.vertexAttribPointer(shadowA,3,gl.FLOAT,false,32,0);gl.drawArrays(gl.TRIANGLES,0,c.m.count);lightingStats.shadowDraws++;}shadowKey=key;
 }
 gl.disable(gl.CULL_FACE);gl.useProgram(p);gl.uniform3fv(U.uCool,atmosphere.cool);gl.uniform3fv(U.uWarm,atmosphere.warm);gl.uniform1f(U.uMaterial,NexoApp.material?.()==='polished'?-.09:NexoApp.material?.()==='soft'?.25:0);gl.uniformMatrix4fv(U.uLightVP,false,lightVP);gl.uniform1f(U.uLighting,ready?1:0);gl.uniform1i(U.uShadowMap,3);gl.uniform1i(U.uReflection,4);gl.activeTexture(gl.TEXTURE3);gl.bindTexture(gl.TEXTURE_2D,ready?shadowTarget.texture:emptyTexture);gl.uniform2f(U.uShadowTexel,ready?1/shadowTarget.width:1,ready?1/shadowTarget.height:1);gl.activeTexture(gl.TEXTURE4);gl.bindTexture(gl.TEXTURE_2D,emptyTexture);
 if(updateReflection){
  const mirror=planeMirror(frontModel),m=mirror.matrix;gl.bindFramebuffer(gl.FRAMEBUFFER,reflectionTarget.buffer);gl.viewport(0,0,reflectionTarget.width,reflectionTarget.height);gl.clearColor(0,0,0,0);gl.clear(gl.COLOR_BUFFER_BIT|gl.DEPTH_BUFFER_BIT);gl.enable(gl.BLEND);gl.blendFunc(gl.SRC_ALPHA,gl.ONE_MINUS_SRC_ALPHA);gl.uniformMatrix4fv(U.uMirror,false,m);gl.uniform1f(U.uReflectPass,1);gl.uniform4fv(U.uClipPlane,mirror.plane);gl.uniform3f(U.uEye,m[4]*.08+m[8]*eye+m[12],m[5]*.08+m[9]*eye+m[13],m[6]*.08+m[10]*eye+m[14]);
  for(const c of commands){if(c.page===selected&&[panel.body,panel.front,rim.body,rim.front].includes(c.m))continue;issue(c);lightingStats.reflectionDraws++;}reflectionKey=key;reflectionTime=ambientTime;
 }
 if(ready){
  gl.activeTexture(gl.TEXTURE4);gl.bindTexture(gl.TEXTURE_2D,reflectionTarget.texture);gl.uniform2f(U.uReflectionTexel,1/reflectionTarget.width,1/reflectionTarget.height);lightingStats.enabled=true;
 }
 gl.bindFramebuffer(gl.FRAMEBUFFER,null);gl.viewport(...viewport);gl.useProgram(p);gl.enable(gl.BLEND);gl.blendFunc(gl.SRC_ALPHA,gl.ONE_MINUS_SRC_ALPHA);gl.uniformMatrix4fv(U.uMirror,false,I());gl.uniform1f(U.uReflectPass,0);gl.uniform3f(U.uEye,0,.08,eye);gl.uniform3fv(U.uCool,atmosphere.cool);gl.uniform3fv(U.uWarm,atmosphere.warm);gl.uniform1f(U.uMaterial,NexoApp.material?.()==='polished'?-.09:NexoApp.material?.()==='soft'?.25:0);
 for(const c of commands)issue(c,ready&&c.page===selected&&c.surface>.5?.3:0);
}
function inspectLayout(){
 const selected=mod(Math.round(target),N),c=canvas.getBoundingClientRect(),rect=NexoApp.headerIconBox?.(),style=getComputedStyle(root),safeTop=parseFloat(style.getPropertyValue('--safe-top'))||0,safeBottom=parseFloat(style.getPropertyValue('--safe-bottom'))||0;
 const homeTop=Math.max(0,root.querySelector('.app-header').getBoundingClientRect().bottom-c.top+8),homeBottom=root.querySelector('.app-footer').getBoundingClientRect().top-c.top-8,expanded=1-Math.pow(1-opening,2),top=mix(homeTop,safeTop+8,expanded),bottom=mix(homeBottom,c.height-safeBottom-8,expanded),h=Math.max(100,bottom-top),points=[];
 for(const item of drawQueue)if(item.page===selected&&item.headerIcon)for(const x of[item.m.bounds[0],item.m.bounds[3]])for(const y of[item.m.bounds[1],item.m.bounds[4]])for(const z of[item.m.bounds[2],item.m.bounds[5]]){const p=project(item.model,x,y,z);points.push([(p[0]+1)*c.width/2,top+(1-p[1])*h/2]);}
 const xs=points.map(p=>p[0]),ys=points.map(p=>p[1]);return {slot:rect,icon:points.length?{left:Math.min(...xs),top:Math.min(...ys),right:Math.max(...xs),bottom:Math.max(...ys)}:null};
}
function inspectLighting(){
 const counts={...lightingStats};for(const [name,t] of [['shadow',shadowTarget],['reflection',reflectionTarget]]){if(!t?.ready){counts[name+'Pixels']=0;continue;}gl.bindFramebuffer(gl.FRAMEBUFFER,t.buffer);const pixels=new Uint8Array(t.width*t.height*4);gl.readPixels(0,0,t.width,t.height,gl.RGBA,gl.UNSIGNED_BYTE,pixels);let count=0;for(let i=0;i<pixels.length;i+=4)if(name==='shadow'?pixels[i]<250:pixels[i+3]>0)count++;counts[name+'Pixels']=count;}gl.bindFramebuffer(gl.FRAMEBUFFER,null);counts.glError=gl.getError();return counts;
}
const motion=NexoMotion.create(gl,program,mul);
function render(now){const w=canvas.clientWidth,fullH=canvas.clientHeight,style=getComputedStyle(root),safeTop=parseFloat(style.getPropertyValue('--safe-top'))||0,safeBottom=parseFloat(style.getPropertyValue('--safe-bottom'))||0,safeLeft=parseFloat(style.getPropertyValue('--safe-left'))||0,safeRight=parseFloat(style.getPropertyValue('--safe-right'))||0,homeTop=Math.max(0,root.querySelector('.app-header').getBoundingClientRect().bottom-canvas.getBoundingClientRect().top+8),homeBottom=root.querySelector('.app-footer').getBoundingClientRect().top-canvas.getBoundingClientRect().top-8,expanded=1-Math.pow(1-opening,2),top=mix(homeTop,safeTop+8,expanded),bottom=mix(homeBottom,fullH-safeBottom-8,expanded),h=Math.max(100,bottom-top),intro=NexoIntro.getState(now),sceneReveal=intro.sceneReveal,ratio=Math.min(devicePixelRatio||1,1.6)*NexoApp.store.get().preferences.renderScale;if(canvas.width!==Math.round(w*ratio)||canvas.height!==Math.round(fullH*ratio)){canvas.width=Math.round(w*ratio);canvas.height=Math.round(fullH*ratio);}const aspect=w/h,f=1/Math.tan(42*Math.PI/360);eye=2+Math.max(W*f/(2*aspect*.60),H*f/(2*.74));vp=perspective(aspect,eye);gl.viewport(0,0,canvas.width,canvas.height);gl.disable(gl.DEPTH_TEST);gl.disable(gl.BLEND);gl.clear(gl.COLOR_BUFFER_BIT|gl.DEPTH_BUFFER_BIT);gl.useProgram(bgp);gl.bindBuffer(gl.ARRAY_BUFFER,bgb);gl.enableVertexAttribArray(bgA);gl.vertexAttribPointer(bgA,2,gl.FLOAT,false,8,0);const atmosphere=NexoApp.atmosphere?.()||{cool:[.29,.71,.81],warm:[.92,.53,.2],bg:[.035,.059,.098]};gl.uniform3fv(bgU.uColdTint,atmosphere.cool);gl.uniform3fv(bgU.uWarmTint,atmosphere.warm);gl.uniform3fv(bgU.uBase,atmosphere.bg);gl.uniform1f(bgU.uOpen,opening);gl.uniform1f(bgU.uTime,ambientTime);gl.uniform1f(bgU.uAspect,w/fullH);gl.uniform1f(bgU.uWarmth,NexoApp.store.get().preferences.warmth);gl.drawArrays(gl.TRIANGLES,0,3);gl.viewport(0,Math.round((fullH-top-h)*ratio),canvas.width,Math.round(h*ratio));gl.useProgram(p);gl.enable(gl.DEPTH_TEST);gl.depthFunc(gl.LEQUAL);gl.enable(gl.BLEND);gl.blendFunc(gl.SRC_ALPHA,gl.ONE_MINUS_SRC_ALPHA);gl.uniformMatrix4fv(U.uVP,false,vp);gl.uniform3f(U.uEye,0,.08,eye);gl.uniform1i(U.uPoster,0);gl.uniform1i(U.uModule,1);gl.uniform1i(U.uGrain,2);gl.activeTexture(gl.TEXTURE2);gl.bindTexture(gl.TEXTURE_2D,grain);gl.uniform1f(U.uTime,ambientTime);renderCount++;gl.uniform1f(U.uWarmth,NexoApp.store.get().preferences.warmth);hitFaces=[];
 collecting=true;drawQueue=[];frontModel=null;
 const selected=mod(Math.round(target),N),poses=names.map((_,i)=>{const a=(i-position)*STEP,frontness=(Math.cos(a)+1)/2;return {i,a,z:Math.cos(a)*2,frontness};}).sort((a,b)=>a.z-b.z);
 for(const pose of poses){const {i,a}=pose,isFront=i===selected,op=isFront?opening:0,alpha=(isFront?1:Math.pow(1-opening,1.5))*(1-launching)*sceneReveal;if(alpha<.008)continue;let x=Math.sin(a)*3.05,y=.10*(1-Math.cos(a)),z=pose.z,yaw=-Math.sin(a)*.35;const closeDistance=eye-2-.30,maxHeightScale=.95*2*closeDistance/(H*f),sx=mix(1,Math.max(180,w-safeLeft-safeRight-24)/w*2*(closeDistance-.10)*aspect/(W*f),1-Math.pow(1-op,2)),sy=mix(1,maxHeightScale,clamp((op-.08)/.92,0,1));x=mix(x,(safeLeft-safeRight)/2*2*(eye-2.3)*aspect/(w*f),op);y*=1-op;z=mix(z,2.30,op)-.10*Math.sin(Math.PI*clamp(op*4,0,1));if(!isFront)z-=opening*.8;z+=isFront?launching*.9:0;yaw*=1-op;z-=2.2*(1-sceneReveal);y-=.14*(1-sceneReveal);yaw+=.12*(1-sceneReveal);
 const model=mul(mul(trans(x,y,z),mul(ry(yaw),rx(-.055*Math.sin(Math.PI*op)))),scale(sx,sy,1)),light=.69+.31*pose.frontness,color=[.055*light,.092*light,.123*light];drawInfo={page:i,poster:posters[i],internal:internals[i],blend:op,counter:0};if(isFront)frontModel=model;gl.activeTexture(gl.TEXTURE0);gl.bindTexture(gl.TEXTURE_2D,posters[i]);gl.activeTexture(gl.TEXTURE1);gl.bindTexture(gl.TEXTURE_2D,internals[i]);gl.uniform1f(U.uBlend,op);object(rim,mul(model,trans(0,0,-.022)),[.47,.59,.62],alpha,.94);drawMesh(panel.body,model,[.13,.19,.23],alpha,0,.82);drawMesh(panel.front,model,color,alpha,isFront?0:1,.12,1);
 const points=[[-W/2,H/2],[W/2,H/2],[W/2,-H/2],[-W/2,-H/2]].map(v=>{const p=project(model,v[0],v[1],.1);return [p[0],1-(top+(1-p[1])*h/2)/fullH*2];});hitFaces.push({i,points,z});
 const xs=points.map(p=>(p[0]+1)*w/2),ys=points.map(p=>(1-p[1])*fullH/2),face={left:Math.min(...xs)+4,top:Math.min(...ys)+4,width:Math.max(...xs)-Math.min(...xs)-8,height:Math.max(...ys)-Math.min(...ys)-8};
 let u=i===pulseIndex&&!reduced.matches?clamp((now-pulseStart)/450,0,1):1;const pulse=u<1?Math.sin(Math.PI*u):0,spin=u<1?TAU*(1-Math.pow(1-u,3)):0;
 NexoApp.onTabFace(points.map(p=>[(p[0]+1)*w/2,(1-p[1])*fullH/2]),i);
 if(isFront){NexoApp.onHomeFace(points.map(p=>[(p[0]+1)*w/2,(1-p[1])*fullH/2]),i,(1-op)*(1-launching)*sceneReveal,blurAmount);NexoApp.onFrame({opening:op,rect:face});}
 const parent=mul(trans(x,y,z),mul(ry(yaw),rx(-.055*Math.sin(Math.PI*op))));
 const iconDepth=.36+pulse*.30+.30*Math.sin(Math.PI*op),iconDistance=eye-z-iconDepth,pixel=2*iconDistance/(h*f),anchor=isFront&&op>.01?NexoApp.headerIconBox?.():null,anchorX=anchor?.width?anchor.left+anchor.width/2:face.left+32,anchorY=anchor?.height?anchor.top+anchor.height/2:face.top+36,headerX=(anchorX/w*2-1)*iconDistance*aspect/f-x,headerY=(1-(anchorY-top)/h*2)*iconDistance/f+.08-y,headerScale=Math.min(anchor?.width||32,32)*pixel/1.28;
 const iconM=mul(parent,mul(trans(mix(0,headerX,op),mix(.25,headerY,op),iconDepth),mul(ry(-.2+spin+.18*Math.sin(Math.PI*op)),mul(rx(.10),scale(mix(1,headerScale,op))))));const tint=NexoApp.themeColors?.(),cool=tint?.cool||[.62,.76,.80],warm=tint?.warm||[.64,.49,.29];gl.uniform1f(U.uTex,0);drawInfo.headerIcon=isFront&&op>.97;
 const context=isFront?NexoApp.contextIcon?.():null,contextBlend=context?clamp((op-.35)/.45,0,1):0;if(contextBlend<.999)minecraftIcon(i,iconM,cool,warm,alpha*(1-contextBlend));if(contextBlend>0)utilityIcon(context,iconM,cool,warm,alpha*contextBlend);
 drawInfo.headerIcon=false;
 if(i===4){
 const skinFrame=op>.97?NexoApp.skinRect():null;if(skinFrame){const r=skinFrame,dist=eye-z-.48,unit=2*dist/(h*f),size=Math.max(.01,Math.min(r.height*.88,r.width*.8)*unit/1.8),cx=((r.x+r.width/2)/w*2-1)*dist*aspect/f-x,cy=(1-(r.y+r.height/2-top)/h*2)*dist/f+.08-y,figure=mul(parent,mul(trans(cx,cy,.48),mul(ry(NexoApp.store.get().skinTurn),scale(size))));
 object(head,mul(figure,trans(0,.57,0)),[.73,.61,.45],1,.05);object(torso,mul(figure,trans(0,-.01,0)),cool,1,.12);
 for(const side of [-1,1]){object(limb,mul(figure,trans(side*.40,-.04,0)),[.36,.48,.57],1,.08);object(leg,mul(figure,trans(side*.14,-.64,0)),[.20,.28,.40],1,.08);}}
 }
 }
 collecting=false;const stage=[0,Math.round((fullH-top-h)*ratio),canvas.width,Math.round(h*ratio)],frame=motion.prepare(drawQueue,vp,stage,canvas.width,canvas.height,now,!reduced.matches&&NexoApp.store.get().preferences.motionBlur,selected);
 blurAmount=motion.getState().maxPixels/ratio;if(motion.getState().frontMoving&&opening<.01){for(const c of drawQueue)if(c.page===selected&&c.surface>.5)c.tex=1;NexoApp.hideMovingLabel();}
 paintScene(drawQueue,selected,stage,w*ratio,h*ratio);motion.finish(frame,eye);

 root.dataset.selected=names[selected];root.dataset.open=String(openTarget===1);root.dataset.launching=String(launchTarget===1);root.dataset.openAmount=opening.toFixed(4);root.dataset.launchAmount=launching.toFixed(4);root.dataset.position=position.toFixed(4);root.dataset.moving=String(Math.abs(position-target)>.001||Math.abs(velocity)>.001);root.dataset.webgl='ready';
}
function frame(now){raf=0;const elapsed=last?Math.min(.1,Math.max(0,(now-last)/1000)):0,dt=Math.min(.04,Math.max(.001,elapsed||.016));last=now;if(ambientEnabled())ambientTime+=elapsed;
 if(reduced.matches){if(!drag)position=target;velocity=0;opening=openTarget;openVelocity=0;launching=launchTarget;launchVelocity=0;pulsePending=false;pulseStart=-10000;}
 else{if(!drag)[position,velocity]=spring(position,velocity,target,dt);[opening,openVelocity]=spring(opening,openVelocity,openTarget,dt,16);[launching,launchVelocity]=spring(launching,launchVelocity,launchTarget,dt,19);if(pulsePending&&!drag&&Math.abs(position-target)<.15){pulseStart=now;pulseIndex=mod(Math.round(target),N);pulsePending=false;}}
 if(Math.abs(position-target)<.0001&&Math.abs(velocity)<.0002&&!drag){position=target;velocity=0;settled=true;}
 if(Math.abs(opening-openTarget)<.0001&&Math.abs(openVelocity)<.0002){opening=openTarget;openVelocity=0;}
 if(Math.abs(launching-launchTarget)<.0001&&Math.abs(launchVelocity)<.0002){launching=launchTarget;launchVelocity=0;}
 if(forcePaint||now-lastPaint>=1000/NexoApp.store.get().preferences.fps-1){render(now);forcePaint=false;lastPaint=now;}if(drag||!settled||Math.abs(launching-launchTarget)>.0001||Math.abs(launchVelocity)>.0002||Math.abs(opening-openTarget)>.0001||Math.abs(openVelocity)>.0002||now-pulseStart<460||pulsePending||NexoIntro.isActive())wake();else if(ambientEnabled())idleTimer=setTimeout(()=>{idleTimer=0;wake();},1000/24);else last=0;
}
function inside(points,x,y){let sign=0;for(let i=0;i<4;i++){const a=points[i],b=points[(i+1)%4],v=(b[0]-a[0])*(y-a[1])-(b[1]-a[1])*(x-a[0]);if(Math.abs(v)<.00001)continue;if(sign&&Math.sign(v)!==sign)return false;sign=Math.sign(v);}return !!sign;}
function hit(e){const r=canvas.getBoundingClientRect(),x=(e.clientX-r.left)/r.width*2-1,y=1-(e.clientY-r.top)/r.height*2;return [...hitFaces].sort((a,b)=>b.z-a.z).find(p=>inside(p.points,x,y))?.i;}
canvas.addEventListener('pointerdown',e=>{if(launchTarget||!e.isPrimary||e.button!==0||(!openTarget&&opening>.001))return;if(openTarget){if(opening>.97&&hit(e)===undefined){e.preventDefault();outsidePress={id:e.pointerId,x:e.clientX,y:e.clientY,moved:false};canvas.setPointerCapture(e.pointerId);}return;}const ix=hit(e);if(ix===undefined)return;e.preventDefault();drag={id:e.pointerId,x:e.clientX,y:e.clientY,lastX:e.clientX,time:performance.now(),base:position,vel:0,moved:false,axis:null,hit:ix};pulseStart=-10000;pulsePending=false;});
canvas.addEventListener('pointermove',e=>{if(outsidePress?.id===e.pointerId){if(Math.hypot(e.clientX-outsidePress.x,e.clientY-outsidePress.y)>8)outsidePress.moved=true;return;}if(!drag||drag.id!==e.pointerId)return;const dx=e.clientX-drag.x,dy=e.clientY-drag.y;if(!drag.axis&&Math.max(Math.abs(dx),Math.abs(dy))>7){drag.axis=Math.abs(dx)>Math.abs(dy)?'x':'y';if(drag.axis==='x')canvas.setPointerCapture(e.pointerId);}if(drag.axis!=='x')return;e.preventDefault();drag.moved=true;const now=performance.now(),span=canvas.clientWidth*.51;drag.vel=-(e.clientX-drag.lastX)/span/Math.max(.001,(now-drag.time)/1000);drag.time=now;drag.lastX=e.clientX;position=drag.base-dx/span;velocity=0;settled=false;wake();},{passive:false});
function release(e,cancelled=false){if(outsidePress?.id===e.pointerId){const d=outsidePress;outsidePress=null;if(canvas.hasPointerCapture(e.pointerId))canvas.releasePointerCapture(e.pointerId);if(!cancelled&&!d.moved&&Math.hypot(e.clientX-d.x,e.clientY-d.y)<=8&&hit(e)===undefined){tapGuard={x:e.clientX,y:e.clientY,at:performance.now()};closeSection();}return;}if(!drag||drag.id!==e.pointerId)return;const d=drag;drag=null;if(canvas.hasPointerCapture(e.pointerId))canvas.releasePointerCapture(e.pointerId);if(d.axis==='y'||cancelled){target=Math.round(position);velocity=0;}else if(!d.moved){tapGuard={x:e.clientX,y:e.clientY,at:performance.now()};activate(d.hit);return;}else{const v=performance.now()-d.time<100?clamp(d.vel,-7,7):0;velocity=v;target=Math.round(position+clamp(v*.075,-.6,.6));}pulsePending=!cancelled;settled=false;updateUI();wake();}
root.addEventListener('click',e=>{if(tapGuard&&e.detail>0&&performance.now()-tapGuard.at<250&&Math.hypot(e.clientX-tapGuard.x,e.clientY-tapGuard.y)<8){tapGuard=null;e.preventDefault();e.stopImmediatePropagation();}},{capture:true});
canvas.addEventListener('pointerup',e=>release(e));canvas.addEventListener('pointercancel',e=>release(e,true));canvas.addEventListener('lostpointercapture',e=>{if(drag||outsidePress)release(e,true);});
buttons.forEach((b,i)=>b.addEventListener('click',()=>activate(i)));
if(!NexoApp.productOnly)backButton.addEventListener('click',closeSection);

new ResizeObserver(()=>{forcePaint=true;wake();}).observe(canvas);reduced.addEventListener('change',()=>{forcePaint=true;wake();});canvas.addEventListener('webglcontextlost',e=>{e.preventDefault();root.querySelector('.nexo-error').hidden=false;root.querySelector('.nexo-error').textContent='A visualização 3D foi interrompida.';if(raf)cancelAnimationFrame(raf);clearTimeout(idleTimer);idleTimer=0;raf=0;NexoApp.sceneFailed?.();});
window.NexoScene={select:choose,activate,close:closeSection,openView(){openTarget=1;updateUI();forcePaint=true;wake();},updateLabels(all=false){for(let i=0;i<(all?N:1);i++){gl.deleteTexture(posters[i]);posters[i]=texture(i,false);}updateUI();forcePaint=true;wake();},
 getState:()=>({selected:mod(Math.round(target),N),position,target,velocity,open:!!openTarget,opening,launching:!!launchTarget,dragging:!!drag?.moved,visible,ready:true,ambientTime,renderCount,ambientActive:visible&&ambientEnabled(),blur:blurAmount,motion:motion.getState(),lighting:lightingStats,faces:hitFaces.map(p=>({index:p.i,points:p.points,z:p.z}))}),
 resetMotionHistory(){motion.reset();},refresh(){forcePaint=true;wake();},setLaunching(value){launchTarget=value?1:0;updateUI();wake();},
 setVisible(value){visible=!!value;if(!visible){motion.reset();if(raf)cancelAnimationFrame(raf);raf=0;clearTimeout(idleTimer);idleTimer=0;}if(visible){last=0;forcePaint=true;wake();}},
 inspectLayout,inspectLighting,inspectMotion:motion.inspect,sources:{vertex:vs,fragment:fs,backgroundVertex:bgvs,backgroundFragment:bgfs,shadowVertex:shadowVS,shadowFragment:shadowFS}};
canvas.addEventListener('webglcontextrestored',()=>location.reload());
document.fonts.load('86px Michroma').then(()=>{for(let i=0;i<N;i++){gl.deleteTexture(posters[i]);posters[i]=texture(i,false);}forcePaint=true;wake();});
updateUI();NexoApp.ready();NexoIntro.start();wake();
})();
