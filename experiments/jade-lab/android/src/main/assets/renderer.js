/* Shared 3D scene: analytic ray/plane and ray/box intersections, one camera,
 * point light, cast shadows and one-bounce reflections. DOM text uses the
 * exact same projective matrices. No CSS lighting or fake perspective. */
(function(){
  'use strict';
  const VERTEX_SOURCE='#version 300 es\nin vec2 aPosition;\nvoid main(){gl_Position=vec4(aPosition,0.,1.);}';
  const FRAGMENT_SOURCE=`#version 300 es
precision highp float;
out vec4 fragmentColor;
uniform vec2 uResolution,uViewport,uPointer;
uniform float uTime,uReduced,uNavigation;
uniform vec4 uPalette[5],uSceneNeutral[3];
uniform vec4 uAtmos,uLight,uMaterial,uSurface,uMotion,uDetail,uEnvironment;
uniform vec4 uBackdropShape,uBackdropLayers,uBackdropLight,uBackdropDetail;
uniform vec4 uGlassModel,uGlassOptics,uGlassDetail,uGlassTargets;
uniform sampler2D uSceneTexture;
uniform int uScenePass,uSelectionStyle,uCount;
uniform vec4 uCenters[8],uAxesU[8],uAxesV[8],uMeta[8];
uniform vec4 uBoxCenters[6],uBoxSizes[6];
float hash12(vec2 p) {
  vec3 p3=fract(vec3(p.xyx)*0.1031);
  p3+=dot(p3,p3.yzx+33.33);
  return fract((p3.x+p3.y)*p3.z);
}
float sround(vec2 p,vec2 center,vec2 size,float radius) {
  vec2 q=abs(p-center)-size*0.5+radius;
  return length(max(q,0.0))+min(max(q.x,q.y),0.0)-radius;
}
float radial(vec2 p,vec2 center,vec2 spread) {
  vec2 d=(p-center)/spread;
  return exp(-dot(d,d)*2.5);
}
float ggxD(float nh,float rough) {
  float a=rough*rough;
  float a2=a*a;
  float v=nh*nh*(a2-1.0)+1.0;
  return a2/max(3.14159*v*v,0.0001);
}
float ggxG(float nv,float nl,float rough) {
  float k=pow(rough+1.0,2.0)/8.0;
  float a=nv/(nv*(1.0-k)+k);
  float b=nl/(nl*(1.0-k)+k);
  return a*b;
}
vec3 tonemap(vec3 c) {
  return clamp(pow(max(c,vec3(0.0)),vec3(0.91)),0.0,1.0);
}

// Ribbon signed distance. A single coherent, continuous curve is layered
// rather than drawing a copyrighted console wallpaper or static PNG.
float ribbonHeight(float x,float idx,float time,float scale,float bend) {
  float p=x*scale;
  return .53+(idx-1.5)*.144
      + bend*(.11*sin(p*3.5-idx*.66+time*.93)
              + .046*sin(p*6.4+idx*1.25-time*.48));
}
vec3 renderBackdrop(vec2 uv,vec2 p,float time,vec2 mouse) {
  float style=uBackdropShape.x;
  float power=uBackdropShape.y;
  float scale=uBackdropShape.z;
  float bend=uBackdropShape.w;
  float count=uBackdropLayers.x;
  float width=uBackdropLayers.y;
  float softness=uBackdropLayers.z;
  float flow=time*uBackdropLayers.w;

  vec2 center=uBackdropLight.zw+(mouse-vec2(.5)) *
    (.14*uBackdropDetail.y);
  float grad=radial(uv,center,vec2(.85,.67));
  float opposite=radial(uv,center+vec2(.44,-.31),vec2(.8,.72));
  float depth=uBackdropLight.y;
  vec3 ink=uPalette[0].rgb;
  vec3 col=mix(ink,uPalette[1].rgb,clamp(grad*depth*.55,0.0,.49));
  col+=uPalette[3].rgb*opposite*depth*.016;
  col+=uPalette[2].rgb*grad*depth*.038;

  // Soft, spatially located ambient light; this is distinct from tile GGX.
  float lightPool=radial(uv,center+vec2(-.16,.18),vec2(.69,.64));
  col+=mix(uPalette[2].rgb,uPalette[3].rgb,.32)*
        lightPool*uBackdropLight.x*power*.052;
  col+=uPalette[4].rgb*
        radial(uv,center+vec2(.58,-.31),vec2(.8,.62))*
        power*uBackdropDetail.w*.014;

  if(style<2.5){
    // Four constant-bound layers are safe in WebGL2 ES; the loop is masked
    // by a user-controlled integer uniform so low-power phones may use one.
    for(int layer=0;layer<4;layer++){
      if(float(layer)>=count)continue;
      float i=float(layer);
      vec2 pos=uv;
      float t=flow*(.9+.23*i);
      float y=ribbonHeight(pos.x,i,t,scale,bend);
      y+=.024*sin(pos.x*4.0-t*.8)*uNavigation;
      if(style>1.5){
        // Satin: diagonal soft bands with very restrained movement.
        y=pos.x*(.26+.035*i)+.22+(i-1.5)*.12+
          bend*.09*sin(pos.x*scale*3.4+t+i*.8);
      }else if(style>0.5){
        // Aurora: larger vertical atmospheric curtains, not hard streaks.
        y=.40+(i-1.5)*.11+bend*.14*sin(pos.x*scale*2.1+t+i*.5);
      }
      float d=(pos.y-y);
      float spread=max(.012,(.018+width*.071)*softness);
      float envelope=exp(-pow(d/spread,2.0));
      float illuminated=exp(-pow((d-spread*.53)/max(spread*.55,.01),2.0));
      float under=exp(-pow((d+spread*.62)/max(spread*.89,.012),2.0));
      float horizonFade=smoothstep(-.16,.13,pos.x)*
                        (1.0-smoothstep(.86,1.12,pos.x));
      // Light follows the band curvature, producing a pearl-like highlight.
      float turn=.58+.42*sin(pos.x*scale*3.9+i*1.64-flow*.82);
      vec3 spectral=layer==0?uPalette[3].rgb:
                    layer==1?uPalette[2].rgb:
                    layer==2?uPalette[4].rgb:uPalette[3].rgb;
      float weight=(.92-.12*i)*power*horizonFade*uAtmos.y;
      col+=spectral*envelope*weight*(.36+.16*turn);
      col+=mix(spectral,vec3(.77,.84,.91),.21)*
           illuminated*weight*uBackdropDetail.z*.18;
      col-=ink*under*weight*.18;
    }
  }else if(style<3.5){
    // Halo: concentric diffuse fields and a short off-axis cold reflection.
    for(int i=0;i<3;i++){
      float layer=float(i);
      vec2 c=center+vec2(.16*layer-.14,-.12*layer+.10);
      float rad=length((uv-c)*vec2(1.0,1.20));
      float rim=exp(-pow((rad-(.25+.14*layer))/
         max(.03,.05*softness),2.0));
      vec3 spectral=i==0?uPalette[2].rgb:
                     i==1?uPalette[3].rgb:uPalette[4].rgb;
      col+=spectral*rim*power*uAtmos.y*.038;
    }
  }
  // Mapped background texture is kept lower contrast than text-bearing tiles.
  // The grain is stable across frames; no high-frequency flicker or shimmer.
  float coarse=hash12(floor(p*.37));
  float fine=hash12(floor(p*.91));
  float grain=(coarse*.32+fine*.68-.50)*
      (uAtmos.z+uBackdropDetail.x*.65)*.036;
  col+=grain;
  col+=uSceneNeutral[2].rgb*grad*uEnvironment.x*.018;
  float vignette=length((uv-.5)*vec2(1.04,.95));
  col*=1.0-smoothstep(.18,.79,vignette)*uAtmos.w*.56;
  return max(col,vec3(0.0));
}
// Blur the ACTUAL GPU-procedural scene defined above, not a CSS-opacity hack.
// The background has no framebuffer backing, so we evaluate it spatially
// (center + 4 axial + optionally 4 diagonal taps). This refracts the authored
// landscape. It does not sample native/DOM text or completed UI layers.
// All offset positions stay in viewport coordinates despite render scaling.
vec3 frostedBackdrop(vec2 uv,vec2 p,float time,vec2 mouse) {
  // Read the cached GPU scene; no more recursive expensive ribbons.
  // WebGL framebuffer stores texture Y bottom-up, while scene UV uses top-down.
  vec2 texUv=vec2(uv.x,1.0-uv.y);
  vec3 center=texture(uSceneTexture,texUv).rgb;
  if(uGlassDetail.w<.5||uGlassModel.z<=.001)return center;
  float radius=1.5+19.0*clamp(uGlassModel.z,0.0,1.0);
  vec2 dx=vec2(radius/max(uViewport.x,1.0),0.0);
  vec2 dy=vec2(0.0,radius/max(uViewport.y,1.0));
  vec3 sum=center*4.0;
  sum+=texture(uSceneTexture,clamp(texUv+dx,vec2(0.),vec2(1.))).rgb;
  sum+=texture(uSceneTexture,clamp(texUv-dx,vec2(0.),vec2(1.))).rgb;
  sum+=texture(uSceneTexture,clamp(texUv+dy,vec2(0.),vec2(1.))).rgb;
  sum+=texture(uSceneTexture,clamp(texUv-dy,vec2(0.),vec2(1.))).rgb;
  float weight=8.0;
  if(uGlassDetail.w>1.5){
    vec2 dxy=vec2(dx.x*.7071068,dy.y*.7071068);
    sum+=texture(uSceneTexture,clamp(texUv+dxy,vec2(0.),vec2(1.))).rgb;
    sum+=texture(uSceneTexture,clamp(texUv-dxy,vec2(0.),vec2(1.))).rgb;
    sum+=texture(uSceneTexture,clamp(texUv+vec2(dxy.x,-dxy.y),vec2(0.),vec2(1.))).rgb;
    sum+=texture(uSceneTexture,clamp(texUv+vec2(-dxy.x,dxy.y),vec2(0.),vec2(1.))).rgb;
    weight=12.0;
  }
  return sum/weight;
}
// Positive Z faces the camera. Every plane has orthonormal U/V axes.
float panelHit(vec3 ro,vec3 rd,int i,out vec3 hit,out vec2 local){
  vec3 n=normalize(cross(uAxesU[i].xyz,uAxesV[i].xyz));
  float denom=dot(rd,n);
  if(abs(denom)<.00001)return 1e8;
  float t=dot(uCenters[i].xyz-ro,n)/denom;
  if(t<=.05)return 1e8;
  hit=ro+rd*t;
  vec3 d=hit-uCenters[i].xyz;
  local=vec2(dot(d,uAxesU[i].xyz),dot(d,uAxesV[i].xyz));
  float sd=sround(local,vec2(0.),vec2(uCenters[i].w,uAxesU[i].w),uMeta[i].w);
  return sd<.5?t:1e8;
}
mat3 rotation(float a){float c=cos(a),s=sin(a);return mat3(c,0.,-s,0.,1.,0.,s,0.,c);}
float boxHit(vec3 ro,vec3 rd,int i,out vec3 normal,out vec3 local){
  mat3 rot=rotation(uBoxSizes[i].w);
  vec3 o=transpose(rot)*(ro-uBoxCenters[i].xyz),d=transpose(rot)*rd;
  vec3 safe=mix(vec3(-1.),vec3(1.),step(vec3(0.),d))*max(abs(d),vec3(.00001));
  vec3 t0=(-uBoxSizes[i].xyz-o)/safe,t1=(uBoxSizes[i].xyz-o)/safe;
  vec3 near3=min(t0,t1),far3=max(t0,t1);
  float nearT=max(max(near3.x,near3.y),near3.z),farT=min(min(far3.x,far3.y),far3.z);
  if(farT<max(nearT,.05))return 1e8;
  local=(o+d*nearT)/max(uBoxSizes[i].xyz,vec3(.001));
  vec3 n=vec3(0.);
  if(near3.x>=near3.y&&near3.x>=near3.z)n.x=-sign(d.x);
  else if(near3.y>=near3.z)n.y=-sign(d.y);else n.z=-sign(d.z);
  normal=rot*n;return nearT;
}
float shadowAt(vec3 p,vec3 normal,vec3 light,int selfPanel,int selfBox){
  vec3 ro=p+normal*1.3,rd=normalize(light-ro);
  float reach=length(light-ro),visibility=1.;
  for(int j=0;j<8;j++){
    if(j>=uCount)break;if(j==selfPanel)continue;
    vec3 h;vec2 q;float t=panelHit(ro,rd,j,h,q);
    if(t<reach)visibility*=mix(.23,.58,uMeta[j].z*uGlassModel.w);
  }
  for(int j=0;j<6;j++){
    if(j==selfBox)continue;vec3 n,q;
    if(boxHit(ro,rd,j,n,q)<reach)visibility*=.30;
  }
  return max(.18,visibility);
}
vec3 boxColor(int i,vec3 q,vec3 n){
  float kind=uBoxCenters[i].w;
  vec3 color=kind<.5?vec3(.56,.42,.31):kind<1.5?mix(uPalette[2].rgb,uSceneNeutral[1].rgb,.72):uPalette[1].rgb*.65;
  if(kind<.5&&q.y<-.40)color=vec3(.065,.057,.047);
  if(kind<.5&&n.z>.5&&q.y>-.08&&q.y<.12&&abs(abs(q.x)-.43)<.12)color=vec3(.03,.04,.04);
  return color;
}
vec3 reflectionAt(vec3 p,vec3 ray,int self){
  float closest=1e8;vec3 reflected=vec3(0.);
  for(int j=0;j<8;j++){
    if(j>=uCount)break;if(j==self)continue;vec3 h;vec2 q;
    float t=panelHit(p+ray*2.,ray,j,h,q);
    if(t<closest){closest=t;reflected=mix(uPalette[1].rgb,uPalette[2].rgb,uMeta[j].y*.25);}
  }
  for(int j=0;j<6;j++){
    vec3 n,q;float t=boxHit(p+ray*2.,ray,j,n,q);
    if(t<closest){closest=t;reflected=boxColor(j,q,n);}
  }
  return reflected;
}
vec3 lighting(vec3 base,vec3 p,vec3 n,vec3 eye,vec3 light,float rough,float metal,float visibility){
  vec3 L=normalize(light-p),V=normalize(eye-p),H=normalize(V+L);
  float nl=max(dot(n,L),0.),nv=max(dot(n,V),.01),nh=max(dot(n,H),0.),vh=max(dot(V,H),0.);
  vec3 f0=mix(vec3(.045),base,metal);
  vec3 f=f0+(vec3(1.)-f0)*pow(1.-vh,5.);
  vec3 spec=min(vec3(2.),f*ggxD(nh,rough)*ggxG(nv,nl,rough)/(4.*nv*max(nl,.01)));
  float distanceFalloff=1./(1.+dot(light-p,light-p)/2200000.);
  float energy=uLight.z*distanceFalloff*visibility;
  vec3 key=mix(uSceneNeutral[2].rgb,vec3(1.),.35);
  vec3 fill=mix(uPalette[4].rgb,vec3(.8),.36)*uEnvironment.z;
  return base*(uLight.w*.49+nl*energy*.83)*key+base*fill*(.05+.09*max(-L.x*n.x,0.))+
      spec*energy*uMaterial.z*(.48+uMotion.w*.12);
}
void main(){
  vec2 screen=vec2(gl_FragCoord.x/uResolution.x,1.-gl_FragCoord.y/uResolution.y)*uViewport;
  vec2 uv=screen/uViewport;
  float time=uTime*uMotion.x*(1.-uReduced);
  vec2 mouse=uPointer/uViewport;
  if(uScenePass==1){fragmentColor=vec4(renderBackdrop(uv,screen,time,mouse),1.);return;}
  vec3 behind=uScenePass==2?renderBackdrop(uv,screen,time,mouse):texture(uSceneTexture,vec2(uv.x,1.-uv.y)).rgb;
  vec3 eye=vec3(uViewport*.5,1200.),ray=normalize(vec3(screen-uViewport*.5,-1200.));
  float angle=radians(uLight.x);
  vec3 light=vec3(uViewport*vec2(.5+.36*cos(angle),.5+.36*sin(angle)),310.+uLight.y*240.);
  light.xy+=(mouse-.5)*uViewport*uMotion.z*.37;
  light.x+=sin(time*.7)*uViewport.x*.045*uMotion.y;
  float nearest=1e8;int panel=-1,box=-1;vec3 point=vec3(0.),normal=vec3(0.),boxLocal=vec3(0.);vec2 local=vec2(0.);
  for(int i=0;i<8;i++){
    if(i>=uCount)break;vec3 h;vec2 q;float t=panelHit(eye,ray,i,h,q);
    if(t<nearest){nearest=t;panel=i;point=h;local=q;}
  }
  for(int i=0;i<6;i++){
    vec3 n,q;float t=boxHit(eye,ray,i,n,q);
    if(t<nearest){nearest=t;box=i;point=eye+ray*t;normal=n;boxLocal=q;panel=-1;}
  }
  vec3 result=behind;
  if(panel<0&&box<0){
    // Shadows are projected onto the atmospheric back plane by the same light.
    vec3 back=eye+ray*((-140.-eye.z)/ray.z);
    float visibility=shadowAt(back,vec3(0.,0.,1.),light,-1,-1);
    result*=mix(1.,visibility,.50*uSurface.y);
  }else if(box>=0){
    vec3 base=boxColor(box,boxLocal,normal);
    float visibility=shadowAt(point,normal,light,-1,box);
    result=lighting(base,point,normal,eye,light,clamp(uMaterial.x+.18,.1,.9),.12,visibility);
    result+=uPalette[2].rgb*pow(1.-max(dot(normal,normalize(eye-point)),0.),3.)*uSurface.z*.065;
  }else{
    vec2 size=vec2(uCenters[panel].w,uAxesU[panel].w),q=local/size+.5;
    float sd=sround(local,vec2(0.),size,uMeta[panel].w);
    vec2 corner=sign(local)*max(abs(local)-size*.5+uMeta[panel].w,vec2(0.));
    vec2 edge=length(corner)>.001?normalize(corner):abs(local.x)/size.x>abs(local.y)/size.y?vec2(sign(local.x),0.):vec2(0.,sign(local.y));
    float bevel=smoothstep(-5.0,-.2,sd);
    vec3 U=uAxesU[panel].xyz,V=uAxesV[panel].xyz,N=normalize(cross(U,V));
    normal=normalize(N+bevel*(U*edge.x+V*edge.y)*uMaterial.w*.75);
    float selected=uMeta[panel].y;
    vec3 base=mix(uPalette[1].rgb,uSceneNeutral[1].rgb,.20);
    base=mix(base,uPalette[2].rgb,selected*.13);
    if(uMeta[panel].x<1.5){
      // Original embossed landscape: strata, distant ridges and a copper sun.
      float ridge=.42+.08*sin(q.x*5.)+.045*sin(q.x*13.+1.3);
      float terrain=smoothstep(ridge-.012,ridge+.012,q.y);
      base=mix(uPalette[0].rgb*.80,mix(uSceneNeutral[1].rgb,uPalette[2].rgb,.24)*.57,terrain);
      base+=uPalette[4].rgb*radial(q,vec2(.23,.28),vec2(.32,.30))*.16;
      float fine=abs(fract(q.y*35.+floor(q.x*22.)*.09)-.5);
      base+=uPalette[2].rgb*terrain*(1.-smoothstep(.02,.05,fine))*.015;
    }
    float visibility=shadowAt(point,normal,light,panel,-1);
    float rough=clamp(uMaterial.x,.075,.96),metal=clamp(uMaterial.y,0.,1.);
    result=lighting(base,point,normal,eye,light,rough,metal,visibility);
    bool glassAllowed=uMeta[panel].z>.5;
    if(glassAllowed&&uGlassModel.x<2.5){
      vec2 offset=(normal.xy-N.xy)*uGlassModel.z*.014;
      vec3 through=frostedBackdrop(clamp(uv+offset,vec2(0.),vec2(1.)),screen,time,mouse);
      float transmission=uGlassModel.w*(1.-uGlassModel.y*.76);
      if(uGlassModel.x>.5)transmission*=uGlassModel.x<1.5?.78:.40;
      vec3 backdropTone=mix(through,base,uGlassOptics.x*.40);
      if(uGlassModel.x>1.5)backdropTone=mix(backdropTone,base,.48);
      result=mix(result,backdropTone,transmission*.64);
      result+=uPalette[3].rgb*uGlassOptics.y*.015;
    }
    float nv=max(dot(normal,normalize(eye-point)),0.);
    float fresnel=pow(1.-nv,4.);
    vec3 reflected=reflectionAt(point,reflect(ray,normal),panel);
    result+=reflected*(.10+metal*.50)*uMaterial.z;
    vec2 envUv=clamp(uv+normal.xy*(.09+.09*rough),vec2(.001),vec2(.999));
    vec3 environment=texture(uSceneTexture,vec2(envUv.x,1.-envUv.y)).rgb;
    result+=environment*(.10+metal*.22)*uMaterial.z+uSceneNeutral[2].rgb*fresnel*uGlassOptics.z*.18;
    float border=1.-smoothstep(.25,1.3,abs(sd+.7));
    result+=mix(uSceneNeutral[2].rgb,uPalette[2].rgb,.25)*border*(.09+selected*.11)*uSurface.z;
    result+=uPalette[2].rgb*radial(q,vec2(.24,.2),vec2(1.,.9))*selected*uSurface.w*.045;
    result+=uSceneNeutral[2].rgb*exp(-pow((q.y-.10)/.17,2.))*uGlassDetail.y*.025;
    result+=(hash12(floor(screen*.8))-.5)*(uGlassDetail.x*.02);
    result*=1.-q.y*uGlassDetail.z*.055;
    if(selected>.5){
      if(uSelectionStyle==0)result+=uPalette[2].rgb*.035;
      else if(uSelectionStyle==1)result+=uPalette[2].rgb*border*.13;
      else if(uSelectionStyle==2)result+=uPalette[2].rgb*smoothstep(.93,.99,q.y)*.24;
    }
    result+=uPalette[2].rgb*uNavigation*exp(-pow((q.x-.5)/.6,2.))*.014;
    result=mix(behind,result,1.-smoothstep(-.7,.4,sd));
  }
  float lum=dot(result,vec3(.2126,.7152,.0722));
  result=mix(vec3(lum),result,uDetail.z);
  result=(result-.10)*uDetail.y+.10;
  result*=.94+uDetail.x*.12;
  fragmentColor=vec4(tonemap(result),1.);
}
`;
  let canvas,gl,program,uniform={},buffer,sceneTexture,sceneFbo,sceneSize=[0,0];
  let working=false,error='',preset={},paused=false,elapsed=0,last=0,raf=0,dirty=true;
  let cursor=[0,0],sceneAvailable=false,adaptiveScale=1,frames=0,sampleStart=0,cpuSum=0,gpuSum=0,gpuSamples=0,cpuMs=0,gpuMs=null,actualFps=0;
  let timerExt=null,queries=[],listeners=false,lastScaleChange=0,sceneData={surfaces:[],boxes:[],impulse:0},lastScene=null;
  const $=id=>document.getElementById(id);
  const num=(v,f)=>Number.isFinite(Number(v))?Number(v):f;
  const color=hex=>{const n=parseInt(String(hex||'#000000').replace('#',''),16)||0;return[(n>>16&255)/255,(n>>8&255)/255,(n&255)/255,1];};
  const vec4=(name,v)=>gl.uniform4fv(uniform[name],v);
  function compile(type,source){const shader=gl.createShader(type);gl.shaderSource(shader,source);gl.compileShader(shader);if(!gl.getShaderParameter(shader,gl.COMPILE_STATUS))throw Error(gl.getShaderInfoLog(shader));return shader;}
  function invalidate(){dirty=true;if(working&&!raf&&!document.hidden)raf=requestAnimationFrame(frame);}
  function notify(){const n=$('render-badge'),fps=Math.round(actualFps);if(n)n.textContent=fps+' FPS · '+Math.round(canvas.width/innerWidth*100)+'%';const info=$('live-hint');if(info)info.textContent=fps+' FPS · CPU '+cpuMs.toFixed(1)+' ms'+(gpuMs===null?'':' · GPU '+gpuMs.toFixed(1)+' ms');}
  function init(){
    canvas=$('gpu');if(!canvas)return false;
    try{
      gl=canvas.getContext('webgl2',{alpha:false,antialias:false,depth:false,stencil:false,preserveDrawingBuffer:false,powerPreference:'high-performance'});
      if(!gl)throw Error('WebGL2 não suportado');
      const vs=compile(gl.VERTEX_SHADER,VERTEX_SOURCE),fs=compile(gl.FRAGMENT_SHADER,FRAGMENT_SOURCE);
      program=gl.createProgram();gl.attachShader(program,vs);gl.attachShader(program,fs);gl.bindAttribLocation(program,0,'aPosition');gl.linkProgram(program);gl.deleteShader(vs);gl.deleteShader(fs);
      if(!gl.getProgramParameter(program,gl.LINK_STATUS))throw Error(gl.getProgramInfoLog(program));gl.useProgram(program);
      const names=['uResolution','uViewport','uPointer','uTime','uReduced','uNavigation','uPalette[0]','uSceneNeutral[0]','uAtmos','uLight','uMaterial','uSurface','uMotion','uDetail','uEnvironment','uBackdropShape','uBackdropLayers','uBackdropLight','uBackdropDetail','uGlassModel','uGlassOptics','uGlassDetail','uGlassTargets','uSceneTexture','uScenePass','uSelectionStyle','uCount','uCenters[0]','uAxesU[0]','uAxesV[0]','uMeta[0]','uBoxCenters[0]','uBoxSizes[0]'];
      names.forEach(k=>uniform[k]=gl.getUniformLocation(program,k));
      buffer=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,buffer);gl.bufferData(gl.ARRAY_BUFFER,new Float32Array([-1,-1,1,-1,-1,1,-1,1,1,-1,1,1]),gl.STATIC_DRAW);gl.enableVertexAttribArray(0);gl.vertexAttribPointer(0,2,gl.FLOAT,false,0,0);
      timerExt=gl.getExtension('EXT_disjoint_timer_query_webgl2');
      if(!listeners){
        listeners=true;
        window.addEventListener('pointermove',e=>{if(e.target.closest?.('#editor'))return;cursor=[e.clientX,e.clientY];invalidate();},{passive:true});
        window.addEventListener('pointerdown',e=>{if(e.target.closest?.('#editor'))return;cursor=[e.clientX,e.clientY];invalidate();},{passive:true});
        window.addEventListener('resize',()=>{window.JadeScene?.invalidate();invalidate();},{passive:true});
        document.addEventListener('visibilitychange',()=>{last=0;sampleStart=0;frames=0;cpuSum=0;if(document.hidden){cancelAnimationFrame(raf);raf=0;}else invalidate();});
        canvas.addEventListener('webglcontextlost',e=>{e.preventDefault();working=false;cancelAnimationFrame(raf);raf=0;error='Contexto GPU perdido';queries=[];});
        canvas.addEventListener('webglcontextrestored',()=>{sceneAvailable=false;sceneTexture=null;sceneFbo=null;sceneSize=[0,0];uniform={};init();});
      }
      cursor=[innerWidth*.38,innerHeight*.35];working=true;error='';document.body.classList.remove('gpu-unavailable');$('gpu-fallback').hidden=true;invalidate();return true;
    }catch(e){working=false;error=String(e.message||e);window.JadeScene?.tick(0);document.body.classList.add('gpu-unavailable');const n=$('gpu-fallback');if(n){n.hidden=false;n.textContent='GPU indisponível · '+error;}return false;}
  }
  function allocateScene(w,h){
    if(sceneAvailable&&sceneSize[0]===w&&sceneSize[1]===h)return;
    if(sceneTexture)gl.deleteTexture(sceneTexture);if(sceneFbo)gl.deleteFramebuffer(sceneFbo);
    sceneTexture=gl.createTexture();gl.bindTexture(gl.TEXTURE_2D,sceneTexture);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.LINEAR);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_S,gl.CLAMP_TO_EDGE);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_T,gl.CLAMP_TO_EDGE);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,w,h,0,gl.RGBA,gl.UNSIGNED_BYTE,null);
    sceneFbo=gl.createFramebuffer();gl.bindFramebuffer(gl.FRAMEBUFFER,sceneFbo);gl.framebufferTexture2D(gl.FRAMEBUFFER,gl.COLOR_ATTACHMENT0,gl.TEXTURE_2D,sceneTexture,0);sceneAvailable=gl.checkFramebufferStatus(gl.FRAMEBUFFER)===gl.FRAMEBUFFER_COMPLETE;gl.bindFramebuffer(gl.FRAMEBUFFER,null);gl.bindTexture(gl.TEXTURE_2D,null);sceneSize=[w,h];
    if(!sceneAvailable)throw Error('Framebuffer de cena incompleto');
  }
  function resize(){
    const scale=Math.max(.35,Math.min(1,num(preset.quality,.75)*adaptiveScale));
    const pixelCap=Math.min(1,Math.sqrt(900000/Math.max(1,innerWidth*innerHeight)));
    const dpr=Math.min(scale,pixelCap),w=Math.max(2,Math.round(innerWidth*dpr)),h=Math.max(2,Math.round(innerHeight*dpr));
    if(canvas.width!==w||canvas.height!==h){canvas.width=w;canvas.height=h;allocateScene(Math.max(2,Math.round(w*.72)),Math.max(2,Math.round(h*.72)));}
  }
  function pollGpu(){
    if(!timerExt)return;
    const disjoint=gl.getParameter(timerExt.GPU_DISJOINT_EXT);
    while(queries.length&&gl.getQueryParameter(queries[0],gl.QUERY_RESULT_AVAILABLE)){
      const q=queries.shift();if(!disjoint){const ms=gl.getQueryParameter(q,gl.QUERY_RESULT)/1e6;gpuSum+=ms;gpuSamples++;}gl.deleteQuery(q);
    }
    if(disjoint){queries.forEach(q=>gl.deleteQuery(q));queries=[];gpuMs=null;gpuSum=0;gpuSamples=0;}
  }
  function frame(now){
    raf=0;if(!working||document.hidden)return;
    const animated=!paused&&!preset.reducedMotion;
    const limit=Math.max(15,Math.min(120,num(preset.fps,60)));
    if(last&&now-last<1000/limit-.5){raf=requestAnimationFrame(frame);return;}
    const dt=last?Math.min((now-last)/1000,.1):0;last=now;
    if(animated)elapsed+=dt;
    const start=performance.now();sceneData=window.JadeScene.tick(elapsed);
    if(dirty||animated||sceneData.animating){
      pollGpu();let query=null;
      if(timerExt&&queries.length<4){query=gl.createQuery();gl.beginQuery(timerExt.TIME_ELAPSED_EXT,query);}
      resize();draw();
      if(query){gl.endQuery(timerExt.TIME_ELAPSED_EXT);queries.push(query);}
      cpuSum+=performance.now()-start;frames++;dirty=false;
      if(!sampleStart)sampleStart=now;
      if(now-sampleStart>=1000){
        actualFps=frames*1000/(now-sampleStart);cpuMs=cpuSum/frames;gpuMs=gpuSamples?gpuSum/gpuSamples:null;
        const cost=Math.max(cpuMs,gpuMs||0),budget=1000/limit;
        // Render work, not frame cadence: a deliberate 30 FPS cap is not load.
        if(now-lastScaleChange>2500&&cost>budget*.92&&adaptiveScale>.55){adaptiveScale=Math.max(.55,adaptiveScale-.08);lastScaleChange=now;}
        else if(now-lastScaleChange>6000&&cost<budget*.48&&adaptiveScale<1){adaptiveScale=Math.min(1,adaptiveScale+.04);lastScaleChange=now;}
        notify();sampleStart=now;frames=0;cpuSum=0;gpuSum=0;gpuSamples=0;
      }
    }
    if(animated||sceneData.animating)raf=requestAnimationFrame(frame);
  }
  function draw(){
    if(!working||!program)return;
    gl.useProgram(program);gl.uniform2f(uniform.uResolution,canvas.width,canvas.height);gl.uniform2f(uniform.uViewport,innerWidth,innerHeight);gl.uniform2f(uniform.uPointer,...cursor);gl.uniform1f(uniform.uTime,elapsed);gl.uniform1f(uniform.uReduced,preset.reducedMotion?1:0);gl.uniform1f(uniform.uNavigation,sceneData.impulse||0);
    gl.uniform4fv(uniform['uPalette[0]'],['bg','surface','accent','cool','warm'].flatMap(k=>color(preset[k])));gl.uniform4fv(uniform['uSceneNeutral[0]'],['sceneNeutral','sceneMetal','sceneHighlight'].flatMap(k=>color(preset[k])));
    vec4('uAtmos',[preset.fog,preset.waves,preset.noise,preset.vignette]);vec4('uLight',[preset.azimuth,preset.elevation,preset.intensity,preset.ambient]);vec4('uMaterial',[preset.roughness,preset.metallic,preset.reflectivity,preset.bevel]);vec4('uSurface',[preset.depth,preset.ao,preset.edgeLight,preset.heroGlow]);vec4('uMotion',[preset.speed,preset.pulse,preset.parallax,preset.bloom]);vec4('uDetail',[preset.diffusion,preset.contrast,preset.saturation,0]);vec4('uEnvironment',[preset.environmentLight,1,preset.environmentWarmth,0]);
    vec4('uBackdropShape',[Math.max(0,['ribbons','aurora','satin','halo','minimal'].indexOf(preset.backgroundStyle)),preset.backgroundIntensity,preset.backgroundScale,preset.ribbonCurve]);vec4('uBackdropLayers',[preset.ribbonLayers,preset.ribbonWidth,preset.ribbonSoftness,preset.backgroundFlow]);vec4('uBackdropLight',[preset.backgroundGlow,preset.gradientDepth,preset.backgroundCenterX,preset.backgroundCenterY]);vec4('uBackdropDetail',[preset.backgroundGrain,preset.backgroundParallax,preset.ribbonGleam,preset.backgroundContrast]);
    vec4('uGlassModel',[Math.max(0,['frosted','tinted','matte','solid'].indexOf(preset.glassStyle)),preset.glassOpacity,preset.glassBlur,preset.glassTransmission]);vec4('uGlassOptics',[preset.glassTint,preset.glassScatter,preset.glassFresnel,preset.glassRim]);vec4('uGlassDetail',[preset.glassGrain,preset.glassHighlight,preset.glassShadow,Math.max(0,['fast','balanced','high'].indexOf(preset.glassQuality))]);vec4('uGlassTargets',[+preset.glassHero,+preset.glassCards,+preset.glassChrome,+preset.glassCTA]);gl.uniform1i(uniform.uSelectionStyle,Math.max(0,['halo','rim','underline','quiet'].indexOf(preset.selectionStyle)));
    gl.uniform1i(uniform.uCount,sceneData.surfaces.length);
    const centers=new Float32Array(32),us=new Float32Array(32),vs=new Float32Array(32),meta=new Float32Array(32),bc=new Float32Array(24),bs=new Float32Array(24);
    sceneData.surfaces.forEach((p,i)=>{centers.set([...p.c,p.w],i*4);us.set([...p.u,p.h],i*4);vs.set([...p.v,0],i*4);meta.set([p.kind,+p.selected,+p.glass,12],i*4);});sceneData.boxes.forEach((b,i)=>{bc.set([...b.c,b.kind],i*4);bs.set([...b.size,b.angle],i*4);});
    gl.uniform4fv(uniform['uCenters[0]'],centers);gl.uniform4fv(uniform['uAxesU[0]'],us);gl.uniform4fv(uniform['uAxesV[0]'],vs);gl.uniform4fv(uniform['uMeta[0]'],meta);gl.uniform4fv(uniform['uBoxCenters[0]'],bc);gl.uniform4fv(uniform['uBoxSizes[0]'],bs);
    gl.activeTexture(gl.TEXTURE0);gl.uniform1i(uniform.uSceneTexture,0);gl.bindTexture(gl.TEXTURE_2D,null);gl.bindFramebuffer(gl.FRAMEBUFFER,sceneFbo);gl.viewport(0,0,...sceneSize);gl.uniform2f(uniform.uResolution,...sceneSize);gl.uniform1i(uniform.uScenePass,1);gl.drawArrays(gl.TRIANGLES,0,6);
    gl.bindFramebuffer(gl.FRAMEBUFFER,null);gl.viewport(0,0,canvas.width,canvas.height);gl.uniform2f(uniform.uResolution,canvas.width,canvas.height);gl.bindTexture(gl.TEXTURE_2D,sceneTexture);gl.uniform1i(uniform.uScenePass,0);gl.drawArrays(gl.TRIANGLES,0,6);lastScene=sceneData;
  }
  function setPreset(p){preset={...p};window.JadeScene?.setPreset(preset);invalidate();}
  function pause(v){paused=!!v;last=0;invalidate();return paused;}
  function screenshot(){if(!working)return null;resize();draw();return canvas.toDataURL('image/png');}
  function state(){return {webgl2:working,error,paused,sceneCache:sceneAvailable,adaptiveScale,fps:actualFps,cpuMs,gpuMs,dimensions:[canvas?.width,canvas?.height],surfaces:sceneData.surfaces.length,elapsed};}
  window.JadeRenderer={init,setPreset,pause,recenter(){cursor=[innerWidth*.38,innerHeight*.35];invalidate();},setPage(){invalidate();},refresh:invalidate,invalidate,screenshot,state,vertexSource:VERTEX_SOURCE,fragmentSource:FRAGMENT_SOURCE,geometry:()=>lastScene};
})();
