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
void main() {
  vec2 p=vec2(gl_FragCoord.x/uResolution.x,1.0-gl_FragCoord.y/uResolution.y)*uViewport;
  vec2 uv=p/max(uViewport,vec2(1.0));
  vec3 bg=uPalette[0].rgb;
  float tt=uTime*uMotion.x*(1.0-uReduced);
  float fog=radial(uv,vec2(0.29+0.025*sin(tt*.12),0.61),vec2(.7,.8));
  float leftGlow=radial(uv,vec2(.86,.39),vec2(.9,.65));
  bg=mix(bg,uPalette[1].rgb,clamp(fog*uAtmos.x*.59,0.0,.54));
  bg+=uPalette[2].rgb*fog*uAtmos.x*.058;
  bg+=uPalette[3].rgb*leftGlow*uAtmos.x*.021;
  // Broad layered energy ribbons, not random moving particles.
  float wave=sin(uv.y*9.4+uv.x*5.2+tt*.29+sin(uv.x*6.0)*.52);
  float wave2=sin(uv.y*14.0-uv.x*5.1-tt*.23);
  bg+=uPalette[2].rgb*max(0.0,wave)*uAtmos.y*.012;
  bg+=uPalette[3].rgb*max(0.0,wave2)*uAtmos.y*.008;
  float nv=length((uv-vec2(.5)) * vec2(.95,1.05));
  bg*=1.0-uAtmos.w*smoothstep(.24,.78,nv)*.51;
  float n=hash12(floor(p*1.18+vec2(fract(tt*.07)*37.0)));
  bg+=(n-.5)*uAtmos.z*.035;

  vec3 result=bg;
  vec2 cursor=(uPointer-.5*uViewport)/max(uViewport,vec2(1.0));
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
  let preset, paused = false, last = 0, elapsed = 0, dpr = 1, renderCount = 0;
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
        "uRects[0]","uPalette[0]","uAtmos","uLight","uMaterial","uSurface","uMotion","uDetail"];
      keys.forEach(k => { uniform[k] = gl.getUniformLocation(program, k); });
      buffer = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
      gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1,-1,1,-1,-1,1, -1,1,1,-1,1,1]), gl.STATIC_DRAW);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
      cursor = [innerWidth*.35, innerHeight*.5];
      canvas.addEventListener("webglcontextlost", e => {e.preventDefault(); working=false; error="Contexto GPU perdido"; notify(error);});
      canvas.addEventListener("webglcontextrestored", () => {working=false;init();});
      window.addEventListener("pointermove", e => {if(!e.target.closest || !e.target.closest("#editor")) cursor=[e.clientX,e.clientY];}, {passive:true});
      window.addEventListener("resize", resize, {passive:true});
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
  function resize() {
    if (!gl || !canvas) return;
    const scale = Math.max(.45, Math.min(1.0, num(preset && preset.quality, .82)));
    const maxDpr = Math.min(window.devicePixelRatio || 1, 2);
    const maxDim = 2200;
    dpr = Math.min(scale * maxDpr, maxDim / Math.max(innerWidth, innerHeight));
    const w=Math.max(2,Math.round(innerWidth*dpr)),h=Math.max(2,Math.round(innerHeight*dpr));
    if(canvas.width!==w || canvas.height!==h){canvas.width=w;canvas.height=h;gl.viewport(0,0,w,h);}
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
    draw();
    renderCount++;
    if(renderCount%150===0) notify("WEBGL2 · "+(Math.round(1000 / Math.max(1,dt*1000)) || limit)+" FPS");
  }
  function vec4(name, values) {
    gl.uniform4fv(uniform[name], new Float32Array(values));
  }
  function draw() {
    if (!preset || !program || !working) return;
    gl.useProgram(program);
    gl.uniform2f(uniform.uResolution,canvas.width,canvas.height);
    gl.uniform2f(uniform.uViewport,innerWidth,innerHeight);
    gl.uniform2f(uniform.uPointer,cursor[0],cursor[1]);
    gl.uniform1f(uniform.uTime,elapsed);
    gl.uniform1f(uniform.uReduced,preset.reducedMotion?1:0);
    const rects=new Float32Array(12*4);
    let count=0;
    NAMES.forEach(name=>{
      const el=document.querySelector('[data-surface="'+name+'"]');
      if(!el || getComputedStyle(el).display==="none") return;
      const r=el.getBoundingClientRect();
      if(r.width<2 || r.height<2 || count>=12)return;
      rects.set([r.x,r.y,r.width,r.height],count*4);
      count++;
    });
    gl.uniform1i(uniform.uRectCount,count);
    gl.uniform4fv(uniform["uRects[0]"],rects);
    const c=["bg","surface","accent","cool","warm"].flatMap(k=>color(preset[k]));
    gl.uniform4fv(uniform["uPalette[0]"],new Float32Array(c));
    vec4("uAtmos",[preset.fog,preset.waves,preset.noise,preset.vignette]);
    vec4("uLight",[preset.azimuth,preset.elevation,preset.intensity,preset.ambient]);
    vec4("uMaterial",[preset.roughness,preset.metallic,preset.reflectivity,preset.bevel]);
    vec4("uSurface",[preset.depth,preset.ao,preset.edgeLight,preset.heroGlow]);
    vec4("uMotion",[preset.speed,preset.pulse,preset.parallax,preset.bloom]);
    vec4("uDetail",[preset.diffusion,preset.contrast,preset.saturation,0]);
    gl.drawArrays(gl.TRIANGLES,0,6);
  }
  function setPreset(p) {
    preset=p;
    if (gl) resize();
  }
  function pause(value) {paused=!!value;return paused;}
  function recenter() {cursor=[innerWidth*.35,innerHeight*.4];}
  function screenshot() {
    if(!gl) return null;
    draw();
    return canvas.toDataURL("image/png");
  }
  function state() {return {webgl2:working,paused,error,dimensions:[canvas&&canvas.width,canvas&&canvas.height]};}
  window.JadeRenderer={init,setPreset,pause,recenter,screenshot,state,vertexSource:VERTEX_SOURCE,fragmentSource:FRAGMENT_SOURCE,refresh:draw};
})();