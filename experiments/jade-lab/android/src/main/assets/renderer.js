/* Horizon Jade Lab: WebGL2 scene with physically evaluated material response.
 * Entire display is rendered as a continuous GPU composition; labels/buttons
 * are a crisp interactive DOM overlay. No image sprites or copyrighted assets.
 */
(function () {
  "use strict";
  const VERTEX_SOURCE = '#version 300 es\nin vec2 aPosition;\nvoid main(){gl_Position=vec4(aPosition,0.0,1.0);}';
  const FRAGMENT_SOURCE = `#version 300 es
precision highp float;
out vec4 fragmentColor;
uniform vec2 uResolution;
uniform vec2 uViewport;
uniform vec2 uPointer;
uniform float uTime;
uniform float uReduced;
uniform int uRectCount;
uniform vec4 uRects[12];
uniform vec4 uPalette[5];
uniform vec4 uAtmos;
uniform vec4 uLight;
uniform vec4 uMaterial;
uniform vec4 uSurface;
uniform vec4 uMotion;
uniform vec4 uDetail;
uniform int uFocusIndex;
uniform int uSelectionStyle;
// The backdrop uses independent uniforms to avoid changing the material ABI.
uniform vec4 uBackdropShape;  // style, brightness, scale, curvature
uniform vec4 uBackdropLayers; // count, width, softness, flow rate
uniform vec4 uBackdropLight;  // halo, depth, centerX, centerY
uniform vec4 uBackdropDetail; // texture, parallax, edge gleam, color contrast
// Independent dark glass system: style, translucency, diffusion and quality.
// Each float is supplied from a real editable/exportable Jade Lab parameter.
uniform vec4 uGlassModel;  // style [0..3], opacity, blurRadius, transmission
uniform vec4 uGlassOptics; // tint, scatter, fresnel, chromatic rim
uniform vec4 uGlassDetail; // fine grain, satin glint, contact shadow, tap tier
uniform vec4 uGlassTargets;// hero, tiles, system chrome, play CTA
uniform sampler2D uSceneTexture;
uniform int uScenePass; // 1=backdrop draw, 0=composite, 2=fallback
uniform vec4 uEnvironment; // environment neutral, pearl highlight, warmth, light spread

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
      + bend*(.11*sin(p*3.5-idx*.66+time*.19)
              + .046*sin(p*6.4+idx*1.25-time*.11));
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
    (.034*uBackdropDetail.y);
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
      float t=flow*(.33+.11*i);
      float y=ribbonHeight(pos.x,i,t,scale,bend);
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
      float turn=.58+.42*sin(pos.x*scale*3.9+i*1.64-flow*.28);
      vec3 spectral=layer==0?uPalette[3].rgb:
                    layer==1?uPalette[2].rgb:
                    layer==2?uPalette[4].rgb:uPalette[3].rgb;
      float weight=(.78-.10*i)*power*horizonFade*uAtmos.y;
      col+=spectral*envelope*weight*(.102+.065*turn);
      col+=mix(spectral,vec3(.77,.84,.91),.21)*
           illuminated*weight*uBackdropDetail.z*.055;
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
  vec3 center=texture(uSceneTexture,uv).rgb;
  if(uGlassDetail.w<.5||uGlassModel.z<=.001)return center;
  float radius=1.5+19.0*clamp(uGlassModel.z,0.0,1.0);
  vec2 dx=vec2(radius/max(uViewport.x,1.0),0.0);
  vec2 dy=vec2(0.0,radius/max(uViewport.y,1.0));
  vec3 sum=center*4.0;
  sum+=texture(uSceneTexture,clamp(uv+dx,vec2(0.),vec2(1.))).rgb;
  sum+=texture(uSceneTexture,clamp(uv-dx,vec2(0.),vec2(1.))).rgb;
  sum+=texture(uSceneTexture,clamp(uv+dy,vec2(0.),vec2(1.))).rgb;
  sum+=texture(uSceneTexture,clamp(uv-dy,vec2(0.),vec2(1.))).rgb;
  float weight=8.0;
  if(uGlassDetail.w>1.5){
    vec2 dxy=vec2(dx.x*.7071068,dy.y*.7071068);
    sum+=texture(uSceneTexture,clamp(uv+dxy,vec2(0.),vec2(1.))).rgb;
    sum+=texture(uSceneTexture,clamp(uv-dxy,vec2(0.),vec2(1.))).rgb;
    sum+=texture(uSceneTexture,clamp(uv+vec2(dxy.x,-dxy.y),vec2(0.),vec2(1.))).rgb;
    sum+=texture(uSceneTexture,clamp(uv+vec2(-dxy.x,dxy.y),vec2(0.),vec2(1.))).rgb;
    weight=12.0;
  }
  return sum/weight;
}
void main() {
  vec2 p=vec2(gl_FragCoord.x/uResolution.x,1.0-gl_FragCoord.y/uResolution.y)*uViewport;
  vec2 uv=p/max(uViewport,vec2(1.0));
  // Physical viewport coordinates remain independent from render scale.
  // Reduced motion freezes time, while cursor-driven lighting remains alive.
  float tt=uTime*uMotion.x*(1.0-uReduced);
  vec2 cursor=uPointer/max(uViewport,vec2(1.0));
  if(uScenePass==1){
    vec3 background=renderBackdrop(uv,p,tt,cursor);
    float diffused=radial(uv,vec2(.5,.43),vec2(.85,.8));
    background+=vec3(uEnvironment.x)*diffused*.20+
      vec3(1.0,.96,.89)*uEnvironment.z*diffused*.015;
    fragmentColor=vec4(background,1.0);
    return;
  }
  vec3 result=uScenePass==2?renderBackdrop(uv,p,tt,cursor):
    texture(uSceneTexture,uv).rgb;
  cursor-=vec2(.5);
  float angle=uLight.x*3.14159265/180.0;
  vec3 light=normalize(vec3(cos(angle)*.87+cursor.x*uMotion.z,
        sin(angle)*.87-cursor.y*uMotion.z,uLight.y+.26));
  vec3 view=vec3(0.0,0.0,1.0);
  vec3 halfVec=normalize(view+light);
  float rough=clamp(uMaterial.x,.075,.96);
  float metallic=clamp(uMaterial.y,0.0,1.0);

  for(int i=0;i<12;i++) {
    if(i>=uRectCount) break;
    vec4 r=uRects[i];
    if(r.z<=0.0 || r.w<=0.0) continue;
    // Skip full material calculation outside the local bounding rectangle.
    if(p.x<r.x-31.0 || p.y<r.y-31.0 ||
       p.x>r.x+r.z+31.0 || p.y>r.y+r.w+31.0) continue;
    float radius=(i==2?22.0:i==10?13.0:i==0?17.0:12.0);
    radius=min(radius,min(r.z,r.w)*.26);
    vec2 mid=r.xy+r.zw*.5;
    float d=sround(p,mid,r.zw,radius);
    float shadow=max(0.0,1.0-smoothstep(1.0,26.0,d));
    result*=1.0-shadow*uSurface.y*.18;
    float inMask=1.0-smoothstep(-.55,.85,d);
    if(inMask<0.0001) continue;
    // Actual focused surface: no procedural left stripe or baked status bar.
    // Local material response is calculated in screen space on every frame.
    bool focused=i==uFocusIndex;
    vec2 q=(p-r.xy)/max(r.zw,vec2(1.0));
    float bevelDistance=smoothstep(-10.0,-.4,d);
    vec3 normal=normalize(vec3(
       (q.x-.5)*.52 + (q.x-.5)*bevelDistance*uMaterial.w*.68,
       (q.y-.5)*.52 + (q.y-.5)*bevelDistance*uMaterial.w*.68,
       1.0-bevelDistance*uMaterial.w*.24));
    float nl=max(dot(normal,light),0.0);
    float nv2=max(dot(normal,view),.01);
    float nh=max(dot(normal,halfVec),0.0);
    float vh=max(dot(view,halfVec),0.0);
    float dspec=ggxD(nh,rough);
    float gspec=ggxG(nv2,nl,rough);
    vec3 base=uPalette[1].rgb;
    if(i==3 || i==7) base=mix(base,uPalette[2].rgb,.21);
    if(i==4 || i==6) base=mix(base,uPalette[3].rgb,.25);
    if(i==5) base=mix(base,uPalette[4].rgb,.22);
    if(i==2) base=mix(base,uPalette[2].rgb,.09);
    if(i==10) base=mix(base,uPalette[2].rgb,.52);
    if(i==0 || i==1 || i==9) base=mix(uPalette[0].rgb,base,.69);
    float softTop=mix(.9,1.08,1.0-q.y);
    base*=softTop;
    vec3 f0=mix(vec3(.034),base,metallic*.74);
    vec3 fresnel=f0+(vec3(1.0)-f0)*pow(1.0-vh,5.0);
    vec3 spec=fresnel*dspec*gspec/(4.0*nv2*max(nl,.01));
    spec=min(spec,vec3(1.35));
    float ao=1.0-uSurface.y*(.04+(.25*max(q.y-.48,0.0)));
    float lighting=uLight.w+nl*uLight.z*.39;
    vec3 mat=base*lighting*ao+spec*uMaterial.z*uLight.z*.47;
    // Physically legible dark glass variants, drawn over the same original
    // procedural backdrop: refracted transmission, matte pigment, Fresnel
    // edge response, top-face microreflection and stable surface grain.
    bool glassAllowed=(i==2?uGlassTargets.x>.5:
      i==10?uGlassTargets.w>.5:
      (i>=3 && i<=8)?uGlassTargets.y>.5:uGlassTargets.z>.5);
    if(glassAllowed && uGlassModel.x<2.5) {
      vec2 screenUv=p/max(uViewport,vec2(1.0));
      vec2 pointerUv=cursor+vec2(.5);
      vec3 behind=frostedBackdrop(screenUv,p,tt,pointerUv);
      float model=uGlassModel.x;
      float pigment=clamp(uGlassModel.y,0.0,1.0);
      float transmission=clamp(uGlassModel.w,0.0,1.0);
      float tint=clamp(uGlassOptics.x,0.0,1.0);
      float scatter=clamp(uGlassOptics.y,0.0,1.0);
      vec3 backdropTone=mix(behind,
        mix(behind,uPalette[1].rgb,.28+.37*tint),tint);
      if(model>.5 && model<1.5){
        // Tinted Frost: tinted thicker polymer with more color and less
        // transmission than Frosted Dark.
        backdropTone=mix(backdropTone,uPalette[1].rgb,.20+.28*tint);
        transmission*=.78;
      }else if(model>1.5){
        // Dense Matte: dense pigment, low transmission, broader reflected
        // light; perceptibly different from a thin glass card.
        backdropTone=mix(backdropTone,base,.48);
        transmission*=.40;
      }
      float visible=transmission*(1.0-pigment*.76);
      mat=mix(mat,backdropTone,visible);
      // Turbid scattering grows toward glancing angles, never pure neon.
      float glancing=pow(1.0-nv2,2.6);
      mat+=uPalette[3].rgb*scatter*(.035+.065*glancing);
      float fresnel=pow(1.0-clamp(dot(normal,view),0.0,1.0),4.5);
      mat+=mix(uPalette[3].rgb,vec3(1.),.43)*
          fresnel*uGlassOptics.z*.30;
      float trim=bevelDistance*(.20+.80*max(dot(normal,light),0.0));
      mat+=mix(uPalette[2].rgb,vec3(.82,.92,.96),.39)*
          trim*uGlassOptics.w*.090;
      // Long top reflection sits on the material, rather than a hard strip.
      float topSweep=exp(-pow((q.y-(.11+.035*sin(q.x*4.1)))/.18,2.0))*
                     smoothstep(.04,.26,q.x)*(1.0-smoothstep(.85,.98,q.x));
      mat+=mix(vec3(1.),uPalette[3].rgb,.4)*
          topSweep*uGlassDetail.y*(.036+.028*nl);
      // Fine-grained roughness remains spatially stable at all frame rates.
      float grain=(hash12(floor(p*.67))-0.5)*uGlassDetail.x*.035;
      mat+=vec3(grain);
      mat*=1.0-clamp(uGlassDetail.z,0.,1.)*
          smoothstep(.42,.97,q.y)*.12;
    }
    // Gradients are physically located near a light source, not tint overlays.
    vec2 pointToLight=q-vec2(.25+.20*cursor.x,.18+.16*cursor.y);
    mat+=uPalette[2].rgb*exp(-dot(pointToLight,pointToLight)*4.2)
         *uSurface.x*.060;
    float rim=bevelDistance*uSurface.z;
    mat+=mix(uPalette[2].rgb,vec3(1.0),.48)*rim*(.035+.075*nl);
    float innerShadow=smoothstep(.03,.20,q.y)*(1.0-smoothstep(.76,.92,q.y));
    mat*=1.0-innerShadow*uSurface.y*.045;
    if(i==2 || i==10) {
      float aura=radial(q,vec2(.32,.76),vec2(.87,1.1));
      float breathing=.8+.2*sin(tt*1.8)*uMotion.y;
      mat+=uPalette[2].rgb*aura*uSurface.w*.082*breathing;
    }
    if(i==10) mat+=uPalette[2].rgb*.025;
    float border=(1.0-smoothstep(.2,1.9,abs(d+.5)));
    mat+=mix(vec3(.13),uPalette[2].rgb,.62)*border*uSurface.z*.23;
    // Focus does not add a fixed sidebar. It modifies the material's
    // physically located light, Fresnel contour, or soft bottom glow.
    if(focused){
      float pulse=1.0-uReduced*.12+sin(tt*1.6)*uMotion.y*.12*(1.0-uReduced);
      vec2 hotspot=q-vec2(.3+.1*cursor.x,.28+.08*cursor.y);
      float halo=exp(-dot(hotspot,hotspot)*4.5)*pulse;
      float edgeMask=1.0-smoothstep(.1,2.1,abs(d+.6));
      if(uSelectionStyle==0){ // Material halo (default)
        mat+=uPalette[2].rgb*halo*.105*uSurface.w;
        mat+=mix(uPalette[3].rgb,vec3(1.0),.45)*edgeMask*.035*uSurface.z;
      }else if(uSelectionStyle==1){ // Rim-lit outline
        mat+=mix(uPalette[2].rgb,vec3(1.0),.23)*edgeMask*.15*uSurface.z;
        mat+=uPalette[3].rgb*halo*.035*uSurface.w;
      }else if(uSelectionStyle==2){ // Soft lower emphasis
        float under=exp(-pow((q.y-.965)*39.0,2.0));
        under*=smoothstep(.05,.29,q.x)*(1.0-smoothstep(.71,.95,q.x));
        mat+=uPalette[2].rgb*under*.17;
        mat+=uPalette[2].rgb*halo*.025;
      }else{ // Quiet selection (accessibility-friendly)
        mat+=uPalette[2].rgb*halo*.025;
      }
    }
    // A deliberately subtle bloom, per local tile rather than global blur.
    mat+=spec*uMotion.w*.12;
    result=mix(result,mat,inMask*.96);
  }
  float luminance=dot(result,vec3(.2126,.7152,.0722));
  result=mix(vec3(luminance),result,uDetail.z);
  result=(result-.5)*uDetail.y+.5;
  result*=.94+uDetail.x*.12;
  result=tonemap(result);
  fragmentColor=vec4(result,1.0);
}
`;
  const NAMES = ["topbar", "nav", "hero", "mods", "packs", "shaders", "files", "skins", "settings", "footer", "cta"];
  let canvas, gl, program, uniform = {}, buffer, working = false;
  let sceneTexture=null,sceneFbo=null,sceneSize=[0,0],sceneAvailable=false,
    geometryDirty=true,rectCache=new Float32Array(12*4),
    slowFrames=0,adaptiveScale=1;
  let preset, paused = false, last = 0, elapsed = 0, dpr = 1, renderCount = 0;
  let selectedPage="Play";
  let cursor = [0, 0], error = "";
  const $ = (id) => document.getElementById(id);
  const num = (v, fallback) => Number.isFinite(Number(v)) ? Number(v) : fallback;
  function color(hex) {
    const s = String(hex || "#000000").replace("#", "");
    const n = parseInt(s, 16) || 0;
    return [(n >> 16 & 255) / 255, (n >> 8 & 255) / 255, (n & 255) / 255, 1];
  }
  function compile(type, source) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      const problem = gl.getShaderInfoLog(shader) || "Shader compilation failed";
      gl.deleteShader(shader);
      throw new Error(problem);
    }
    return shader;
  }
  function init() {
    canvas = $("gpu");
    if (!canvas) return false;
    try {
      gl = canvas.getContext("webgl2", { alpha: false, premultipliedAlpha: false,
        antialias: false, depth: false, stencil: false, preserveDrawingBuffer: true,
        powerPreference: "high-performance" });
      if (!gl) throw new Error("WebGL2 não suportado");
      const vs = compile(gl.VERTEX_SHADER, VERTEX_SOURCE);
      const fs = compile(gl.FRAGMENT_SHADER, FRAGMENT_SOURCE);
      program = gl.createProgram();
      gl.attachShader(program, vs);
      gl.attachShader(program, fs);
      gl.bindAttribLocation(program, 0, "aPosition");
      gl.linkProgram(program);
      gl.deleteShader(vs);
      gl.deleteShader(fs);
      if (!gl.getProgramParameter(program, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(program));
      gl.useProgram(program);
      const keys = ["uResolution","uViewport","uPointer","uTime","uReduced","uRectCount",
        "uRects[0]","uPalette[0]","uAtmos","uLight","uMaterial","uSurface","uMotion","uDetail",
        "uFocusIndex","uSelectionStyle","uBackdropShape",
        "uBackdropLayers","uBackdropLight","uBackdropDetail",
        "uGlassModel","uGlassOptics","uGlassDetail","uGlassTargets",
        "uSceneTexture","uScenePass","uEnvironment"];
      keys.forEach(k => { uniform[k] = gl.getUniformLocation(program, k); });
      gl.uniform1i(uniform.uSceneTexture,0);
      buffer = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
      gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1,-1,1,-1,-1,1, -1,1,1,-1,1,1]), gl.STATIC_DRAW);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
      cursor = [innerWidth*.35, innerHeight*.5];
      canvas.addEventListener("webglcontextlost",e=>{
        e.preventDefault();working=false;error="Contexto GPU perdido";
        sceneTexture=null;sceneFbo=null;sceneAvailable=false;notify(error);
      });
      canvas.addEventListener("webglcontextrestored", () => {working=false;init();});
      window.addEventListener("pointermove", e => {if(!e.target.closest || !e.target.closest("#editor")) cursor=[e.clientX,e.clientY];}, {passive:true});
      window.addEventListener("resize",()=>{geometryDirty=true;resize();},{passive:true});
      document.addEventListener("pointerdown",()=>{geometryDirty=true;},{passive:true});
      document.addEventListener("visibilitychange", () => {last=0;});
      working = true;
      resize();
      requestAnimationFrame(frame);
      return true;
    } catch(e) {
      error = e.message || String(e);
      const hint = $("gpu-fallback");
      if (hint) {hint.hidden=false;hint.textContent="WebGL2: "+error;}
      working=false;
      return false;
    }
  }
  function notify(s) {
    const label=$("render-badge");
    if(label) label.textContent=s.substring(0,36);
  }
  function allocateScene(w,h){
    if(!gl||!w||!h)return;
    if(sceneAvailable&&sceneSize[0]===w&&sceneSize[1]===h)return;
    if(sceneTexture)gl.deleteTexture(sceneTexture);
    if(sceneFbo)gl.deleteFramebuffer(sceneFbo);
    sceneTexture=gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D,sceneTexture);
    gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_S,gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_T,gl.CLAMP_TO_EDGE);
    gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,w,h,0,gl.RGBA,gl.UNSIGNED_BYTE,null);
    sceneFbo=gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER,sceneFbo);
    gl.framebufferTexture2D(gl.FRAMEBUFFER,gl.COLOR_ATTACHMENT0,
      gl.TEXTURE_2D,sceneTexture,0);
    sceneAvailable=gl.checkFramebufferStatus(gl.FRAMEBUFFER)===gl.FRAMEBUFFER_COMPLETE;
    gl.bindFramebuffer(gl.FRAMEBUFFER,null);
    gl.bindTexture(gl.TEXTURE_2D,null);
    sceneSize=[w,h];
  }
  function refreshRects(){
    rectCache.fill(0);
    NAMES.forEach((name,index)=>{
      const element=document.querySelector('[data-surface="'+name+'"]');
      if(!element||element.classList?.contains("tile"))return;
      if((name==="topbar"||name==="nav"||name==="footer")&&!preset.glassChrome)return;
      if(getComputedStyle(element).display==="none")return;
      const r=element.getBoundingClientRect();
      if(r.width<2||r.height<2)return;
      rectCache.set([r.x,r.y,r.width,r.height],index*4);
    });
    geometryDirty=false;
  }
  function resize() {
    if (!gl || !canvas) return;
    // Scale independently of Android DPR (often x3): DOM text stays sharp.
    const scale=Math.max(.38,Math.min(.88,
      num(preset && preset.quality,.62)*adaptiveScale));
    const pixelCap=Math.min(1,Math.sqrt(620000/Math.max(1,innerWidth*innerHeight)));
    dpr=Math.min(scale,pixelCap);
    const w=Math.max(2,Math.round(innerWidth*dpr)),h=Math.max(2,Math.round(innerHeight*dpr));
    if(canvas.width!==w||canvas.height!==h){
      canvas.width=w;canvas.height=h;gl.viewport(0,0,w,h);
      allocateScene(Math.max(2,Math.floor(w*.64)),Math.max(2,Math.floor(h*.64)));
    }
  }
  function frame(now) {
    if (!working) return;
    requestAnimationFrame(frame);
    if (document.hidden) return;
    const limit=Math.max(15,Math.min(120,num(preset && preset.fps,60)));
    if(now-last<1000/limit - .6) return;
    let dt=last?Math.min((now-last)/1000,.09):0;
    last=now;
    if(!paused && !(preset && preset.reducedMotion)) elapsed+=dt;
    resize();
    if(dt>.054)slowFrames++;else slowFrames=Math.max(0,slowFrames-1);
    if(slowFrames>=20&&adaptiveScale>.52){
      adaptiveScale=Math.max(.52,adaptiveScale-.12);
      slowFrames=0;resize();notify("GPU ECO · "+Math.round(adaptiveScale*100)+"%");
    }
    draw();
    renderCount++;
    if(renderCount%120===0)notify("WEBGL2 · "+
      Math.round(1000/Math.max(1,dt*1000))+" FPS");
  }
  function vec4(name, values) {
    gl.uniform4fv(uniform[name], new Float32Array(values));
  }
  function draw() {
    if (!preset || !program || !working) return;
    gl.useProgram(program);
    const e=preset.environmentLight??.3;
    gl.uniform4f(uniform.uEnvironment,e,.60,preset.environmentWarmth??.28,.60);
    gl.uniform2f(uniform.uResolution,canvas.width,canvas.height);
    gl.uniform2f(uniform.uViewport,innerWidth,innerHeight);
    gl.uniform2f(uniform.uPointer,cursor[0],cursor[1]);
    gl.uniform1f(uniform.uTime,elapsed);
    gl.uniform1f(uniform.uReduced,preset.reducedMotion?1:0);
    // Cached layout; transformed cards are rendered by the compositor.
    if(geometryDirty||renderCount%18===0)refreshRects();
    gl.uniform1i(uniform.uRectCount,NAMES.length);
    const surfaces={Play:"hero",Mods:"mods",Packs:"packs",Shaders:"shaders",
      Files:"files",Skins:"skins",Settings:"settings"};
    const selectedSurface=surfaces[selectedPage]||"hero";
    // Surface order is unchanged by navigation: only the light source moves.
    gl.uniform1i(uniform.uFocusIndex,NAMES.indexOf(selectedSurface));
    const modes=["halo","rim","underline","quiet"];
    gl.uniform1i(uniform.uSelectionStyle,
      Math.max(0,modes.indexOf(preset.selectionStyle)));
    gl.uniform4fv(uniform["uRects[0]"],rectCache);
    const c=["bg","surface","accent","cool","warm"].flatMap(k=>color(preset[k]));
    gl.uniform4fv(uniform["uPalette[0]"],new Float32Array(c));
    vec4("uAtmos",[preset.fog,preset.waves,preset.noise,preset.vignette]);
    vec4("uLight",[preset.azimuth,preset.elevation,preset.intensity,preset.ambient]);
    vec4("uMaterial",[preset.roughness,preset.metallic,preset.reflectivity,preset.bevel]);
    vec4("uSurface",[preset.depth,preset.ao,preset.edgeLight,preset.heroGlow]);
    vec4("uMotion",[preset.speed,preset.pulse,preset.parallax,preset.bloom]);
    vec4("uDetail",[preset.diffusion,preset.contrast,preset.saturation,0]);
    const styles=["ribbons","aurora","satin","halo","minimal"];
    vec4("uBackdropShape",[
      Math.max(0,styles.indexOf(preset.backgroundStyle)),
      preset.backgroundIntensity,preset.backgroundScale,preset.ribbonCurve
    ]);
    vec4("uBackdropLayers",[
      preset.ribbonLayers,preset.ribbonWidth,preset.ribbonSoftness,
      preset.backgroundFlow
    ]);
    vec4("uBackdropLight",[
      preset.backgroundGlow,preset.gradientDepth,
      preset.backgroundCenterX,preset.backgroundCenterY
    ]);
    vec4("uBackdropDetail",[
      preset.backgroundGrain,preset.backgroundParallax,
      preset.ribbonGleam,preset.backgroundContrast
    ]);
    const glassModels=["frosted","tinted","matte","solid"];
    const tapTiers=["fast","balanced","high"];
    vec4("uGlassModel",[
      Math.max(0,glassModels.indexOf(preset.glassStyle)),
      preset.glassOpacity,preset.glassBlur,preset.glassTransmission
    ]);
    vec4("uGlassOptics",[
      preset.glassTint,preset.glassScatter,
      preset.glassFresnel,preset.glassRim
    ]);
    vec4("uGlassDetail",[
      preset.glassGrain,preset.glassHighlight,preset.glassShadow,
      Math.max(0,tapTiers.indexOf(preset.glassQuality))
    ]);
    vec4("uGlassTargets",[
      preset.glassHero?1:0,preset.glassCards?1:0,
      preset.glassChrome?1:0,preset.glassCTA?1:0
    ]);
    if(sceneAvailable){
      // Never sample the texture while it is bound as an FBO attachment.
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D,null);
      gl.bindFramebuffer(gl.FRAMEBUFFER,sceneFbo);
      gl.viewport(0,0,sceneSize[0],sceneSize[1]);
      gl.uniform2f(uniform.uResolution,sceneSize[0],sceneSize[1]);
      gl.uniform1i(uniform.uScenePass,1);
      gl.drawArrays(gl.TRIANGLES,0,6);
      gl.bindFramebuffer(gl.FRAMEBUFFER,null);
      gl.viewport(0,0,canvas.width,canvas.height);
      gl.uniform2f(uniform.uResolution,canvas.width,canvas.height);
      gl.bindTexture(gl.TEXTURE_2D,sceneTexture);
      gl.uniform1i(uniform.uScenePass,0);
    }else gl.uniform1i(uniform.uScenePass,2);
    gl.drawArrays(gl.TRIANGLES,0,6);
  }
  function setPreset(p) {
    preset=p;geometryDirty=true;
    if(gl)resize();
  }
  function setPage(name) {
    selectedPage=String(name||"Play");geometryDirty=true;
  }
  function pause(value) {paused=!!value;return paused;}
  function recenter() {cursor=[innerWidth*.35,innerHeight*.4];}
  function screenshot() {
    if(!gl) return null;
    draw();
    return canvas.toDataURL("image/png");
  }
  function state() {return {webgl2:working,paused,error,sceneCache:sceneAvailable,
    adaptiveScale,dimensions:[canvas&&canvas.width,canvas&&canvas.height]};}
  window.JadeRenderer={init,setPreset,setPage,pause,recenter,screenshot,state,
    vertexSource:VERTEX_SOURCE,fragmentSource:FRAGMENT_SOURCE,refresh:draw};
})();