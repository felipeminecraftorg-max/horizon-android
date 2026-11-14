package dev.horizon.jadelab;

import android.app.Activity;
import android.content.ContentValues;
import android.content.Intent;
import android.graphics.Color;
import android.net.Uri;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.os.Build;
import android.provider.MediaStore;
import android.util.Base64;
import android.util.Log;
import android.view.View;
import android.view.Window;
import android.view.WindowManager;
import android.webkit.ConsoleMessage;
import android.webkit.JavascriptInterface;
import android.webkit.ValueCallback;
import android.webkit.WebChromeClient;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.widget.Toast;

import java.io.OutputStream;
import java.util.Locale;
import java.util.regex.Pattern;

/** Offline full-screen Jade material laboratory; intentionally a separate app. */
public final class JadeLabActivity extends Activity {
    private static final String TAG = "JadeLab";
    private static final int FILE_REQUEST = 716;
    private static final int MAX_BYTES = 18 * 1024 * 1024;
    private static final Pattern SAFE_NAME = Pattern.compile("[A-Za-z0-9._-]{1,80}");
    private final Handler main = new Handler(Looper.getMainLooper());
    private WebView browser;
    private ValueCallback<Uri[]> importCallback;

    @Override public void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        Window window = getWindow();
        window.setStatusBarColor(Color.BLACK);
        window.setNavigationBarColor(Color.BLACK);
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        if (Build.VERSION.SDK_INT >= 28) {
            WindowManager.LayoutParams p = window.getAttributes();
            p.layoutInDisplayCutoutMode =
                WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES;
            window.setAttributes(p);
        }
        immersive();

