package dev.horizon.android;

/** Original Horizon platform host. Game/account/content operations remain Rust. */
public final class HorizonActivity extends android.app.NativeActivity {
    static { System.loadLibrary("horizon_android"); }
    private native void nativeDocumentResult(int request, int result, String uri);
    private native boolean nativeNexoRequest(String command);
    private android.webkit.WebView nexo;
    private boolean nexoReady, nexoSeen, nexoSkipIntro;
    private String nexoState;
    private Object nexoBack;
    private static final String NEXO_URL = "https://appassets.androidplatform.net/assets/nexo/index.html";

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
                settings.setBlockNetworkLoads(true);
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
                        return !NEXO_URL.equals(request.getUrl().toString());
                    }
                    @Override public void onPageFinished(android.webkit.WebView v,String url) {
                        if (v != nexo || !NEXO_URL.equals(url)) return;
                        nexoReady = true;
                        sendNexoState();
                    }
                    @Override public boolean onRenderProcessGone(android.webkit.WebView v, android.webkit.RenderProcessGoneDetail detail) {
                        failNexo(); return true;
                    }
                });
                view.addJavascriptInterface(new NexoBridge(),"HorizonNative");
                view.setOverScrollMode(android.view.View.OVER_SCROLL_NEVER);
                view.setFocusableInTouchMode(true);
                addContentView(view,new android.view.ViewGroup.LayoutParams(-1,-1));
                view.requestFocus();
                if (android.os.Build.VERSION.SDK_INT >= 33) {
                    android.window.OnBackInvokedCallback callback = this::nexoBack;
                    nexoBack = callback;
                    getOnBackInvokedDispatcher().registerOnBackInvokedCallback(android.window.OnBackInvokedDispatcher.PRIORITY_DEFAULT,callback);
                }
                view.loadUrl(NEXO_URL);
            } catch (Throwable error) { failNexo(); }
        });
    }
    private final class NexoBridge {
        @android.webkit.JavascriptInterface public boolean request(String command) {
            return command != null && command.length() <= 16384 && nativeNexoRequest(command);
        }
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
        try { nativeNexoRequest("{\"id\":"+Math.max(1,System.nanoTime())+",\"op\":\"fallback\"}"); } catch(Throwable ignored) { }
        destroyNexo();
        android.widget.Toast.makeText(this,"Old School indisponível. Retornando à interface original.",android.widget.Toast.LENGTH_LONG).show();
    }
    private void destroyNexo() {
        if (android.os.Build.VERSION.SDK_INT >= 33 && nexoBack != null) {
            getOnBackInvokedDispatcher().unregisterOnBackInvokedCallback((android.window.OnBackInvokedCallback)nexoBack);
            nexoBack=null;
        }
        android.webkit.WebView view=nexo;nexo=null;nexoReady=false;
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
