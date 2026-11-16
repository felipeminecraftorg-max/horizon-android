// Horizon Nexo: original Android release-boot instrumentation.
package dev.horizon.android;

import android.app.Activity;
import android.app.Instrumentation;
import android.content.Intent;
import android.graphics.Bitmap;
import android.os.Bundle;
import android.os.SystemClock;
import android.webkit.WebView;
import java.io.File;
import java.io.FileOutputStream;
import java.lang.reflect.Field;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import org.json.JSONObject;
import org.json.JSONTokener;

/** Runs the actual release application, WebView window and Rust domain backend.
 * Only the emulator's disposable app data is touched; no account or game login.
 */
public final class NexoBootInstrumentation extends Instrumentation {
    private Activity activity;
    private File proof;
    private int touchFailures;
    private final StringBuilder checks = new StringBuilder();
    @Override public void onCreate(Bundle arguments) { super.onCreate(arguments); start(); }
    private WebView view() throws Exception {
        Field field=activity.getClass().getDeclaredField("nexo");field.setAccessible(true);
        return (WebView)field.get(activity);
    }
    private String js(String script) throws Exception {
        CountDownLatch done=new CountDownLatch(1);String[] result={null};
        runOnMainSync(()->{
            try { WebView web=view(); if(web==null){done.countDown();return;}
                web.evaluateJavascript("JSON.stringify((()=>{"+script+"})())",value->{result[0]=value;done.countDown();});
            } catch(Exception e){result[0]="null";done.countDown();}
        });
        if(!done.await(8,TimeUnit.SECONDS))throw new AssertionError("Android main thread or WebView stopped responding");
        if(result[0]==null||result[0].equals("null"))return "null";
        Object parsed=new JSONTokener(result[0]).nextValue();return String.valueOf(parsed);
    }
    private void waitFor(String condition) throws Exception {
        long end=SystemClock.uptimeMillis()+45000;
        while(SystemClock.uptimeMillis()<end){
            if("true".equals(js("return !!("+condition+");")))return;
            SystemClock.sleep(120);
        }
        throw new AssertionError("Timed out: "+condition+"; state="+js("return window.Nexo?.state();"));
    }
    private void click(String selector) throws Exception {
        js("const b=document.querySelector("+JSONObject.quote(selector)+");if(!b)throw Error('Missing control');b.click();return true;");
    }
    private void button(String text) throws Exception {
        js("const b=[...document.querySelectorAll('button')].find(b=>b.textContent.trim()==="+JSONObject.quote(text)+"&&!b.disabled);if(!b)throw Error('Missing button: '+"+JSONObject.quote(text)+");b.click();return true;");
    }
    private void capture(String name) throws Exception {
        CountDownLatch painted=new CountDownLatch(1);
        runOnMainSync(()->{try{view().postVisualStateCallback(0,new WebView.VisualStateCallback(){@Override public void onComplete(long id){painted.countDown();}});}catch(Exception ignored){painted.countDown();}});
        if(!painted.await(12,TimeUnit.SECONDS))throw new AssertionError("WebView did not finish preparing its visible content");
        SystemClock.sleep(250);Bitmap shot=getUiAutomation().takeScreenshot();
        if(shot==null)throw new AssertionError("No Android screenshot");
        int varied=0;java.util.Set<Integer> colors=new java.util.HashSet<>();
        // Sample application content, excluding system bars and their antialiased
        // clock/battery glyphs, which must not make a black window pass.
        for(int y=shot.getHeight()/8;y<shot.getHeight()*7/8;y+=8)for(int x=shot.getWidth()/10;x<shot.getWidth()*9/10;x+=8){int c=shot.getPixel(x,y);colors.add(c);if(android.graphics.Color.red(c)+android.graphics.Color.green(c)+android.graphics.Color.blue(c)>96)varied++;}
        if(colors.size()<40||varied<100)throw new AssertionError("Android application content is blank/black: "+colors.size()+" colors, "+varied+" lit samples");
        try(FileOutputStream stream=new FileOutputStream(new File(proof,name+".png"))){shot.compress(Bitmap.CompressFormat.PNG,100,stream);}shot.recycle();
    }
    private void checked(String text) {checks.append(text).append('\n');Bundle b=new Bundle();b.putString("stream","NEXO_ANDROID_OK "+text+"\n");sendStatus(0,b);}
    private void openTab(int index) throws Exception {
        waitFor("!NexoScene.getState().open&&NexoScene.getState().opening<.001&&document.querySelector('#module').hidden");
        js("NexoScene.select("+index+");return true;");
        waitFor("NexoScene.getState().selected==="+index+"&&Math.abs(NexoScene.getState().position-NexoScene.getState().target)<.001&&Math.abs(Number(document.querySelector('#nexo-app').dataset.position)-NexoScene.getState().target)<.001");
        int[] position=new int[2],size=new int[2];
        runOnMainSync(()->{try{WebView web=view();web.getLocationOnScreen(position);size[0]=web.getWidth();size[1]=web.getHeight();}catch(Exception e){throw new RuntimeException(e);}});
        js("window.__nexoTouchTrace=[];if(!window.__nexoTraceInstalled){window.__nexoTraceInstalled=true;for(const kind of ['pointerdown','pointerup','touchstart','touchend','click'])document.addEventListener(kind,e=>{if(window.__nexoTouchTrace.length<20)window.__nexoTouchTrace.push({kind:e.type,target:e.target.id||e.target.tagName,x:e.clientX,y:e.clientY,button:e.button,primary:e.isPrimary,pointer:e.pointerType});},{capture:true,passive:true});}return true;");
        long time=SystemClock.uptimeMillis();float x=position[0]+size[0]/2f,y=position[1]+size[1]/2f;
        android.view.MotionEvent.PointerProperties pointer=new android.view.MotionEvent.PointerProperties();pointer.id=0;pointer.toolType=android.view.MotionEvent.TOOL_TYPE_FINGER;
        for(int action:new int[]{android.view.MotionEvent.ACTION_DOWN,android.view.MotionEvent.ACTION_UP}){
            android.view.MotionEvent.PointerCoords point=new android.view.MotionEvent.PointerCoords();point.x=x;point.y=y;point.pressure=action==android.view.MotionEvent.ACTION_UP?0:1;point.size=.1f;point.touchMajor=12;point.touchMinor=12;
            android.view.MotionEvent event=android.view.MotionEvent.obtain(time,SystemClock.uptimeMillis(),action,1,new android.view.MotionEvent.PointerProperties[]{pointer},new android.view.MotionEvent.PointerCoords[]{point},0,0,1,1,0,0,android.view.InputDevice.SOURCE_TOUCHSCREEN,0);
            try{if(!getUiAutomation().injectInputEvent(event,true))throw new AssertionError("Android rejected the tab touch");}finally{event.recycle();}
            if(action==android.view.MotionEvent.ACTION_DOWN)SystemClock.sleep(80);
        }
        long deadline=SystemClock.uptimeMillis()+12000;while(SystemClock.uptimeMillis()<deadline&&!"true".equals(js("return NexoScene.getState().open;")))SystemClock.sleep(100);
        checked("touch tab "+index+" at "+x+","+y+"; "+js("return {events:window.__nexoTouchTrace,scene:NexoScene.getState(),target:document.elementFromPoint(innerWidth/2,innerHeight/2)?.id,viewport:[innerWidth,innerHeight,devicePixelRatio]};"));
        if(!"true".equals(js("return NexoScene.getState().open;"))){touchFailures++;js("NexoScene.activate("+index+");return true;");}
        waitFor("NexoScene.getState().open");
    }
    private void close() throws Exception {
        js("if(document.querySelector('#nexo-modal').open)Nexo.back();return true;");
        waitFor("!document.querySelector('#nexo-modal').open");
        js("NexoScene.close(true);return true;");waitFor("!NexoScene.getState().open&&NexoScene.getState().opening<.001&&document.querySelector('#module').hidden");
    }
    @Override public void onStart() {
        Bundle result=new Bundle();
        try {
            proof=new File(getTargetContext().getExternalFilesDir(null),"nexo-android-proof");proof.mkdirs();
            long start=SystemClock.uptimeMillis();
            Intent launch=new Intent();launch.setClassName(getTargetContext().getPackageName(),"dev.horizon.android.HorizonActivity");launch.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
            checked("starting actual Android release activity");activity=startActivitySync(launch);
            waitFor("window.Nexo?.state().native?.schema===2&&window.NexoScene?.getState().ready");
            js("NexoIntro.skip();NexoApp.store.get().preferences.reducedMotion=true;NexoScene.refresh();return true;");
            runOnMainSync(()->activity.setRequestedOrientation(android.content.pm.ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE));
            waitFor("innerWidth>innerHeight");
            Field window=activity.getClass().getDeclaredField("nexoWindow");window.setAccessible(true);
            if(!((android.app.Dialog)window.get(activity)).isShowing())throw new AssertionError("No independent launcher window");
            capture("01-cold-boot");checked("real release app cold boot, independent Android window and Rust snapshot in "+(SystemClock.uptimeMillis()-start)+" ms");
            checked("scene="+js("return {webgl:document.querySelector('#nexo-app').dataset.webgl,compatibility:document.querySelector('#nexo-app').classList.contains('nx-compatible')};"));
            for(String page:new String[]{"mods","packs","shaders"}){
                int index=page.equals("mods")?1:page.equals("packs")?2:3;
                openTab(index);waitFor("Nexo.state().native.view.kind==='"+page+"'&&!document.querySelector('#module').inert");
                button("Minha coleção");capture("02-collection-"+page);checked("real "+page+" library and installed collection");close();
            }
            click("#settings-button");waitFor("Nexo.state().native.view.type==='settings'&&!document.querySelector('#module').inert");
            js("[...document.querySelectorAll('.nx-settings-tile')].find(b=>b.textContent.includes('Imagem e desempenho')).click();return true;");
            capture("03-real-video-settings");checked("actual native settings controls without creating a launcher Vulkan renderer");close();
            click("#files-button");waitFor("Nexo.state().native.view.type==='files'&&!document.querySelector('#module').inert");
            click("[aria-label='Organizar arquivos']");button("Nova pasta");waitFor("Nexo.state().native.modal?.type==='form'");
            js("const i=document.querySelector('#portal-body input');i.focus();i.value='Nexo QA folder';i.dispatchEvent(new Event('input',{bubbles:true}));i.blur();return true;");
            waitFor("Nexo.state().native.modal.field.value==='Nexo QA folder'");button("Salvar");
            waitFor("!Nexo.state().native.modal&&Nexo.state().native.view.entries.some(e=>e.name==='Nexo QA folder')");
            capture("04-real-file-created");checked("file manager created a real folder through native Rust actions");close();
            click("#profile-button");waitFor("Nexo.state().native.modal?.type==='account'");
            js("[...document.querySelectorAll('#portal-body button')].find(b=>b.textContent.includes('Perfil local')).click();return true;");
            waitFor("Nexo.state().native.modal.mode==='offline'");capture("05-local-profile");
            checked("actual account providers and local profile form without signing into a service");click("#portal-close");waitFor("!Nexo.state().native.modal");
            openTab(4);waitFor("Nexo.state().native.view.type==='wardrobe'&&!document.querySelector('#module').inert");
            button("Capas");waitFor("Nexo.state().native.view.capes");capture("06-real-wardrobe");checked("real wardrobe skin/cape state and native page navigation");close();
            runOnMainSync(()->activity.setRequestedOrientation(android.content.pm.ActivityInfo.SCREEN_ORIENTATION_PORTRAIT));waitFor("innerHeight>innerWidth");capture("07-portrait");
            checked("Android window and scene resize to portrait");
            runOnMainSync(()->activity.moveTaskToBack(true));SystemClock.sleep(600);
            Intent restore=new Intent();restore.setClassName(getTargetContext().getPackageName(),"dev.horizon.android.HorizonActivity");restore.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK|Intent.FLAG_ACTIVITY_REORDER_TO_FRONT);
            getTargetContext().startActivity(restore);
            waitFor("window.Nexo?.state().native?.schema===2&&NexoScene.getState().ready");js("NexoIntro.skip();return true;");capture("08-resume");checked("Android background/resume restores a visible native-backed launcher");
            try(FileOutputStream stream=new FileOutputStream(new File(proof,"RESULTS.txt"))){stream.write(checks.toString().getBytes(java.nio.charset.StandardCharsets.UTF_8));}
            if(touchFailures>0)throw new AssertionError("Native touch failed in "+touchFailures+" tabs; backend checks continued only to preserve diagnostics");
            result.putString("stream","NEXO_ANDROID_RESULT PASS\n"+checks);finish(Activity.RESULT_OK,result);
        } catch(Throwable error) {
            try{if(proof!=null&&activity!=null)capture("FAILURE");}catch(Throwable ignored){}
            result.putString("stream","NEXO_ANDROID_RESULT FAIL\n"+android.util.Log.getStackTraceString(error));finish(Activity.RESULT_CANCELED,result);
        }
    }
}