        browser = new WebView(this);
        browser.setBackgroundColor(Color.rgb(7, 17, 13));
        browser.setLayerType(View.LAYER_TYPE_HARDWARE, null);
        WebSettings settings = browser.getSettings();
        settings.setJavaScriptEnabled(true);
        settings.setDomStorageEnabled(true);
        settings.setAllowFileAccess(true);
        settings.setAllowContentAccess(true);
        settings.setJavaScriptCanOpenWindowsAutomatically(false);
        settings.setSupportMultipleWindows(false);
        settings.setMixedContentMode(WebSettings.MIXED_CONTENT_NEVER_ALLOW);
        settings.setMediaPlaybackRequiresUserGesture(true);
        browser.setOverScrollMode(View.OVER_SCROLL_NEVER);
        browser.setWebContentsDebuggingEnabled(false);
        browser.setWebChromeClient(new WebChromeClient() {
            @Override public boolean onShowFileChooser(WebView view,
                  ValueCallback<Uri[]> callback, FileChooserParams params) {
                if (importCallback != null) importCallback.onReceiveValue(null);
                importCallback = callback;
                Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
                intent.addCategory(Intent.CATEGORY_OPENABLE);
                intent.setType("*/*");
                intent.putExtra(Intent.EXTRA_MIME_TYPES,
                    new String[]{"application/json", "application/zip",
                                 "application/octet-stream", "text/plain"});
                try {
                    startActivityForResult(intent, FILE_REQUEST);
                    return true;
                } catch (Exception failure) {
                    importCallback = null;
                    message("Nenhum seletor de arquivos disponível");
                    return false;
                }
            }
            @Override public boolean onConsoleMessage(ConsoleMessage m) {
                Log.i(TAG, m.messageLevel()+": "+m.message()+" @ "+m.lineNumber());
                return true;
            }
        });
        browser.setWebViewClient(new WebViewClient() {
            @Override public boolean shouldOverrideUrlLoading(WebView view, String url) {
                // A standalone offline editor never follows untrusted URLs.
                return !url.startsWith("file:///android_asset/");
            }
            @Override public boolean shouldOverrideUrlLoading(WebView view,
                    android.webkit.WebResourceRequest request) {
                return !"file".equals(request.getUrl().getScheme())
                    || !request.getUrl().toString().startsWith("file:///android_asset/");
            }
        });
        browser.addJavascriptInterface(new DownloadsBridge(), "AndroidBridge");
        setContentView(browser);
        browser.loadUrl("file:///android_asset/index.html");
    }

    private void immersive() {
        getWindow().getDecorView().setSystemUiVisibility(
            View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
            | View.SYSTEM_UI_FLAG_FULLSCREEN
            | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
            | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
            | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
            | View.SYSTEM_UI_FLAG_LAYOUT_STABLE);
    }

    @Override public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus) immersive();
    }

    @Override protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (requestCode != FILE_REQUEST) return;
        if (importCallback == null) return;
        Uri result = resultCode == RESULT_OK && data != null ? data.getData() : null;
        importCallback.onReceiveValue(result != null ? new Uri[]{result} : null);
        importCallback = null;
    }

    @Override public void onBackPressed() {
        if (browser == null) { super.onBackPressed(); return; }
        browser.evaluateJavascript(
            "(function(){var e=document.getElementById('editor');"+
            "if(e&&!e.classList.contains('hidden')&&!e.classList.contains('collapsed')){"+
            "e.classList.add('collapsed');return true;}return false;})()",
            result -> {
                if (!"true".equals(result)) JadeLabActivity.super.onBackPressed();
            });
    }

    @Override protected void onDestroy() {
        if (importCallback != null) {
            importCallback.onReceiveValue(null);
            importCallback = null;
        }
        if (browser != null) {
            browser.removeJavascriptInterface("AndroidBridge");
            browser.stopLoading();
            browser.destroy();
            browser = null;
        }
        super.onDestroy();
    }

    private void message(String s) {
        main.post(() -> Toast.makeText(getApplicationContext(), s, Toast.LENGTH_SHORT).show());
    }

    /** Restricted local-save bridge; it never grants arbitrary filesystem access. */
    private final class DownloadsBridge {
        @JavascriptInterface public void saveBase64(String name, String mime, String content) {
            if (name == null || !SAFE_NAME.matcher(name).matches()) {
                message("Nome do arquivo inválido");return;
            }
            String m = mime == null ? "" : mime.toLowerCase(Locale.ROOT);
            if (!m.equals("application/json") && !m.equals("application/zip")
                    && !m.equals("image/png") && !m.equals("text/plain")) {
                message("Formato não permitido");return;
            }
            if (content == null || content.length() > (MAX_BYTES * 4L / 3L + 16)) {
                message("Arquivo muito grande");return;
            }
            try {
                byte[] bytes = Base64.decode(content, Base64.DEFAULT);
                if (bytes.length > MAX_BYTES) throw new IllegalArgumentException("Tamanho excedido");
                ContentValues values = new ContentValues();
                values.put(MediaStore.Downloads.DISPLAY_NAME, name);
                values.put(MediaStore.Downloads.MIME_TYPE, m);
                values.put(MediaStore.Downloads.RELATIVE_PATH, "Download/JadeLab");
                values.put(MediaStore.Downloads.IS_PENDING, 1);
                Uri uri = getContentResolver().insert(
                    MediaStore.Downloads.EXTERNAL_CONTENT_URI, values);
                if (uri == null) throw new IllegalStateException("Downloads indisponível");
                try {
                    try (OutputStream out = getContentResolver().openOutputStream(uri)) {
                        if (out == null) throw new IllegalStateException("Não foi possível gravar");
                        out.write(bytes);
                    }
                    values.clear();
                    values.put(MediaStore.Downloads.IS_PENDING, 0);
                    getContentResolver().update(uri, values, null, null);
                    message("Salvo em Downloads/JadeLab: "+name);
                } catch (Exception writeError) {
                    getContentResolver().delete(uri, null, null);
                    throw writeError;
                }
            } catch (Exception error) {
                Log.w(TAG, "Jade Lab export failed", error);
                message("Falha no download: "+error.getMessage());
            }
        }
    }
}
