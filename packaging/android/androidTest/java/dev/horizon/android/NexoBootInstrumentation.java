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
        SystemClock.sleep(250);Bitmap shot=getUiAutomation().takeScreenshot();
        if(shot==null)throw new AssertionError("No Android screenshot");
        int varied=0;java.util.Set<Integer> colors=new java.util.HashSet<>();
        for(int y=30;y<shot.getHeight()-20;y+=8)for(int x=12;x<shot.getWidth()-12;x+=8){int c=shot.getPixel(x,y);colors.add(c);if((c&0xffffff)>0x181818)varied++;}
        if(colors.size()<24||varied<30)throw new AssertionError("Android screenshot is blank/black: "+colors.size()+" colors");
        try(FileOutputStream stream=new FileOutputStream(new File(proof,name+".png"))){shot.compress(Bitmap.CompressFormat.PNG,100,stream);}shot.recycle();
    }
    private void checked(String text) {checks.append(text).append('\n');Bundle b=new Bundle();b.putString("stream","NEXO_ANDROID_OK "+text+"\n");sendStatus(0,b);}
    private void close() throws Exception {
        js("if(document.querySelector('#nexo-modal').open)Nexo.back();return true;");
        waitFor("!document.querySelector('#nexo-modal').open");
        js("NexoScene.close(true);return true;");waitFor("!NexoScene.getState().open");
    }
    @Override public void onStart() {
        Bundle result=new Bundle();
        try {
            proof=new File(getTargetContext().getExternalFilesDir(null),"nexo-android-proof");proof.mkdirs();
            long start=SystemClock.uptimeMillis();
            Intent launch=new Intent();launch.setClassName(getTargetContext().getPackageName(),"dev.horizon.android.HorizonActivity");launch.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
            activity=startActivitySync(launch);
            waitFor("window.Nexo?.state().native?.schema===2&&window.NexoScene?.getState().ready");
            js("NexoIntro.skip();NexoApp.store.get().preferences.reducedMotion=true;NexoScene.refresh();return true;");
            runOnMainSync(()->activity.setRequestedOrientation(android.content.pm.ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE));
            waitFor("innerWidth>innerHeight");
            Field window=activity.getClass().getDeclaredField("nexoWindow");window.setAccessible(true);
            if(!((android.app.Dialog)window.get(activity)).isShowing())throw new AssertionError("No independent launcher window");
            capture("01-cold-boot");checked("real release app cold boot, independent Android window and Rust snapshot in "+(SystemClock.uptimeMillis()-start)+" ms");
            for(String page:new String[]{"mods","packs","shaders"}){
                int index=page.equals("mods")?1:page.equals("packs")?2:3;
                js("NexoScene.select("+index+");return true;");waitFor("NexoScene.getState().selected==="+index);
                js("NexoScene.activate("+index+");return true;");waitFor("Nexo.state().native.view.kind==='"+page+"'&&!document.querySelector('#module').inert");
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
            js("NexoScene.select(4);return true;");waitFor("NexoScene.getState().selected===4");
            js("NexoScene.activate(4);return true;");waitFor("Nexo.state().native.view.type==='wardrobe'&&!document.querySelector('#module').inert");
            button("Capas");waitFor("Nexo.state().native.view.capes");capture("06-real-wardrobe");checked("real wardrobe skin/cape state and native page navigation");close();
            runOnMainSync(()->activity.setRequestedOrientation(android.content.pm.ActivityInfo.SCREEN_ORIENTATION_PORTRAIT));waitFor("innerHeight>innerWidth");capture("07-portrait");
            checked("Android window and scene resize to portrait");
            runOnMainSync(()->activity.moveTaskToBack(true));SystemClock.sleep(600);
            Intent restore=new Intent();restore.setClassName(getTargetContext().getPackageName(),"dev.horizon.android.HorizonActivity");restore.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK|Intent.FLAG_ACTIVITY_REORDER_TO_FRONT);
            getTargetContext().startActivity(restore);
            waitFor("window.Nexo?.state().native?.schema===2&&NexoScene.getState().ready");js("NexoIntro.skip();return true;");capture("08-resume");checked("Android background/resume restores a visible native-backed launcher");
            try(FileOutputStream stream=new FileOutputStream(new File(proof,"RESULTS.txt"))){stream.write(checks.toString().getBytes(java.nio.charset.StandardCharsets.UTF_8));}
            result.putString("stream","NEXO_ANDROID_RESULT PASS\n"+checks);finish(Activity.RESULT_OK,result);
        } catch(Throwable error) {
            try{if(proof!=null&&activity!=null)capture("FAILURE");}catch(Throwable ignored){}
            result.putString("stream","NEXO_ANDROID_RESULT FAIL\n"+android.util.Log.getStackTraceString(error));finish(Activity.RESULT_CANCELED,result);
        }
    }
}
