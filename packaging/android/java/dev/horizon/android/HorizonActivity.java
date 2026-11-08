package dev.horizon.android;

/** SAF result callback only. The native activity, UI and file operations stay Rust. */
public final class HorizonActivity extends android.app.NativeActivity {
    static { System.loadLibrary("horizon_android"); }
    private native void nativeDocumentResult(int request, int result, String uri);

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
