package dev.horizon.android;

/** Original Horizon platform host. Game/account/content operations remain Rust. */
public final class HorizonActivity extends android.app.NativeActivity {
    static { System.loadLibrary("horizon_android"); }
    private native void nativeDocumentResult(int request, int result, String uri);
    private native boolean nativeNexoRequest(String command);
    private android.webkit.WebView nexo;
    private android.app.Dialog nexoWindow;
    private volatile boolean nexoBackendReady;
    private final android.os.Handler nexoHandler = new android.os.Handler(android.os.Looper.getMainLooper());
    private boolean nexoReady, nexoSeen, nexoSkipIntro;
    private String nexoState;
    private Object nexoBack;
    private String nexoUrl() { return "https://appassets.androidplatform.net/assets/nexo/"+(horizonNexoOnly()?"nexo.html":"index.html"); }
    public boolean horizonNexoOnly() { return "dev.horizon.nexo".equals(getPackageName()); }

    @Override protected void onCreate(android.os.Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        // NativeActivity owns the Activity surface (Window.takeSurface). A normal
        // Dialog window supplies an independent, Android-rendered WebView surface.
        if (horizonNexoOnly()) horizonNexo(true);
    }
    public void horizonNexoBackendReady(boolean ready) { nexoBackendReady = ready; }
    private void attachNexoWindow(android.view.View content) {
        if (nexoWindow != null) nexoWindow.dismiss();
        android.app.Dialog window = new android.app.Dialog(this, R.style.HorizonGameTheme);
        nexoWindow = window;
        window.setCancelable(false);
        window.setOnKeyListener((dialog,key,event) -> {
            if (key != android.view.KeyEvent.KEYCODE_BACK) return false;
            if (event.getAction() == android.view.KeyEvent.ACTION_UP) nexoBack();
            return true;
        });
        window.setContentView(content);
        android.view.Window w = window.getWindow();
        w.setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(0xff090f19));
        w.clearFlags(android.view.WindowManager.LayoutParams.FLAG_DIM_BEHIND);
        w.addFlags(android.view.WindowManager.LayoutParams.FLAG_HARDWARE_ACCELERATED);
        w.setSoftInputMode(android.view.WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE);
        window.show();
        w.setLayout(android.view.ViewGroup.LayoutParams.MATCH_PARENT,android.view.ViewGroup.LayoutParams.MATCH_PARENT);
        if (android.os.Build.VERSION.SDK_INT >= 33) {
            android.window.OnBackInvokedCallback callback = this::nexoBack;
            nexoBack = callback;
            w.getOnBackInvokedDispatcher().registerOnBackInvokedCallback(android.window.OnBackInvokedDispatcher.PRIORITY_DEFAULT,callback);
        }
    }
    /** Only the selected Old School preset constructs this local visual view. */
    public void horizonNexo(final boolean show) {
        runOnUiThread(() -> {
            if (!show) { destroyNexo(); return; }
            if (nexo != null) return;
            try {
                nexoSkipIntro = nexoSeen;
                nexoSeen = true;
                nexoReady = false;
                android.webkit.WebView view = new android.webkit.WebView(this);
                nexo = view;
                view.setBackgroundColor(android.graphics.Color.rgb(9,15,25));
                android.webkit.WebSettings settings = view.getSettings();
                settings.setJavaScriptEnabled(true);
                settings.setDomStorageEnabled(true);
                settings.setAllowFileAccess(false);
                settings.setAllowContentAccess(false);
                settings.setAllowFileAccessFromFileURLs(false);
                settings.setAllowUniversalAccessFromFileURLs(false);
                // Requests are fenced by shouldInterceptRequest below. Blocking the
                // virtual HTTPS origin here can prevent the asset loader from running.
                settings.setBlockNetworkLoads(false);
                settings.setGeolocationEnabled(false);
                settings.setJavaScriptCanOpenWindowsAutomatically(false);
                settings.setSupportMultipleWindows(false);
                settings.setMediaPlaybackRequiresUserGesture(false);
                settings.setMixedContentMode(android.webkit.WebSettings.MIXED_CONTENT_NEVER_ALLOW);
                android.webkit.WebView.setWebContentsDebuggingEnabled(false);
                final androidx.webkit.WebViewAssetLoader loader = new androidx.webkit.WebViewAssetLoader.Builder()
                    .addPathHandler("/assets/",new androidx.webkit.WebViewAssetLoader.AssetsPathHandler(this)).build();
                view.setWebViewClient(new android.webkit.WebViewClient() {
                    @Override public android.webkit.WebResourceResponse shouldInterceptRequest(android.webkit.WebView v, android.webkit.WebResourceRequest request) {
                        android.net.Uri uri = request.getUrl();
                        if ("https".equals(uri.getScheme()) && "appassets.androidplatform.net".equals(uri.getHost())
                                && uri.getPath() != null && uri.getPath().startsWith("/assets/nexo/")) {
                            android.webkit.WebResourceResponse response = loader.shouldInterceptRequest(uri);
                            if (response != null) return response;
                        }
                        return new android.webkit.WebResourceResponse("text/plain","UTF-8",403,"Local assets only",java.util.Collections.emptyMap(),new java.io.ByteArrayInputStream(new byte[0]));
                    }
                    @Override public boolean shouldOverrideUrlLoading(android.webkit.WebView v,android.webkit.WebResourceRequest request) {
                        return !nexoUrl().equals(request.getUrl().toString());
                    }
                    @Override public void onPageFinished(android.webkit.WebView v,String url) {
                        if (v != nexo || !nexoUrl().equals(url)) return;
                        nexoReady = true;
                        nexoHandler.postDelayed(() -> {
                            if(v!=nexo)return;
                            v.evaluateJavascript("!!(window.NexoScene&&NexoScene.getState().ready)",ready->{if(v==nexo&&!"true".equals(ready))failNexo();});
                        },15000);
                        sendNexoState();
                    }
                    @Override public void onReceivedError(android.webkit.WebView v,android.webkit.WebResourceRequest request,android.webkit.WebResourceError error) {
                        if(v==nexo&&request.isForMainFrame())failNexo();
                    }
                    @Override public void onReceivedHttpError(android.webkit.WebView v,android.webkit.WebResourceRequest request,android.webkit.WebResourceResponse response) {
                        if(v==nexo&&request.isForMainFrame()&&response.getStatusCode()>=400)failNexo();
                    }
                    @Override public boolean onRenderProcessGone(android.webkit.WebView v, android.webkit.RenderProcessGoneDetail detail) {
                        failNexo(); return true;
                    }
                });
                view.setWebChromeClient(new android.webkit.WebChromeClient(){
                    @Override public boolean onConsoleMessage(android.webkit.ConsoleMessage message){
                        if(message.messageLevel()==android.webkit.ConsoleMessage.MessageLevel.ERROR)
                            android.util.Log.e("NexoUI",message.message()+" @"+message.lineNumber());
                        return false;
                    }
                });
                view.addJavascriptInterface(new NexoBridge(),"HorizonNative");
                view.setOverScrollMode(android.view.View.OVER_SCROLL_NEVER);
                view.setFocusableInTouchMode(true);
                attachNexoWindow(view);
                view.requestFocus();
                view.loadUrl(nexoUrl());
            } catch (Throwable error) { failNexo(); }
        });
    }
    private final class NexoBridge {
        @android.webkit.JavascriptInterface public boolean request(String command) {
            return command != null && command.length() <= 16384 && nativeNexoRequest(command);
        }
        @android.webkit.JavascriptInterface public boolean isReady() { return nexoBackendReady; }
        @android.webkit.JavascriptInterface public boolean skipIntro() { return nexoSkipIntro; }
    }
    public void horizonNexoState(final String json) {
        runOnUiThread(() -> { nexoState=json;sendNexoState(); });
    }
    private void sendNexoState() {
        if (nexo != null && nexoReady && nexoState != null)
            nexo.evaluateJavascript("window.HorizonHost&&HorizonHost.receive("+org.json.JSONObject.quote(nexoState)+")",null);
    }
    private void nexoBack() {
        if (nexo != null) nexo.evaluateJavascript("window.HorizonHost&&HorizonHost.back()",null);
    }
    private void failNexo() {
        if(horizonNexoOnly()) {
            destroyNexo();
            android.widget.LinearLayout recovery=new android.widget.LinearLayout(this);
            recovery.setOrientation(android.widget.LinearLayout.VERTICAL);recovery.setGravity(android.view.Gravity.CENTER);recovery.setPadding(32,32,32,32);recovery.setBackgroundColor(0xff090f19);
            android.widget.TextView title=new android.widget.TextView(this);title.setText("NEXO");title.setTextSize(30);title.setTextColor(0xffdeedef);recovery.addView(title);
            android.widget.TextView message=new android.widget.TextView(this);message.setText("Vamos reabrir sua interface. Seus arquivos continuam salvos.");message.setTextColor(0xffb2c3c8);message.setPadding(0,20,0,20);recovery.addView(message);
            android.widget.Button retry=new android.widget.Button(this);retry.setText("Reabrir Nexo");retry.setOnClickListener(v->{if(nexoWindow!=null){nexoWindow.dismiss();nexoWindow=null;}horizonNexo(true);});recovery.addView(retry);attachNexoWindow(recovery);return;
        }

        try { nativeNexoRequest("{\"id\":"+Math.max(1,System.nanoTime())+",\"op\":\"fallback\"}"); } catch(Throwable ignored) { }
        destroyNexo();
        android.widget.Toast.makeText(this,"Old School indisponível. Retornando à interface original.",android.widget.Toast.LENGTH_LONG).show();
    }
    private void destroyNexo() {
        if (android.os.Build.VERSION.SDK_INT >= 33 && nexoBack != null) {
            if(nexoWindow!=null)nexoWindow.getWindow().getOnBackInvokedDispatcher().unregisterOnBackInvokedCallback((android.window.OnBackInvokedCallback)nexoBack);
            nexoBack=null;
        }
        android.webkit.WebView view=nexo;nexo=null;nexoReady=false;
        if(nexoWindow!=null){nexoWindow.dismiss();nexoWindow=null;}
        if (view != null) {
            try { view.evaluateJavascript("window.Nexo&&Nexo.setVisible(false)",null); view.stopLoading(); } catch (Throwable ignored) { }
            android.view.ViewParent parent=view.getParent();
            if(parent instanceof android.view.ViewGroup) ((android.view.ViewGroup)parent).removeView(view);
            view.removeJavascriptInterface("HorizonNative");
            view.destroy();
            // The WebView's WebGL context is released before native gameplay.
            getWindow().getDecorView().requestFocus();
        }
    }
    @Override public void onBackPressed() { if(nexo!=null) nexoBack();else super.onBackPressed(); }
    @Override protected void onPause() {
        if(nexo!=null){nexo.evaluateJavascript("window.Nexo&&Nexo.setVisible(false)",null);nexo.onPause();}
        super.onPause();
    }
    @Override protected void onResume() {
        super.onResume();
        if(nexo!=null){nexo.onResume();nexo.evaluateJavascript("window.Nexo&&Nexo.setVisible(true)",null);}
    }
    @Override protected void onDestroy() {destroyNexo();super.onDestroy();}

    /**
     * Asks the display for its mode nearest {@code hz} at the current resolution, the
     * fastest when {@code hz} is 0. Some systems keep apps at 60 Hz unless the window asks
     * for a mode of its own; the native frame-rate hint alone does not always get one.
     */
    public void horizonPreferRefreshRate(final float hz) {
        runOnUiThread(new Runnable() {
            @Override public void run() {
                try {
                    android.view.Window window = getWindow();
                    android.view.Display display = getWindowManager().getDefaultDisplay();
                    android.view.Display.Mode current = display.getMode();
                    android.view.Display.Mode best = null;
                    for (android.view.Display.Mode mode : display.getSupportedModes()) {
                        if (mode.getPhysicalWidth() != current.getPhysicalWidth()
                                || mode.getPhysicalHeight() != current.getPhysicalHeight()) {
                            continue;
                        }
                        if (best == null) {
                            best = mode;
                            continue;
                        }
                        float rate = mode.getRefreshRate();
                        float kept = best.getRefreshRate();
                        if (hz <= 0f ? rate > kept : Math.abs(rate - hz) < Math.abs(kept - hz)) {
                            best = mode;
                        }
                    }
                    if (best == null) return;
                    android.view.WindowManager.LayoutParams params = window.getAttributes();
                    if (params.preferredDisplayModeId == best.getModeId()) return;
                    params.preferredDisplayModeId = best.getModeId();
                    params.preferredRefreshRate = best.getRefreshRate();
                    window.setAttributes(params);
                } catch (Throwable ignored) {
                    // The native hint still applies.
                }
            }
        });
    }

    /** The refresh rate the display runs at now, for the performance overlay. */
    public float horizonDisplayRefreshRate() {
        try {
            return getWindowManager().getDefaultDisplay().getRefreshRate();
        } catch (Throwable ignored) {
            return 0f;
        }
    }

    @Override protected void onActivityResult(int request, int result, android.content.Intent data) {
        super.onActivityResult(request, result, data);
        if (request != 4101) return;
        nativeDocumentResult(request, result,
                data == null || data.getData() == null ? null : data.getData().toString());
    }
}
