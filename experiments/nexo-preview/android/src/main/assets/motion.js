/* Original per-mesh screen-velocity motion blur. The stationary scene has no post pass. */
(function(){
 'use strict';
 window.NexoMotion={create(gl,compile,multiply){
  const vertex=`attribute vec3 aP;uniform mat4 uModel,uPreviousModel,uVP,uPreviousVP;varying vec4 vCurrent,vPrevious;varying float vDistance;uniform float uEye;
void main(){vec4 p=uModel*vec4(aP,1.);vCurrent=uVP*p;vPrevious=uPreviousVP*uPreviousModel*vec4(aP,1.);vDistance=uEye-p.z;gl_Position=vCurrent;}`;
  const fragment=`precision highp float;varying vec4 vCurrent,vPrevious;varying float vDistance;uniform vec4 uViewport,uPreviousViewport;uniform float uExposure,uMaximum,uAlpha;
void main(){vec2 a=uViewport.xy+(vCurrent.xy/vCurrent.w*.5+.5)*uViewport.zw,b=uPreviousViewport.xy+(vPrevious.xy/vPrevious.w*.5+.5)*uPreviousViewport.zw;vec2 v=(a-b)*uExposure;v*=min(1.,uMaximum/max(length(v),.0001));gl_FragColor=vec4(.5+v/(2.*uMaximum),clamp(vDistance/32.,0.,1.),uAlpha);}`;
  const postVertex='attribute vec2 aP;varying vec2 vUV;void main(){vUV=(aP+1.)*.5;gl_Position=vec4(aP,0.,1.);}';
  const postFragment=`precision highp float;varying vec2 vUV;uniform sampler2D uColor,uVelocity;uniform vec2 uPixel;uniform float uMaximum;
vec2 velocity(vec4 sampleValue){vec2 v=(sampleValue.rg*2.-1.)*uMaximum;return sampleValue.a>.05&&length(v)>.6?v:vec2(0.);}
void main(){vec4 center=texture2D(uColor,vUV),field=texture2D(uVelocity,vUV);vec2 flow=velocity(field);float depth=field.b;
 // Dilate only along a candidate's trajectory so a moving edge leaves a short
 // shutter trail; closer stationary geometry keeps its occlusion.
 for(int k=0;k<8;k++){float a=mod(float(k),4.)*1.570796;vec2 delta=vec2(cos(a),sin(a))*(k<4?3.:10.);vec4 nearField=texture2D(uVelocity,vUV+delta*uPixel);vec2 candidate=velocity(nearField);float speed=length(candidate);float along=dot(delta,candidate)/max(speed*speed,.01),across=abs(delta.x*candidate.y-delta.y*candidate.x)/max(speed,.01);
  if(speed>length(flow)&&along>=0.&&along<=1.&&across<2.5&&(field.a<.05||nearField.b<depth+.004)){flow=candidate;depth=nearField.b;}}
 if(length(flow)<.6){gl_FragColor=center;return;}
 vec4 total=vec4(0.);float weightSum=0.;
 for(int k=0;k<9;k++){float t=float(k)/8.;vec2 uv=vUV+flow*uPixel*t;vec4 other=texture2D(uVelocity,uv);float weight=1.-t*.55;
  if(other.a>.05&&field.a>.05&&other.b<depth-.008&&length(velocity(other))<.6)weight*=.05;
  total+=texture2D(uColor,uv)*weight;weightSum+=weight;}
 gl_FragColor=total/max(weightSum,.001);
}`;
  const vp=compile(vertex,fragment),pp=compile(postVertex,postFragment),V={},P={};
  for(const name of ['uModel','uPreviousModel','uVP','uPreviousVP','uEye','uViewport','uPreviousViewport','uExposure','uMaximum','uAlpha'])V[name]=gl.getUniformLocation(vp,name);
  for(const name of ['uColor','uVelocity','uPixel','uMaximum'])P[name]=gl.getUniformLocation(pp,name);
  const va=gl.getAttribLocation(vp,'aP'),pa=gl.getAttribLocation(pp,'aP'),triangle=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,triangle);gl.bufferData(gl.ARRAY_BUFFER,new Float32Array([-1,-1,3,-1,-1,3]),gl.STATIC_DRAW);
  let history=new Map(),previousVP=null,previousViewport=null,previousTime=0,width=0,height=0,color=null,field=null,last=null,state={active:false,maxPixels:0,last:null};
  function texture(w,h){const t=gl.createTexture();gl.activeTexture(gl.TEXTURE6);gl.bindTexture(gl.TEXTURE_2D,t);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,w,h,0,gl.RGBA,gl.UNSIGNED_BYTE,null);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.LINEAR);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_S,gl.CLAMP_TO_EDGE);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_T,gl.CLAMP_TO_EDGE);return t;}
  function allocate(w,h){if(w===width&&h===height&&field)return;width=w;height=h;history.clear();previousVP=null;if(color)gl.deleteTexture(color);if(field){gl.deleteTexture(field.texture);gl.deleteFramebuffer(field.buffer);gl.deleteRenderbuffer(field.depth);}color=texture(w,h);const t=texture(w,h),b=gl.createFramebuffer(),d=gl.createRenderbuffer();gl.bindRenderbuffer(gl.RENDERBUFFER,d);gl.renderbufferStorage(gl.RENDERBUFFER,gl.DEPTH_COMPONENT16,w,h);gl.bindFramebuffer(gl.FRAMEBUFFER,b);gl.framebufferTexture2D(gl.FRAMEBUFFER,gl.COLOR_ATTACHMENT0,gl.TEXTURE_2D,t,0);gl.framebufferRenderbuffer(gl.FRAMEBUFFER,gl.DEPTH_ATTACHMENT,gl.RENDERBUFFER,d);field={texture:t,buffer:b,depth:d,ready:gl.checkFramebufferStatus(gl.FRAMEBUFFER)===gl.FRAMEBUFFER_COMPLETE};gl.bindFramebuffer(gl.FRAMEBUFFER,null);}
  function screen(matrix,p,viewport){const w=matrix[3]*p[0]+matrix[7]*p[1]+matrix[11]*p[2]+matrix[15];return [viewport[0]+((matrix[0]*p[0]+matrix[4]*p[1]+matrix[8]*p[2]+matrix[12])/w*.5+.5)*viewport[2],viewport[1]+((matrix[1]*p[0]+matrix[5]*p[1]+matrix[9]*p[2]+matrix[13])/w*.5+.5)*viewport[3]];}
  function prepare(commands,currentVP,viewport,w,h,now,enabled,selected){
   allocate(w,h);const elapsed=(now-previousTime)/1000,usable=enabled&&field.ready&&previousVP&&elapsed>0&&elapsed<1,exposure=usable?Math.min(2,(1/90)/elapsed):0,maximum=32;let max=0,frontMoving=false,left=w,right=0,bottom=h,top=0;
   for(const c of commands){const old=history.get(c.key)||c.model;c.previousModel=usable?old:c.model;if(!usable)continue;const a=multiply(currentVP,c.model),b=multiply(previousVP,old);let speed=0;
    for(const x of [c.m.bounds[0],c.m.bounds[3]])for(const y of [c.m.bounds[1],c.m.bounds[4]])for(const z of [c.m.bounds[2],c.m.bounds[5]]){const q=screen(a,[x,y,z],viewport),p=screen(b,[x,y,z],previousViewport);speed=Math.max(speed,Math.hypot(q[0]-p[0],q[1]-p[1])*exposure);if(speed>.6){left=Math.min(left,q[0],p[0]);right=Math.max(right,q[0],p[0]);bottom=Math.min(bottom,q[1],p[1]);top=Math.max(top,q[1],p[1]);}}
    max=Math.max(max,Math.min(maximum,speed));if(c.page===selected&&c.surface>.5&&speed>.6)frontMoving=true;
   }
   const region=[Math.max(0,Math.floor(left-maximum)),Math.max(0,Math.floor(bottom-maximum)),Math.min(w,Math.ceil(right+maximum)),Math.min(h,Math.ceil(top+maximum))];
   state={active:!!usable&&max>.6,maxPixels:usable?max:0,frontMoving,region,last};return {exposure,maximum,currentVP,viewport,previousVP:previousVP||currentVP,previousViewport:previousViewport||viewport,commands,now};
  }
  function finish(frame,eye){
   if(state.active){
    gl.bindFramebuffer(gl.FRAMEBUFFER,null);gl.activeTexture(gl.TEXTURE6);gl.bindTexture(gl.TEXTURE_2D,color);gl.copyTexSubImage2D(gl.TEXTURE_2D,0,0,0,0,0,width,height);
    gl.bindFramebuffer(gl.FRAMEBUFFER,field.buffer);gl.viewport(0,0,width,height);gl.clearColor(.5,.5,1,0);gl.clear(gl.COLOR_BUFFER_BIT|gl.DEPTH_BUFFER_BIT);gl.viewport(...frame.viewport);gl.enable(gl.DEPTH_TEST);gl.disable(gl.BLEND);gl.disable(gl.CULL_FACE);gl.useProgram(vp);gl.uniformMatrix4fv(V.uVP,false,frame.currentVP);gl.uniformMatrix4fv(V.uPreviousVP,false,frame.previousVP);gl.uniform4fv(V.uViewport,frame.viewport);gl.uniform4fv(V.uPreviousViewport,frame.previousViewport);gl.uniform1f(V.uExposure,frame.exposure);gl.uniform1f(V.uMaximum,frame.maximum);gl.uniform1f(V.uEye,eye);
    for(const c of frame.commands){if(c.alpha<.02)continue;gl.uniformMatrix4fv(V.uModel,false,c.model);gl.uniformMatrix4fv(V.uPreviousModel,false,c.previousModel);gl.uniform1f(V.uAlpha,c.alpha);gl.bindBuffer(gl.ARRAY_BUFFER,c.m.buffer);gl.enableVertexAttribArray(va);gl.vertexAttribPointer(va,3,gl.FLOAT,false,32,0);gl.drawArrays(gl.TRIANGLES,0,c.m.count);}
    gl.bindFramebuffer(gl.FRAMEBUFFER,null);gl.viewport(0,0,width,height);gl.disable(gl.DEPTH_TEST);gl.disable(gl.BLEND);gl.useProgram(pp);gl.activeTexture(gl.TEXTURE0);gl.bindTexture(gl.TEXTURE_2D,color);gl.activeTexture(gl.TEXTURE1);gl.bindTexture(gl.TEXTURE_2D,field.texture);gl.uniform1i(P.uColor,0);gl.uniform1i(P.uVelocity,1);gl.uniform2f(P.uPixel,1/width,1/height);gl.uniform1f(P.uMaximum,frame.maximum);gl.bindBuffer(gl.ARRAY_BUFFER,triangle);gl.enableVertexAttribArray(pa);gl.vertexAttribPointer(pa,2,gl.FLOAT,false,8,0);const r=state.region;gl.enable(gl.SCISSOR_TEST);gl.scissor(r[0],r[1],Math.max(0,r[2]-r[0]),Math.max(0,r[3]-r[1]));gl.drawArrays(gl.TRIANGLES,0,3);gl.disable(gl.SCISSOR_TEST);last={maxPixels:state.maxPixels,region:[...r],at:frame.now};state.last=last;
   }
   history=new Map(frame.commands.map(c=>[c.key,c.model]));previousVP=frame.currentVP;previousViewport=[...frame.viewport];previousTime=frame.now;
  }
  function inspect(){const result={...state};if(!field?.ready||!last)return result;gl.bindFramebuffer(gl.FRAMEBUFFER,field.buffer);const data=new Uint8Array(width*height*4);gl.readPixels(0,0,width,height,gl.RGBA,gl.UNSIGNED_BYTE,data);let moving=0,horizontal=0,vertical=0,background=0;for(let i=0;i<data.length;i+=4){if(data[i+3]<12){background++;continue;}const x=(data[i]/255*2-1)*32,y=(data[i+1]/255*2-1)*32;if(Math.hypot(x,y)>.7){moving++;horizontal+=Math.abs(x);vertical+=Math.abs(y);}}gl.bindFramebuffer(gl.FRAMEBUFFER,null);return {...result,movingPixels:moving,horizontal,vertical,backgroundPixels:background,glError:gl.getError()};}
  return {prepare,finish,inspect,getState:()=>({...state}),reset(){history.clear();previousVP=null;previousTime=0;state={active:false,maxPixels:0,last};},sources:{vertex,fragment,postVertex,postFragment}};
 }};
})();
